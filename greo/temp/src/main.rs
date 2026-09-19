use regex::Regex;
use std::borrow::Cow;
use std::path::Path;
use std::sync::LazyLock;

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
const TRAILING_PUNCT: &[char] = &['!', '.', ',', ';', ':', ')', ']', '}', '>', '"', '\''];

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
                caps.name("line_num")
                    .map(|l| l.as_str().parse::<u32>().unwrap()),
                caps.name("col_num")
                    .map(|c| c.as_str().parse::<u32>().unwrap()),
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

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn found(input: &str) -> Vec<(String, Option<u32>, Option<u32>)> {
        extract_path_matches(input)
            .into_iter()
            .map(|m| (m.path, m.line_num, m.col_num))
            .collect()
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
}
