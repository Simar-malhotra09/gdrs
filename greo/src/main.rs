use core::fmt;
use regex::Regex;
use std::borrow::Cow;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

struct SearchResult<'a> {
    file_path: Option<&'a str>,
    line_num: Option<u32>,
    contents: &'a str,
}

impl<'a> fmt::Display for SearchResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}:{}:{}\n",
            self.file_path.unwrap(),
            self.line_num.unwrap(),
            self.contents,
        )
    }
}

fn split_result(output: &str) -> Vec<&str> {
    output.lines().collect()
}

fn parse_result<'a>(split_output: &[&'a str]) -> Vec<SearchResult<'a>> {
    let mut res = Vec::new();

    for so in split_output {
        let Some((file_path, rest)) = so.split_once(':') else {
            continue;
        };

        let Some((line_num, contents)) = rest.split_once(':') else {
            continue;
        };

        let Ok(line_num) = line_num.parse::<u32>() else {
            continue;
        };

        res.push(SearchResult {
            file_path: Some(file_path),
            line_num: Some(line_num),
            contents,
        });
    }

    res
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct PathMatch {
    path: String,
    line_num: Option<u32>,
    col_num: Option<u32>,
    start: usize,
    end: usize,
}

#[allow(dead_code)]
const TRAILING_PUNCT: &[char] = &['.', ',', ';', ':', ')', ']', '}', '>', '"', '\'', '!'];

#[allow(dead_code)]
fn expand_tilde(path: &str) -> Cow<'_, str> {
    match path.strip_prefix("~/") {
        Some(rest) => match std::env::var("HOME") {
            Ok(home) => Cow::Owned(format!("{home}/{rest}")),
            Err(_) => Cow::Borrowed(path),
        },
        None => Cow::Borrowed(path),
    }
}

/// Longest prefix of `raw` that names an existing file, after trimming
/// trailing sentence punctuation the regex can't tell apart from the path.
#[allow(dead_code)]
fn validate_path(raw: &str) -> Option<usize> {
    if raw.trim_matches('/').is_empty() {
        return None;
    }
    let mut len = raw.len();
    loop {
        let candidate = &raw[..len];
        if Path::new(expand_tilde(candidate).as_ref()).exists() {
            return Some(len);
        }
        let last = candidate.chars().next_back().unwrap();
        if len > 1 && TRAILING_PUNCT.contains(&last) {
            len -= last.len_utf8();
        } else {
            return None;
        }
    }
}

