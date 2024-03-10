use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: isize,
    is_negative_lines: bool,
    bytes: Option<usize>,
}

fn count_lines(filename: &str) -> MyResult<isize> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        buf.clear();
    }
    Ok(num_lines)
}

pub fn run(config: Config) -> MyResult<()> {
    let max_files_num = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        println!("{}", filename);
        match open(&filename) {
            Err(e) => eprintln!("Failed to open {}: {}", filename, e),
            Ok(mut file) => {
                if max_files_num > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                if let Some(num_bytes) = config.bytes {
                    let mut handle = file.take(num_bytes as u64);
                    let mut buffer = vec![0; num_bytes];
                    let bytes_read = handle.read(&mut buffer)?;
                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
                } else {
                    let max_lines = count_lines(&filename)?;
                    let mut line = String::new();
                    let end = if config.is_negative_lines {
                        if max_lines > config.lines {
                            max_lines - config.lines
                        } else {
                            0
                        }
                    } else {
                        config.lines
                    };
                    for _ in 0..end {
                        let bytes = file.read_line(&mut line)?;
                        if bytes == 0 {
                            break;
                        }
                        print!("{}", line);
                        line.clear();
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("Masahiro Mori")
        .about("Rusty head")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .multiple(true)
                .help("Input file(s)")
                .allow_hyphen_values(false)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Print the first NUM lines instead of the first 10")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Print the first NUM bytes of each file"),
        )
        .get_matches();
    let lines = matches
        .value_of("lines")
        .map(parse_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;
    let is_negative_lines = matches
        .value_of("lines")
        .map_or(false, |s| s.starts_with('-'));
    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("file").unwrap(),
        lines: lines.unwrap(),
        is_negative_lines,
        bytes,
    })
}

fn parse_int(val: &str) -> MyResult<isize> {
    let num_re = Regex::new(r"^([+-])?(\d+)$").unwrap();
    match num_re.captures(val) {
        Some(caps) => Ok(caps.get(2).unwrap().as_str().parse()?),
        _ => Err(From::from(val)),
    }
    // let num = match val.strip_prefix("-") {
    //     Some(num) => num,
    //     _ => return Err(From::from(val)),
    // };
    // match num.parse::<isize>() {
    //     Ok(n) => Ok(n),
    //     _ => Err(From::from(val)),
    // }
}

#[test]
fn test_parse_int() {
    let res = parse_int("-1");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 1);
    let res = parse_int("0");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 0);
    let res = parse_int("-0");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 0);
    let res = parse_int("1");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 1);
    let res = parse_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3は正の整数なのでOK
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // 数字でない文字列の場合はエラー
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // 0の場合もエラー
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
