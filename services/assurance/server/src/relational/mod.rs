mod account;
mod assessment;
mod main_model;
mod release;
mod run;
mod status;
mod target;

use serde_json::Value;
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub(super) enum Schema {
    Text,
    Number,
    Boolean,
    // Never queried by field: held as JSON text so a record's tail needs no tables.
    Document,
    Object(Vec<(&'static str, Schema)>),
    List(Box<Schema>),
    Ref(&'static str),
}
fn object(fields: Vec<(&'static str, Schema)>) -> Schema {
    Schema::Object(fields)
}
fn list(item: Schema) -> Schema {
    Schema::List(Box::new(item))
}

#[derive(Clone)]
struct Node {
    name: String,
    schema: Schema,
    children: Vec<usize>,
    parent: Option<usize>,
}
struct Registry {
    nodes: Vec<Node>,
    roots: BTreeMap<&'static str, usize>,
}
fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = Registry {
            nodes: Vec::new(),
            roots: BTreeMap::new(),
        };
        for (kind, schema) in run::schemas()
            .into_iter()
            .chain(account::schemas())
            .chain(release::schemas())
            .chain(status::schemas())
            .chain(target::schemas())
            .chain(assessment::schemas())
            .chain(main_model::schemas())
        {
            assert!(
                !registry.roots.contains_key(kind),
                "duplicate relational record schema"
            );
            let root = registry.add(format!("assurance_{kind}"), schema, None);
            registry.roots.insert(kind, root);
        }
        registry
    })
}
impl Registry {
    fn add(&mut self, path: String, schema: Schema, parent: Option<usize>) -> usize {
        let name = table_name(&path);
        let id = self.nodes.len();
        self.nodes.push(Node {
            name,
            schema: schema.clone(),
            children: Vec::new(),
            parent,
        });
        let fields: Vec<(String, Schema)> = match &schema {
            Schema::Object(fields) => fields
                .iter()
                .filter(|(_, s)| !scalar(s))
                .map(|(f, s)| ((*f).into(), s.clone()))
                .collect(),
            Schema::List(item) => vec![("item".into(), (**item).clone())],
            _ => Vec::new(),
        };
        for (field, child) in fields {
            let child = self.add(format!("{path}_{field}"), child, Some(id));
            self.nodes[id].children.push(child);
        }
        id
    }
}
fn table_name(path: &str) -> String {
    // PostgreSQL truncates identifiers at 63 bytes, so long paths keep a stable hashed suffix.
    if path.len() > 51 {
        format!(
            "{}_{}",
            &path[..40],
            &azimuth::fingerprint::sha256(path.as_bytes())[..10]
        )
    } else {
        path.into()
    }
}

pub(crate) fn table(path: &str) -> String {
    let name = table_name(path);
    assert!(
        registry().nodes.iter().any(|node| node.name == name),
        "unknown relational table {path}"
    );
    name
}

// Generated table names may be hashed, so hand-written SQL names them through placeholders.
pub(crate) fn with_tables(sql: &str) -> String {
    sql.replace(
        "{verification_challenge_executions}",
        &table("assurance_run_verification_challenge_executions_item"),
    )
    .replace(
        "{context_capture_capture}",
        &table("assurance_context_capture_capture"),
    )
}

pub(crate) fn assessment_triggers() -> String {
    with_tables(include_str!("assessment_triggers.sql"))
}

fn scalar(schema: &Schema) -> bool {
    matches!(
        schema,
        Schema::Text | Schema::Number | Schema::Boolean | Schema::Document
    )
}
fn sql_type(schema: &Schema) -> &'static str {
    match schema {
        Schema::Text | Schema::Document => "text",
        Schema::Number => "numeric",
        Schema::Boolean => "boolean",
        _ => unreachable!(),
    }
}
fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
fn column(field: &str) -> String {
    quote(&format!("f_{field}"))
}
fn invalid(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Protocol(message.into())
}

const SCHEMA_VERSION: i32 = 9;

