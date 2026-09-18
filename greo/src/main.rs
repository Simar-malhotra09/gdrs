use core::fmt;
use std::process::Command;

/// Known bug: This way of impl probably treats the '10' in -m 10 when
/// passed a cmd line arg as a positional arg;
/// We should maybe start parsing from the right, take last as search path
/// second last as pattern, and everything else as std args.
///

struct SearchResult<'a> {
    file_path: &'a str,
    line_num: u32,
    contents: &'a str,
}

impl<'a> fmt::Display for SearchResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}\n",
            self.file_path, self.line_num, self.contents,
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
            file_path,
            line_num,
            contents,
        });
    }

    res
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    let command_args = &args[2..];

    let mut cmd = Command::new(command);
    cmd.args(command_args);

    println!("Command: {:?}", cmd);

    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let split_output = split_result(&stdout);
    let search_results: Vec<SearchResult> = parse_result(split_output.as_slice());
    for sr in search_results {
        println!("{}", sr);
    }

    Ok(())
}
