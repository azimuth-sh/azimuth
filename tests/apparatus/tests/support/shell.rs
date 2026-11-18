//! Comments and echoed command names must not grant a release job publishing rights.

pub fn commands(script: &str) -> Vec<Vec<String>> {
    let script = script.replace("\\\n", "");
    let mut result = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escape = false;
    let mut comment = false;
    for c in script.chars().chain(std::iter::once('\n')) {
        if comment {
            if c == '\n' {
                comment = false;
            } else {
                continue;
            }
        }
        if escape {
            word.push(c);
            escape = false;
            continue;
        }
        if c == '\\' && quote != Some('\'') {
            escape = true;
            continue;
        }
        if let Some(open) = quote {
            if c == open {
                quote = None;
            } else {
                word.push(c);
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            continue;
        }
        if c == '#' && word.is_empty() {
            comment = true;
            continue;
        }
        if c.is_whitespace() || matches!(c, ';' | '|' | '&') {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            if (c == '\n' || matches!(c, ';' | '|' | '&')) && !words.is_empty() {
                while words.first().is_some_and(|w| {
                    w.contains('=') || ["then", "do", "env", "exec"].contains(&w.as_str())
                }) {
                    words.remove(0);
                }
                if !words.is_empty() {
                    result.push(std::mem::take(&mut words));
                }
            }
        } else {
            word.push(c);
        }
    }
    assert!(
        quote.is_none(),
        "an unterminated quote in a release step: {script}"
    );
    result
}

pub fn invokes(words: &[String], expected: &[&str]) -> bool {
    words.len() >= expected.len()
        && words
            .iter()
            .zip(expected)
            .enumerate()
            .all(|(i, (word, expected))| {
                if i == 0 {
                    std::path::Path::new(word)
                        .file_name()
                        .is_some_and(|name| name == *expected)
                } else {
                    word == expected
                }
            })
}

#[test]
fn comments_and_echoes_are_not_publishing_commands() {
    let actual=commands("# npm publish\necho 'npm stage publish'\nVERSION=x npm stage publish 'package/a.tgz' # npm publish\n");
    assert_eq!(
        actual,
        vec![
            vec!["echo", "npm stage publish"],
            vec!["npm", "stage", "publish", "package/a.tgz"]
        ]
    );
    assert_eq!(
        actual
            .iter()
            .filter(|w| invokes(w, &["npm", "stage", "publish"]))
            .count(),
        1
    );
    assert!(!actual.iter().any(|w| invokes(w, &["npm", "publish"])));
}
