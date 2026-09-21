use regex::Regex;
use std::borrow::Cow;
use std::fmt;
// use std::io::{self, IsTerminal, Read};
use std::path::Path;
// use std::process::Command;
use ratatui::style::Color;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use std::sync::LazyLock;

const TEXT_FG_COLOR: Color = SLATE.c200;

#[allow(dead_code)]
#[derive(Default)]
pub struct Packed {
    pub content: String,
    pub matches: Vec<PathMatch>,
}

impl Packed {
    pub fn new(content: String) -> Self {
        let matches = extract_path_matches(&content);
        Self { content, matches }
    }
}

impl fmt::Display for Packed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // writeln!(f, "Content:\n{}", self.content)?;
        writeln!(f, "Matches:")?;

        for m in &self.matches {
            writeln!(f, "  {m}")?;
        }

        Ok(())
    }
}
#[derive(Default)]
pub struct Output {
    pub o_stdin: Packed,
    pub o_stdout: Packed,
    pub o_stderr: Packed,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PathMatch {
    pub path: String,
    pub line_num: Option<u32>,
    pub col_num: Option<u32>,
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for PathMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.path,
            self.line_num.map_or("-".to_string(), |n| n.to_string()),
            self.col_num.map_or("-".to_string(), |n| n.to_string()),
        )
    }
}

#[allow(dead_code)]
pub const TRAILING_PUNCT: &[char] = &['.', ',', ';', ':', ')', ']', '}', '>', '"', '\'', '!'];

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
pub fn extract_path_matches(input: &str) -> Vec<PathMatch> {
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

impl From<&PathMatch> for ListItem<'_> {
    fn from(value: &PathMatch) -> Self {
        let path = value.path.clone();
        let line_num = if value.line_num.is_some() {
            value.line_num.unwrap().to_string()
        } else {
            "-".to_string()
        };
        let col_num = if value.col_num.is_some() {
            value.col_num.unwrap().to_string()
        } else {
            "-".to_string()
        };

        let line = Line::styled(
            format!("Path: {}, Line: {}, Col: {}", path, line_num, col_num,),
            TEXT_FG_COLOR,
        );
        ListItem::new(line)
    }
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut i_stdin = String::new();
//     let mut i_stdout = String::new();
//     let mut i_stderr = String::new();
//     // let mut cmd_output = Output {
//     //     stdout: String::new(),
//     //     stderr: String::new(),
//     // };
//     if !io::stdin().is_terminal() {
//         io::stdin().read_to_string(&mut i_stdin).unwrap();
//     } else {
//         let args: Vec<String> = std::env::args().skip(1).collect();
//         let Some((command, command_args)) = args.split_first() else {
//             eprintln!("usage: greo [--] <command> [args...]");
//             std::process::exit(2);
//         };
//         let mut cmd = Command::new(command);
//         cmd.args(command_args);
//
//         println!("Command: {} {}", command, command_args.join(" "));
//
//         let output = match cmd.output() {
//             Ok(output) => output,
//             Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
//                 eprintln!("greo: command not found: {command}");
//                 std::process::exit(127);
//             }
//             Err(e) => return Err(e.into()),
//         };
//
//         i_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
//         i_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
//         // cmd_output.stdout = stdout;
//         // cmd_output.stderr = stderr;
//     }
//     let output = Output {
//         o_stdin: Packed {
//             matches: extract_path_matches(&i_stdin),
//             content: i_stdin,
//         },
//         o_stdout: Packed {
//             matches: extract_path_matches(&i_stdout),
//             content: i_stdout,
//         },
//         o_stderr: Packed {
//             matches: extract_path_matches(&i_stderr),
//             content: i_stderr,
//         },
//     };
//     println!("STDIN\n{}", output.o_stdin);
//     println!("STDOUT\n{}", output.o_stdout);
//     println!("STDERR\n{}", output.o_stderr);
//
//     Ok(())
// }

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
