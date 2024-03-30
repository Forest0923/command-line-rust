use clap::{Arg, ArgAction, Command};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("uniqr")
        .version("0.1.0")
        .author("Masahiro Mori")
        .about("`uniq` command written in Rust")
        .arg(
            Arg::new("input")
                .value_name("INPUT")
                .help("Input file or stdin")
                .action(ArgAction::Set)
                .default_value("-"),
        )
        .arg(
            Arg::new("output")
                .value_name("OUTPUT")
                .help("Output file or stdout")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("COUNT")
                .help("Prefix lines by the number of occurrences")
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    Ok(Config {
        in_file: matches.get_one("input").cloned().unwrap(),
        out_file: matches.get_one("output").cloned(),
        count: matches.get_flag("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // dbg!(&config);
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut pre_line = String::new();
    let mut count: u64 = 0;
    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(f) => Box::new(File::create(f)?),
        None => Box::new(io::stdout()),
    };

    let mut print = |count: u64, text: &str| {
        let out = if config.count {
            format!("{:>4} {}", count, text)
        } else {
            format!("{}", text)
        };
        let _ = out_file.write_all(out.as_bytes());
    };

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            if !pre_line.is_empty() {
                print(count, &pre_line.as_str());
            }
            break;
        }
        // dbg!(&line, &pre_line);
        if line.trim_end() == pre_line.trim_end() {
            count += 1;
        } else {
            if count != 0 {
                print(count, &pre_line.as_str());
            }
            count = 1;
            pre_line = line.clone();
        }
        line.clear();
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
