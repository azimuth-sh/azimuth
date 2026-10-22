use super::{list, object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![("main_publication", main_publication())]
}

fn main_publication() -> Schema {
    object(vec![
        ("format", Schema::Text),
        ("version", Schema::Number),
        ("commit", Schema::Text),
        ("main_sequence", Schema::Number),
        ("authority", Schema::Ref("authority")),
        ("reviews", list(Schema::Ref("review"))),
        ("catalog", Schema::Ref("catalog")),
        ("ratification", Schema::Document),
    ])
}
