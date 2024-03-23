use clap::{Arg, ArgAction, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("wcr")
        .version("0.1.0")
        .author("Masahiro Mori")
        .about("Rusty word count")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .action(ArgAction::Append)
                .help("Input files(s)")
                .default_value("-"),
        )
        .arg(
            Arg::new("lines")
                .short('l')
                .long("lines")
                .help("Print the newline counts")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("words")
                .short('w')
                .long("words")
                .help("Print the word counts")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .help("Print the byte counts")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("chars")
                .short('m')
                .long("chars")
                .help("Print the character counts")
                .action(ArgAction::SetTrue)
                .conflicts_with("bytes"),
        )
        .get_matches();
    let mut lines = matches.get_flag("lines");
    let mut words = matches.get_flag("words");
    let mut bytes = matches.get_flag("bytes");
    let chars = matches.get_flag("chars");
    if !lines && !words && !bytes && !chars {
        lines = true;
        words = true;
        bytes = true;
    }
    Ok(Config {
        files: matches.get_many("files").unwrap().cloned().collect(),
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(&config);
    for filename in &config.files {
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                if let Ok(info) = count(file) {
                    println!("{:?}", info);
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// why pub?
pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();
    loop {
        let bytes_read = file.read_line(&mut line)?;
        dbg!(line.split_whitespace().count());
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_words += line.split_whitespace().count();
        num_bytes += bytes_read;
        num_chars += line.chars().count();
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count_empty() {
        let text = "";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 0,
            num_words: 0,
            num_bytes: 0,
            num_chars: 0,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_count_one() {
        let text = "hello \tworld\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 2,
            num_bytes: 14,
            num_chars: 14,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_count_strange_f() {
        let text = "ﬀ\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 1,
            num_bytes: 5,
            num_chars: 3,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
