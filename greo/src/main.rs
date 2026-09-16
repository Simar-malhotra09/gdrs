use core::fmt;
use std::{fmt::Display, process::Command};

/// Known bug: This way of impl probably treats the '10' in -m 10 when
/// passed a cmd line arg as a positional arg;
/// We should maybe start parsing from the right, take last as search path
/// second last as pattern, and everything else as std args.
///
struct SearchResult<'a> {
    file_path: &'a str,
    line_num: u32,
    contents_before_match: &'a str,
    contents_match: &'a str,
    contents_after_match: &'a str,
}

impl<'a> fmt::Display for SearchResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent_width = self
            .contents_before_match
            .chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| if c == '\t' { 2 } else { 1 })
            .sum::<usize>();
        let contents_before_match_s = self.contents_before_match.trim_start_matches([' ', '\t']);
        write!(
            f,
            "{}:\x1b[93m{}\x1b[0m:{}{}\x1b[91m{}\x1b[0m{}\n",
            self.file_path,
            self.line_num,
            "\u{2218}".repeat(indent_width),
            contents_before_match_s,
            self.contents_match,
            self.contents_after_match,
        )
    }
}

struct InputArgs {
    args: Vec<String>,
}

impl Default for InputArgs {
    fn default() -> Self {
        Self { args: Vec::new() }
    }
}

struct GrepInputArgs<'a> {
    path: &'a str,
    pattern: &'a str,
    named_args: Vec<&'a str>,
}

impl<'a> From<&'a InputArgs> for GrepInputArgs<'a> {
    fn from(args: &'a InputArgs) -> Self {
        Self {
            path: &args.args[1],
            pattern: &args.args[2],
            named_args: vec!["-rni", "-m", "10"],
        }
    }
}
impl<'a> Default for GrepInputArgs<'a> {
    fn default() -> Self {
        Self {
            path: "/Users/0saker/Desktop/code/probe/fdrs/src/",
            pattern: "run",
            named_args: vec!["-rni", "-m", "10"],
        }
    }
}
impl<'a> GrepInputArgs<'a> {
    fn into_parts(self) -> (&'a str, &'a str, Vec<&'a str>) {
        (self.path, self.pattern, self.named_args)
    }
}

fn split_result(output: &str) -> Vec<&str> {
    output.lines().collect()
}

fn parse_result<'a>(pattern: &'a str, split_output: &[&'a str]) -> Vec<SearchResult<'a>> {
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

        let Some((contents_before_match, contents_after_match)) = contents.split_once(pattern)
        else {
            continue;
        };

        res.push(SearchResult {
            file_path,
            line_num,
            contents_before_match,
            contents_match: pattern,
            contents_after_match,
        });
    }

    res
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let input = InputArgs { args };

    let (path, pattern, named_args) = if input.args.len() < 3 {
        GrepInputArgs::default().into_parts()
    } else {
        GrepInputArgs::from(&input).into_parts()
    };

    let output = Command::new("grep")
        .args(named_args)
        .arg(pattern)
        .arg(path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let split_output = split_result(&stdout);
    let search_results: Vec<SearchResult> = parse_result(pattern, split_output.as_slice());
    for sr in search_results {
        println!("{}", sr);
    }

    Ok(())
}