pub(crate) async fn initialize(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(848332101)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS assurance_storage_schema (version integer PRIMARY KEY, signature text NOT NULL)").execute(&mut *tx).await?;
    let reg = registry();
    let signature = azimuth::fingerprint::sha256(
        format!(
            "{:?}",
            reg.roots
                .iter()
                .map(|(kind, root)| (kind, &reg.nodes[*root].schema))
                .collect::<Vec<_>>()
        )
        .as_bytes(),
    );
    let existing: Option<(i32, String)> = sqlx::query_as(
        "SELECT version,signature FROM assurance_storage_schema ORDER BY version DESC LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((version, existing)) = &existing {
        if *version != SCHEMA_VERSION || existing != &signature {
            return Err(invalid(
                "relational schema changed without an explicit schema version",
            ));
        }
        tx.commit().await?;
        return Ok(());
    }
    sqlx::query("CREATE TABLE IF NOT EXISTS assurance_records (id bigserial PRIMARY KEY, kind text NOT NULL, project text NOT NULL, record_key text NOT NULL, UNIQUE(kind,project,record_key))").execute(&mut *tx).await?;
    for (id, node) in reg.nodes.iter().enumerate() {
        let mut columns = vec![
            "_record_id bigint NOT NULL REFERENCES assurance_records(id)".to_string(),
            "_node bigint NOT NULL".into(),
            "_parent bigint".into(),
            "_position integer NOT NULL".into(),
            "_map_key text".into(),
            "_is_null boolean NOT NULL".into(),
            "_present text[] NOT NULL".into(),
            "_nulls text[] NOT NULL".into(),
        ];
        match &node.schema {
            s if scalar(s) => columns.push(format!("_value {}", sql_type(s))),
            Schema::Object(fields) => {
                for (field, schema) in fields.iter().filter(|(_, s)| scalar(s)) {
                    columns.push(format!("{} {}", column(field), sql_type(schema)));
                }
            }
            Schema::Ref(_) => {
                columns.push("_ref_kind text".into());
                columns.push("_ref_project text".into());
                columns.push("_ref_key text".into());
                columns.push("FOREIGN KEY(_ref_kind,_ref_project,_ref_key) REFERENCES assurance_records(kind,project,record_key) DEFERRABLE INITIALLY DEFERRED".into());
            }
            _ => (),
        }
        columns.push("PRIMARY KEY(_record_id,_node)".into());
        columns.push("UNIQUE(_record_id,_parent,_position)".into());
        columns.push("UNIQUE(_record_id,_parent,_map_key)".into());
        if let Some(parent) = node.parent {
            columns.push(format!("FOREIGN KEY(_record_id,_parent) REFERENCES {}(_record_id,_node) DEFERRABLE INITIALLY DEFERRED",quote(&reg.nodes[parent].name)));
        }
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            quote(&node.name),
            columns.join(",")
        ))
        .execute(&mut *tx)
        .await?;
        assert_eq!(
            id,
            reg.nodes
                .iter()
                .position(|n| n.name == node.name)
                .expect("node"),
            "relational table name collision"
        );
    }
    // Ref readers may recurse, so their shared dispatcher must exist before SQL bodies are checked.
    sqlx::query("CREATE OR REPLACE FUNCTION assurance_record(k text,p text,r text) RETURNS jsonb LANGUAGE plpgsql STABLE AS $$ BEGIN RETURN NULL; END $$").execute(&mut *tx).await?;
    for node in reg.nodes.iter().rev() {
        let expression = read_expression(reg, node);
        let sql = format!("CREATE OR REPLACE FUNCTION {}(rid bigint,n bigint) RETURNS jsonb LANGUAGE sql STABLE AS $$ SELECT CASE WHEN t._is_null THEN 'null'::jsonb ELSE {expression} END FROM {} t WHERE t._record_id=rid AND t._node=n $$",quote(&format!("read_{}",node.name)),quote(&node.name));
        sqlx::query(&sql).execute(&mut *tx).await?;
    }
    let cases = reg
        .roots
        .iter()
        .map(|(kind, root)| {
            format!(
                "WHEN {} THEN RETURN {}(rid,0);",
                literal(kind),
                quote(&format!("read_{}", reg.nodes[*root].name))
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    sqlx::query(&format!("CREATE OR REPLACE FUNCTION assurance_record(k text,p text,r text) RETURNS jsonb LANGUAGE plpgsql STABLE AS $$ DECLARE rid bigint; BEGIN SELECT id INTO rid FROM assurance_records WHERE kind=k AND project=p AND record_key=r; IF NOT FOUND THEN RETURN NULL; END IF; CASE k {cases} ELSE RETURN NULL; END CASE; END $$")).execute(&mut *tx).await?;
    let mut field_cases = Vec::new();
    for (kind, root) in &reg.roots {
        let node = &reg.nodes[*root];
        if let Schema::Object(fields) = &node.schema {
            let cases = fields
                .iter()
                .map(|(field, schema)| {
                    format!(
                        "WHEN {} THEN {}",
                        literal(field),
                        object_field(reg, node, field, schema)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            sqlx::query(&format!("CREATE OR REPLACE FUNCTION {}(rid bigint,f text) RETURNS jsonb LANGUAGE sql STABLE AS $$ SELECT CASE f {cases} ELSE NULL END FROM {} t WHERE t._record_id=rid AND t._node=0 $$",quote(&format!("field_{}",node.name)),quote(&node.name))).execute(&mut *tx).await?;
            field_cases.push(format!(
                "WHEN {} THEN {}(id,f)",
                literal(kind),
                quote(&format!("field_{}", node.name))
            ));
        } else {
            field_cases.push(format!(
                "WHEN {} THEN assurance_record(k,p,r)->f",
                literal(kind)
            ));
        }
    }
    sqlx::query(&format!("CREATE OR REPLACE FUNCTION assurance_record_field(k text,p text,r text,f text) RETURNS jsonb LANGUAGE sql STABLE AS $$ SELECT CASE k {} ELSE NULL END FROM assurance_records WHERE kind=k AND project=p AND record_key=r $$",field_cases.join(" "))).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO assurance_storage_schema(version,signature) VALUES($1,$2)")
        .bind(SCHEMA_VERSION)
        .bind(signature)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
fn child_read(reg: &Registry, child: usize) -> String {
    let name = &reg.nodes[child].name;
    format!("(SELECT {}(rid,c._node) FROM {} c WHERE c._record_id=rid AND c._parent=t._node ORDER BY c._position LIMIT 1)",quote(&format!("read_{name}")),quote(name))
}
fn object_field(reg: &Registry, node: &Node, field: &str, schema: &Schema) -> String {
    let value = if scalar(schema) {
        scalar_read(schema, &format!("t.{}", column(field)))
    } else {
        let Schema::Object(fields) = &node.schema else {
            unreachable!()
        };
        let index = fields
            .iter()
            .filter(|(_, s)| !scalar(s))
            .position(|(f, _)| *f == field)
            .unwrap();
        child_read(reg, node.children[index])
    };
    format!(
        "CASE WHEN {}=ANY(t._nulls) THEN 'null'::jsonb ELSE {value} END",
        literal(field)
    )
}
fn scalar_read(schema: &Schema, expression: &str) -> String {
    match schema {
        Schema::Document => format!("{expression}::jsonb"),
        _ => format!("to_jsonb({expression})"),
    }
}
fn read_expression(reg: &Registry, node: &Node) -> String {
    match &node.schema {
        s if scalar(s) => scalar_read(s, "t._value"),
        Schema::Object(fields) => {
            let fields=fields.iter().map(|(field,schema)|format!("CASE WHEN {}=ANY(t._present) THEN jsonb_build_object({}, {}) ELSE '{{}}'::jsonb END",literal(field),literal(field),object_field(reg,node,field,schema))).collect::<Vec<_>>();
            if fields.is_empty() {
                "'{}'::jsonb".into()
            } else {
                fields.join(" || ")
            }
        }
        Schema::List(_) => {
            let child = &reg.nodes[node.children[0]].name;
            format!("COALESCE((SELECT jsonb_agg({}(rid,c._node) ORDER BY c._position) FROM {} c WHERE c._record_id=rid AND c._parent=t._node),'[]'::jsonb)",quote(&format!("read_{child}")),quote(child))
        }
        Schema::Ref(_) => "assurance_record(t._ref_kind,t._ref_project,t._ref_key)".into(),
        _ => unreachable!(),
    }
}

pub(crate) async fn load(
    pool: &PgPool,
    kind: &str,
    project: &str,
    key: &str,
) -> Result<Option<Value>, sqlx::Error> {
    sqlx::query_scalar("SELECT assurance_record($1,$2,$3)")
        .bind(kind)
        .bind(project)
        .bind(key)
        .fetch_one(pool)
        .await
}
fn content_key(value: &Value) -> Result<String, sqlx::Error> {
    if let Some(key) = value
        .get("bundle_fingerprint")
        .or_else(|| value.get("fingerprint"))
        .and_then(Value::as_str)
    {
        return Ok(key.into());
    }
    let native = azimuth::run::strict_json("relational reference", &value.to_string())
        .map_err(|e| invalid(e.to_string()))?;
    azimuth::run::canonical_fingerprint(&native).map_err(invalid)
}
#[derive(Clone)]
struct Pending {
    kind: String,
    project: String,
    key: String,
    value: Value,
}
struct StoredRow {
    node: i64,
    parent: Option<i64>,
    position: i32,
    map_key: Option<String>,
    is_null: bool,
    present: Vec<String>,
    nulls: Vec<String>,
    values: Vec<Option<String>>,
    reference: Option<(String, String, String)>,
}
struct Writer<'a> {
    reg: &'a Registry,
    next: i64,
    rows: BTreeMap<usize, Vec<StoredRow>>,
    pending: Vec<Pending>,
    project: &'a str,
}
impl Writer<'_> {
    fn flatten(
        &mut self,
        id: usize,
        value: &Value,
        parent: Option<i64>,
        position: i32,
        map_key: Option<String>,
        depth: usize,
    ) -> Result<(), sqlx::Error> {
        if depth > 128 {
            return Err(invalid("relational record exceeds structural depth"));
        }
        let node = self.reg.nodes[id].clone();
        let current = self.next;
        self.next += 1;
        let mut row = StoredRow {
            node: current,
            parent,
            position,
            map_key,
            is_null: value.is_null(),
            present: Vec::new(),
            nulls: Vec::new(),
            values: Vec::new(),
            reference: None,
        };
        if !value.is_null() {
            match &node.schema {
                s if scalar(s) => {
                    row.values.push(Some(scalar_value(s, value).ok_or_else(|| {
                        invalid(format!("{} requires {}", node.name, sql_type(s)))
                    })?))
                }
                Schema::Object(fields) => {
                    let object = value
                        .as_object()
                        .ok_or_else(|| invalid(format!("{} requires object", node.name)))?;
                    if let Some(key) = object
                        .keys()
                        .find(|key| !fields.iter().any(|(name, _)| *name == key.as_str()))
                    {
                        return Err(invalid(format!("{}: undeclared field {key}", node.name)));
                    }
                    row.present = object.keys().cloned().collect();
                    row.nulls = object
                        .iter()
                        .filter(|(_, v)| v.is_null())
                        .map(|(k, _)| k.clone())
                        .collect();
                    let mut child = 0;
                    for (field, schema) in fields {
                        if scalar(schema) {
                            row.values.push(
                                object
                                    .get(*field)
                                    .filter(|v| !v.is_null())
                                    .map(|v| {
                                        scalar_value(schema, v).ok_or_else(|| {
                                            invalid(format!(
                                                "{}.{field}: invalid scalar",
                                                node.name
                                            ))
                                        })
                                    })
                                    .transpose()?,
                            );
                        } else {
                            if let Some(v) = object.get(*field).filter(|v| !v.is_null()) {
                                self.flatten(
                                    node.children[child],
                                    v,
                                    Some(current),
                                    0,
                                    None,
                                    depth + 1,
                                )?;
                            }
                            child += 1;
                        }
                    }
                }
                Schema::List(_) => {
                    for (i, v) in value
                        .as_array()
                        .ok_or_else(|| invalid(format!("{} requires array", node.name)))?
                        .iter()
                        .enumerate()
                    {
                        self.flatten(
                            node.children[0],
                            v,
                            Some(current),
                            i.try_into().map_err(|_| invalid("array too long"))?,
                            None,
                            depth + 1,
                        )?;
                    }
                }
                Schema::Ref(kind) => {
                    if !self.reg.roots.contains_key(kind) {
                        return Err(invalid(format!("unknown relational kind {kind}")));
                    }
                    let key = content_key(value)?;
                    row.reference = Some(((*kind).into(), self.project.into(), key.clone()));
                    self.pending.push(Pending {
                        kind: (*kind).into(),
                        project: self.project.into(),
                        key,
                        value: value.clone(),
                    });
                }
                _ => unreachable!(),
            }
        } else {
            match &node.schema {
                s if scalar(s) => row.values.push(None),
                Schema::Object(fields) => {
                    row.values = fields
                        .iter()
                        .filter(|(_, s)| scalar(s))
                        .map(|_| None)
                        .collect()
                }
                _ => (),
            }
        }
        self.rows.entry(id).or_default().push(row);
        Ok(())
    }
}
fn scalar_value(schema: &Schema, value: &Value) -> Option<String> {
    match schema {
        Schema::Text => value.as_str().map(str::to_owned),
        Schema::Number if value.is_number() => Some(value.to_string()),
        Schema::Boolean => value.as_bool().map(|v| v.to_string()),
        Schema::Document => Some(value.to_string()),
        _ => None,
    }
}
pub(crate) async fn store(
    conn: &mut PgConnection,
    kind: &str,
    project: &str,
    key: &str,
    value: &Value,
) -> Result<(), sqlx::Error> {
    let reg = registry();
    let mut pending = vec![Pending {
        kind: kind.into(),
        project: project.into(),
        key: key.into(),
        value: value.clone(),
    }];
    let mut seen: HashMap<(String, String, String), Value> = HashMap::new();
    while let Some(record) = pending.pop() {
        let identity = (
            record.kind.clone(),
            record.project.clone(),
            record.key.clone(),
        );
        if let Some(previous) = seen.get(&identity) {
            if previous != &record.value {
                return Err(invalid("immutable relational reference collision"));
            }
            continue;
        }
        seen.insert(identity, record.value.clone());
        let root = *reg
            .roots
            .get(record.kind.as_str())
            .ok_or_else(|| invalid(format!("unknown relational record {}", record.kind)))?;
        let mut writer = Writer {
            reg,
            next: 0,
            rows: BTreeMap::new(),
            pending: Vec::new(),
            project: &record.project,
        };
        writer.flatten(root, &record.value, None, 0, None, 0)?;
        let id:Option<i64>=sqlx::query_scalar("INSERT INTO assurance_records(kind,project,record_key) VALUES($1,$2,$3) ON CONFLICT DO NOTHING RETURNING id").bind(&record.kind).bind(&record.project).bind(&record.key).fetch_optional(&mut *conn).await?;
        let Some(id) = id else {
            let existing: Option<Value> = sqlx::query_scalar("SELECT assurance_record($1,$2,$3)")
                .bind(&record.kind)
                .bind(&record.project)
                .bind(&record.key)
                .fetch_one(&mut *conn)
                .await?;
            if existing.as_ref() != Some(&record.value) {
                return Err(invalid(format!(
                    "immutable {} record collision",
                    record.kind
                )));
            }
            continue;
        };
        for (node_id, rows) in writer.rows {
            let node = &reg.nodes[node_id];
            let mut columns = vec![
                "_record_id".into(),
                "_node".into(),
                "_parent".into(),
                "_position".into(),
                "_map_key".into(),
                "_is_null".into(),
                "_present".into(),
                "_nulls".into(),
            ];
            match &node.schema {
                s if scalar(s) => columns.push("_value".into()),
                Schema::Object(fields) => columns.extend(
                    fields
                        .iter()
                        .filter(|(_, s)| scalar(s))
                        .map(|(f, _)| column(f)),
                ),
                Schema::Ref(_) => {
                    columns.extend(["_ref_kind".into(), "_ref_project".into(), "_ref_key".into()])
                }
                _ => (),
            }
            for chunk in rows.chunks(250) {
                let mut query = QueryBuilder::<Postgres>::new(format!(
                    "INSERT INTO {} ({}) ",
                    quote(&node.name),
                    columns.join(",")
                ));
                query.push_values(chunk, |mut q, row| {
                    q.push_bind(id)
                        .push_bind(row.node)
                        .push_bind(row.parent)
                        .push_bind(row.position)
                        .push_bind(&row.map_key)
                        .push_bind(row.is_null)
                        .push_bind(&row.present)
                        .push_bind(&row.nulls);
                    match &node.schema {
                        s if scalar(s) => {
                            q.push_bind(&row.values[0])
                                .push_unseparated(format!("::{}", sql_type(s)));
                        }
                        Schema::Object(fields) => {
                            for ((_, s), value) in
                                fields.iter().filter(|(_, s)| scalar(s)).zip(&row.values)
                            {
                                q.push_bind(value)
                                    .push_unseparated(format!("::{}", sql_type(s)));
                            }
                        }
                        Schema::Ref(_) => {
                            q.push_bind(row.reference.as_ref().map(|r| &r.0));
                            q.push_bind(row.reference.as_ref().map(|r| &r.1));
                            q.push_bind(row.reference.as_ref().map(|r| &r.2));
                        }
                        _ => (),
                    }
                });
                query.build().execute(&mut *conn).await?;
            }
        }
        pending.extend(writer.pending);
    }
    Ok(())
}

pub(crate) async fn discard_assessment_requirement(
    conn: &mut PgConnection,
    project: &str,
    key: &str,
) -> Result<(), sqlx::Error> {
    let id:Option<i64>=sqlx::query_scalar("SELECT id FROM assurance_records WHERE kind='assessment_requirement' AND project=$1 AND record_key=$2").bind(project).bind(key).fetch_optional(&mut *conn).await?;
    let Some(id) = id else {
        return Ok(());
    };
    let reg = registry();
    fn descendants(reg: &Registry, node: usize, nodes: &mut Vec<usize>) {
        for child in &reg.nodes[node].children {
            descendants(reg, *child, nodes);
        }
        nodes.push(node);
    }
    let mut nodes = Vec::new();
    descendants(reg, reg.roots["assessment_requirement"], &mut nodes);
    for node in nodes {
        sqlx::query(&format!(
            "DELETE FROM {} WHERE _record_id=$1",
            quote(&reg.nodes[node].name)
        ))
        .bind(id)
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query("DELETE FROM assurance_records WHERE id=$1")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod schema_version_tests {
    use super::*;
    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    #[tokio::test]
    async fn only_the_exact_current_schema_signature_is_accepted(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().start().await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432).await?
        ))
        .await?;
        crate::migrate(&pool).await?;
        let (version, signature): (i32, String) =
            sqlx::query_as("SELECT version,signature FROM assurance_storage_schema")
                .fetch_one(&pool)
                .await?;
        assert_eq!(version, 9);
        initialize(&pool).await?;
        for (version, signature) in [(8, signature.clone()), (9, format!("{signature}-changed"))] {
            sqlx::query("DELETE FROM assurance_storage_schema")
                .execute(&pool)
                .await?;
            sqlx::query("INSERT INTO assurance_storage_schema VALUES($1,$2)")
                .bind(version)
                .bind(&signature)
                .execute(&pool)
                .await?;
            let error = initialize(&pool)
                .await
                .expect_err("a different schema generation must be rejected");
            assert!(error
                .to_string()
                .contains("relational schema changed without an explicit schema version"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn fresh_database_reaches_schema_version_nine_and_restarts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().start().await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432).await?
        ))
        .await?;
        crate::migrate(&pool).await?;
        let version: i32 = sqlx::query_scalar("SELECT version FROM assurance_storage_schema")
            .fetch_one(&pool)
            .await?;
        assert_eq!(version, 9);
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT table_name FROM information_schema.tables WHERE table_schema='public'",
        )
        .fetch_all(&pool)
        .await?;
        for expected in [
            "main_publications",
            "main_publication_reviews",
            "assurance_main_publication",
            "assurance_release",
            "model_revisions",
            "assurance_review_artifacts_item",
            "assurance_review_challenge_basis_item",
            "assurance_authority_requirement_intents_item",
            "provenance_ledgers",
            "provenance_instances",
            "provenance_attempts",
            "provenance_attempt_targets",
            "provenance_attempt_guidance",
            "provenance_attempt_entities",
            "provenance_failure_attributions",
            "provenance_failure_attribution_attempts",
        ] {
            assert!(tables.iter().any(|t| t == expected), "missing {expected}");
        }
        for removed in [
            "current_authorities",
            "authority_selections",
            "release_acceptances",
            "assurance_review_variant_0",
        ] {
            assert!(
                !tables.iter().any(|t| t == removed),
                "{removed} must not exist"
            );
        }
        let columns: Vec<(String, String)> = sqlx::query_as(
            "SELECT table_name,column_name FROM information_schema.columns WHERE table_schema='public' AND (table_name,column_name) IN (('assurance_releases','baseline_model_revision'),('assurance_releases','integration_commit'),('assurance_releases','integration_sequence'),('assessment_requirements','supported_since_ms'))",
        )
        .fetch_all(&pool)
        .await?;
        let columns: Vec<String> = columns
            .into_iter()
            .map(|(t, c)| format!("{t}.{c}"))
            .collect();
        assert!(!columns.contains(&"assurance_releases.baseline_model_revision".into()));
        for expected in [
            "assurance_releases.integration_commit",
            "assurance_releases.integration_sequence",
            "assessment_requirements.supported_since_ms",
        ] {
            assert!(columns.iter().any(|c| c == expected), "missing {expected}");
        }
        crate::migrate(&pool).await?;
        Ok(())
    }

    #[test]
    fn hand_written_sql_names_generated_tables() {
        let triggers = assessment_triggers();
        assert!(!triggers.contains("{verification_challenge_executions}"));
        assert!(triggers.contains(&table(
            "assurance_run_verification_challenge_executions_item"
        )));
    }
}
