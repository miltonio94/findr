use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::{
    error::Error,
    fmt::format,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};
use walkdir::{DirEntry, WalkDir};

#[derive(Eq, PartialEq, Debug)]
enum EntryType {
    Dir,
    File,
    Link,
}

type MyResult<R> = Result<R, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("M")
        .about("Rust find")
        .arg(
            Arg::with_name("paths")
                .value_name("PATHS")
                .help("Input file")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .takes_value(true)
                .value_name("NAME")
                .multiple(true)
                .help("Output file"),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .multiple(true)
                .long("type")
                .takes_value(true)
                .value_name("TYPE")
                .multiple(true)
                .possible_value("f")
                .possible_value("d")
                .possible_value("l")
                .help("Output file"),
        )
        .get_matches();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        names: matches
            .values_of_lossy("name")
            .map(|vals| {
                vals.into_iter()
                    .map(|name| {
                        Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default(),

        entry_types: matches
            .values_of_lossy("type")
            .map(|vals| {
                vals.iter()
                    .map(|t| match t.as_str() {
                        "f" => File,
                        "d" => Dir,
                        "l" => Link,
                        _ => unreachable!("Can't convert"),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let filter_by_file_type = |dir_entry: &DirEntry| {
        config
            .entry_types
            .iter()
            .any(|entry_type| match entry_type {
                Link => dir_entry.file_type().is_symlink(),
                Dir => dir_entry.file_type().is_dir(),
                File => dir_entry.file_type().is_file(),
            })
            || config.entry_types.is_empty()
    };

    let filter_by_name = |dir_entry: &DirEntry| {
        config
            .names
            .iter()
            .any(|rgx| rgx.is_match(&dir_entry.file_name().to_string_lossy()))
            || config.names.is_empty()
    };

    for path in config.paths {
        let entries: Vec<String> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
                Ok(entry) => Some(entry),
            })
            .filter(filter_by_file_type)
            .filter(filter_by_name)
            .map(|entry| entry.path().display().to_string())
            .collect();
        println!("{}", entries.join("\n"));
    }
    Ok(())
}
