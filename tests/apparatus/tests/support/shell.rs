//! Comments and echoed command names must not grant a release job publishing rights.

pub fn commands(script: &str) -> Vec<Vec<String>> {
    let script = script.replace("\\\n", "");
    let mut result = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut word_started = false;
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
            word_started = true;
            escape = false;
            continue;
        }
        if c == '\\' && quote != Some('\'') {
            escape = true;
            word_started = true;
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
            word_started = true;
            continue;
        }
        if c == '#' && !word_started {
            comment = true;
            continue;
        }
        if c.is_whitespace() || matches!(c, ';' | '|' | '&') {
            if word_started {
                words.push(std::mem::take(&mut word));
                word_started = false;
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
            word_started = true;
        }
    }
    assert!(
        quote.is_none(),
        "an unterminated quote in a release step: {script}"
    );
    result
}

pub fn without_comments(script: &str) -> String {
    let mut result = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;
    let mut boundary = true;
    for c in script.chars() {
        if comment {
            if c == '\n' {
                comment = false;
                boundary = true;
                result.push(c);
            }
            continue;
        }
        if escaped {
            result.push(c);
            escaped = false;
            boundary = false;
            continue;
        }
        if c == '\\' && quote != Some('\'') {
            result.push(c);
            escaped = true;
            continue;
        }
        if let Some(open) = quote {
            result.push(c);
            if c == open {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            result.push(c);
            boundary = false;
            continue;
        }
        if c == '#' && boundary {
            comment = true;
            continue;
        }
        result.push(c);
        boundary = c.is_whitespace() || matches!(c, ';' | '|' | '&');
    }
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

#[test]
fn empty_arguments_survive_and_only_shell_comments_are_removed() {
    assert_eq!(
        commands("gh release create tag --notes \"\" dist/VERSION"),
        vec![vec![
            "gh",
            "release",
            "create",
            "tag",
            "--notes",
            "",
            "dist/VERSION"
        ]]
    );
    assert_eq!(
        without_comments("# npm publish\nsh -c 'npm publish package.tgz' # ignored\n"),
        "\nsh -c 'npm publish package.tgz' \n"
    );
}
