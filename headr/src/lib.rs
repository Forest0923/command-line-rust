use crate::TakeValue::*;
use clap::{value_parser, Arg, Command};
use regex::Regex;
use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<usize>,
}

#[derive(Debug, PartialEq)]
enum TakeValue {
    MinusZero,
    TakeNum(i64),
}

fn count_lines(filename: &str) -> MyResult<(i64, Vec<u8>)> {
    let mut file: Box<dyn BufRead> = if filename == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(filename)?))
    };
    let mut num_lines = 0;
    let mut buf = Vec::new();
    let mut stdin_buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        if filename == "-" {
            stdin_buf.extend_from_slice(&buf);
        }
        buf.clear();
    }
    Ok((num_lines, stdin_buf))
}

fn get_end_line(line_num: &TakeValue, total_lines: i64) -> Option<u64> {
    match line_num {
        MinusZero => Some(total_lines as u64),
        TakeNum(n) => {
            if *n >= 0 {
                Some(min(*n as u64, total_lines as u64))
            } else {
                if n + total_lines >= 0 {
                    Some((n + total_lines).try_into().unwrap())
                } else {
                    Some(0)
                }
            }
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    eprintln!("{:#?}", config);
    let max_files_num = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        dbg!(file_num, filename, max_files_num);
        match open(&filename) {
            Err(e) => eprintln!("Failed to open {}: {}", filename, e),
            Ok(file) => {
                // multi files
                if max_files_num > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }

                // print contents
                if let Some(num_bytes) = config.bytes {
                    let mut handle = file.take(num_bytes as u64);
                    let mut buffer = vec![0; num_bytes];
                    let bytes_read = handle.read(&mut buffer)?;
                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
                } else {
                    let (total_lines, stdin_buf) = count_lines(filename)?;
                    let stdin_buf = String::from_utf8_lossy(&stdin_buf);
                    let _ = print_lines(
                        file,
                        &config.lines,
                        filename.to_string(),
                        total_lines,
                        stdin_buf.to_string(),
                    );
                }
            }
        }
    }
    Ok(())
}

fn print_lines(
    file: Box<dyn BufRead>,
    lines: &TakeValue,
    filename: String,
    total_lines: i64,
    stdin_buf: String,
) -> MyResult<()> {
    let mut file_reader = BufReader::new(file);
    if let Some(end) = get_end_line(&lines, total_lines) {
        let mut line = String::new();
        if filename == "-" {
            for (current_line, line) in stdin_buf.split('\n').enumerate() {
                if current_line < end.try_into().unwrap() {
                    println!("{}", line);
                }
            }
        } else {
            for _ in 0..end {
                let bytes = file_reader.read_line(&mut line)?;
                if bytes == 0 {
                    break;
                }

                print!("{line}");
                line.clear();
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .author("Masahiro Mori")
        .about("Rusty head")
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .action(clap::ArgAction::Append)
                .help("Input file(s)")
                .default_value("-"),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("LINES")
                .help("Print the first NUM lines instead of the first 10")
                .allow_negative_numbers(true)
                .default_value("10"),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Print the first NUM bytes of each file")
                .value_parser(value_parser!(usize)),
        )
        .get_matches();
    let lines = matches
        .get_one("lines")
        .cloned()
        .map(|l: String| parse_num(l.as_str()))
        .transpose()
        .map_err(|e| {
            format!("invalid value '{e}' for '--lines <LINES>': invalid digit found in string")
        })?
        .unwrap();
    dbg!(&lines);
    let bytes = matches.get_one::<usize>("bytes").copied();

    Ok(Config {
        files: matches
            .get_many::<String>("file")
            .unwrap()
            .map(|v| v.to_string())
            .collect::<Vec<_>>(),
        lines, //: parse_num("10")?,
        bytes,
    })
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let num_re = Regex::new(r"^(-)?(\d+)$").unwrap();
    dbg!(num_re.captures(val));
    match num_re.captures(val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("+", |m| m.as_str());
            let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
            if let Ok(val) = num.parse() {
                if sign == "-" && val == 0 {
                    Ok(MinusZero)
                } else {
                    Ok(TakeNum(val))
                }
            } else {
                Err(From::from(val))
            }
        }
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_num() {
    let ret = parse_num("1");
    assert_eq!(ret.unwrap(), TakeNum(1));
    let ret = parse_num("0");
    assert_eq!(ret.unwrap(), TakeNum(0));
    let ret = parse_num("-0");
    assert_eq!(ret.unwrap(), MinusZero);
    let ret = parse_num("-1");
    assert_eq!(ret.unwrap(), TakeNum(-1));
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
