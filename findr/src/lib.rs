use crate::EntryType::*;
use clap::{value_parser, Arg, ArgAction, Command};
use regex::Regex;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("findr")
        .version("0.1.0")
        .author("Masahiro Mori")
        .about("`find` command written in Rust")
        .arg(
            Arg::new("paths")
                .value_name("PATH")
                .help("Search paths")
                .default_value(".")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("names")
                .value_name("NAME")
                .short('n')
                .long("name")
                .help("Name")
                .action(ArgAction::Append)
                .value_parser(Regex::new),
        )
        .arg(
            Arg::new("types")
                .value_name("TYPE")
                .short('t')
                .long("type")
                .help("Entry type")
                .action(ArgAction::Append)
                .value_parser(["f", "d", "l"]),
        )
        .get_matches();

    Ok(Config {
        paths: matches.get_many("paths").unwrap().cloned().collect(),
        names: match matches.get_many("names") {
            Some(n) => n.cloned().collect(),
            None => vec![],
        },
        entry_types: match matches.get_many::<String>("types") {
            Some(t) => t
                .map(|val| match val.as_str() {
                    "f" => EntryType::File,
                    "d" => EntryType::Dir,
                    "l" => EntryType::Link,
                    _ => unreachable!(),
                })
                .collect(),
            None => vec![],
        },
    })
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}
