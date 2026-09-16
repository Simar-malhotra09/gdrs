use std::process::Command;

#[derive(Debug)]
struct SearchResult<'a> {
    file_path: &'a str,
    line_num: u32,
    contents: &'a str,
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
    let search_results: Vec<SearchResult> = parse_result(split_output.as_slice());
    for sr in search_results {
        println!("{:?}", sr);
    }

    Ok(())
}