#[allow(dead_code)]
fn extract_path_matches(input: &str) -> Vec<PathMatch> {
    static CANDIDATE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?P<path>[^\s:/]*/[^\s:]*|(?:[A-Za-z0-9_+~-]+\.)+[A-Za-z0-9_+~-]+)(?::(?P<line_num>[0-9]+)(?::(?P<col_num>[0-9]+))?)?",
        )
        .unwrap()
    });

    let mut matches = Vec::new();
    for caps in CANDIDATE.captures_iter(input) {
        let m = caps.name("path").unwrap();
        let raw = m.as_str();
        let Some(valid_len) = validate_path(raw) else {
            continue;
        };

        let (line_num, col_num, end) = if valid_len == raw.len() {
            (
                caps.name("line_num").map(|l| l.as_str().parse().unwrap()),
                caps.name("col_num").map(|c| c.as_str().parse().unwrap()),
                caps.get(0).unwrap().end(),
            )
        } else {
            // A :line_num suffix glued after trimmed punctuation isn't trusted.
            (None, None, m.start() + valid_len)
        };

        matches.push(PathMatch {
            path: raw[..valid_len].to_string(),
            line_num,
            col_num,
            start: m.start(),
            end,
        });
    }
    matches
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = match args.first().map(String::as_str) {
        Some("--") => &args[1..],
        _ => &args[..],
    };
    let Some((command, command_args)) = rest.split_first() else {
        eprintln!("usage: greo [--] <command> [args...]");
        std::process::exit(2);
    };

    let mut cmd = Command::new(command);
    cmd.args(command_args);

    println!("Command: {:?}", cmd);

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("greo: command not found: {command}");
            std::process::exit(127);
        }
        Err(e) => return Err(e.into()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let split_output = split_result(&stdout);
    let search_results: Vec<SearchResult> = parse_result(split_output.as_slice());
    for sr in search_results {
        println!(
            "{}, {}, {}",
            sr.file_path.unwrap(),
            sr.line_num.unwrap(),
            sr.contents
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(input: &str) -> Vec<(String, Option<u32>, Option<u32>)> {
        extract_path_matches(input)
            .into_iter()
            .map(|m| (m.path, m.line_num, m.col_num))
            .collect()
    }

    #[test]
    fn test_expand_path_valid() {
        let path = "~/Desktop";
        let expanded_path = expand_tilde(path);
        // it doesn't have the trailing '/'
        assert_eq!(expanded_path, "/Users/0saker/Desktop");
    }

    #[test]
    fn test_expand_path_invalid() {
        let path = "~//Desktop";
        let expanded_path = expand_tilde(path);
        // it doesn't have the trailing '/'
        assert_eq!(expanded_path, "/Users/0saker//Desktop");
    }

    // Todo: decide what to do
    #[test]
    fn test_expand_path_multiple_tilde() {
        let path = "~~/Desktop";
        let expanded_path = expand_tilde(path);
        assert_eq!(expanded_path, "~~/Desktop");
    }
    #[test]
    fn test_validate_path_valid() {
        let path = "~/Desktop";
        let expected_size: usize = path.len();
        assert_eq!(validate_path(path).unwrap(), expected_size);
    }
    #[test]
    fn test_validate_path_invalid_multiple_tilde() {
        let path = "~~/Desktop";
        assert!(validate_path(path).is_none());
    }
    #[test]
    fn test_validate_path_invalid_dne() {
        let path = "~/abc/def";
        assert!(validate_path(path).is_none());
    }

    #[test]
    fn test_validate_path_trailing_punc_single() {
        let path = "~/Desktop,";
        let expected_size: usize = "~/Desktop".len();
        assert_eq!(validate_path(path).unwrap(), expected_size);
    }

    #[test]
    fn test_validate_path_trailing_punc_multiple() {
        let path = "~/Desktop,!";
        let expected_size: usize = "~/Desktop".len();
        assert_eq!(validate_path(path).unwrap(), expected_size);
    }

    #[test]
    fn test_extract_path_matches_path() {
        let path = "~/Desktop";
        assert_eq!(found(path), vec![("~/Desktop".to_string(), None, None)])
    }
    #[test]
    fn test_extract_path_matches_path_and_line_num() {
        let path = "~/Desktop:12";
        assert_eq!(found(path), vec![("~/Desktop".to_string(), Some(12), None)])
    }
    #[test]
    fn test_extract_path_matches_path_and_line_num_and_col_num() {
        let path = "~/Desktop:12:10";
        assert_eq!(
            found(path),
            vec![("~/Desktop".to_string(), Some(12), Some(10))]
        )
    }
    #[test]
    fn test_extract_path_matches_path_trailing_col() {
        let path = "~/Desktop:";
        assert_eq!(found(path), vec![("~/Desktop".to_string(), None, None)])
    }
    #[test]
    fn test_extract_path_matches_path_trailing_col_multiple() {
        let path = "~/Desktop::;";
        assert_eq!(found(path), vec![("~/Desktop".to_string(), None, None)])
    }
    #[test]
    fn test_extract_path_matches_colon_chars() {
        let path = "~/Desktop:10:abc";
        assert_eq!(found(path), vec![("~/Desktop".to_string(), Some(10), None)])
    }

    #[test]
    fn fixture_bare_path_no_line_number() {
        assert_eq!(
            found(include_str!(
                "../tests/fixtures/bare_path_no_line_number.txt"
            )),
            vec![("src/main.rs".to_string(), None, None)]
        );
    }

    #[test]
    fn fixture_path_with_line_number() {
        assert_eq!(
            found(include_str!("../tests/fixtures/path_with_line_number.txt")),
            vec![("src/main.rs".to_string(), Some(42), None)]
        );
    }

    #[test]
    fn fixture_mixed_multiline() {
        assert_eq!(
            found(include_str!("../tests/fixtures/mixed_multiline.txt")),
            vec![
                ("src/main.rs".to_string(), None, None),
                ("Cargo.toml".to_string(), Some(15), None),
            ]
        );
    }

    #[test]
    fn fixture_no_path_prose() {
        assert!(found(include_str!("../tests/fixtures/no_path_prose.txt")).is_empty());
    }

    #[test]
    fn absolute_paths() {
        let abs = std::env::current_dir()
            .unwrap()
            .join("src/main.rs")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            found(&format!("{abs}:some content")),
            vec![(abs.clone(), None, None)]
        );
        assert_eq!(
            found(&format!("{abs}:42:some content")),
            vec![(abs, Some(42), None)]
        );
    }

    #[test]
    fn trailing_punctuation_is_trimmed() {
        assert_eq!(
            found("edit src/main.rs, then Cargo.toml."),
            vec![
                ("src/main.rs".to_string(), None, None),
                ("Cargo.toml".to_string(), None, None),
            ]
        );
    }

    #[test]
    fn line_and_col() {
        assert_eq!(
            found("src/main.rs:7:3:uh oh"),
            vec![("src/main.rs".to_string(), Some(7), Some(3))]
        );
    }

    #[test]
    fn span_offsets_cover_path_and_suffix() {
        let input = "see src/main.rs:42 now";
        let m = &extract_path_matches(input)[0];
        assert_eq!(&input[m.start..m.end], "src/main.rs:42");
    }

    #[test]
    fn tilde_expansion() {
        let home = std::env::var("HOME").unwrap();
        assert_eq!(expand_tilde("~/x").as_ref(), format!("{home}/x").as_str());
        assert_eq!(expand_tilde("/abs/x").as_ref(), "/abs/x");
    }

    #[test]
    fn made_up_paths_are_dropped() {
        assert!(found("read notreal.xyz or e.g. whatever").is_empty());
    }
}
