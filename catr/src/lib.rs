use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(e) => eprintln!("Faild to open {}: {}", filename, e),
            Ok(file) => {
                let mut line_num = 1;
                for lines_iter in file.lines() {
                    let line = lines_iter?;
                    if config.number_lines {
                        println!("{:>6}\t{}", line_num, line);
                        line_num += 1;
                    } else if config.number_nonblank_lines {
                        if line.is_empty() {
                            println!("");
                        } else {
                            println!("{:>6}\t{}", line_num, line);
                            line_num += 1;
                        }
                    } else {
                        println!("{}", line);
                        line_num += 1;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Masahiro Mor")
        .about("Rusty cat")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .multiple(true)
                .help("Input file(s)")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number_lines")
                .short("n")
                .long("number")
                .help("Number all output lines")
                .takes_value(false)
                .conflicts_with("number_nonblank_lines"),
        )
        .arg(
            Arg::with_name("number_nonblank_lines")
                .short("b")
                .long("number-nonblank")
                .help("Number nonempty output lines")
                .takes_value(false)
                .conflicts_with("number_lines"),
        )
        .get_matches();
    Ok(Config {
        files: matches.values_of_lossy("file").unwrap(),
        number_lines: matches.is_present("number_lines"),
        number_nonblank_lines: matches.is_present("number_nonblank_lines"),
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
