extern crate clap;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use console::style;
use std::collections::BTreeMap;

use std::fs;
use std::path::Path;
use toolz::ioutils::{
    delete_directory, delete_file, get_cwd, read_file, write_map_to_yaml, Result, UserFriendlyError,
};
use walkdir::WalkDir;

fn scan(target_dir: &str) -> Result<BTreeMap<String, String>> {
    let current_dir = get_cwd();

    let mut result = BTreeMap::new();
    let parent = match Path::new(&current_dir).canonicalize() {
        Ok(path) => format!("{}", path.as_os_str().to_str().unwrap()),
        Err(e) => {
            return serde::__private::Err(UserFriendlyError {
                message: format!("cannot get absolute path for '{}': {}", target_dir, e),
            })
        }
    };
    for entry in WalkDir::new(target_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let file_name = match entry.file_name() {
            p => format!("{}", p.to_str().unwrap()),
        };
        if !entry.path().is_file()
            || entry.path().ends_with(".0b4sk8d")
            || file_name == "0b4sk8d.yaml"
        {
            continue;
        }
        let parent = parent.clone();
        let path = entry.path().canonicalize().unwrap();
        let full_path = format!("{}", path.as_os_str().to_str().unwrap());
        let filename = full_path.replace(&parent, ".");

        let digest = md5::compute(full_path.as_bytes());
        let hex = format!("{:x}", digest);
        let prefix0 = &hex[..1];
        let prefix1 = &hex[..2];
        let prefix2 = &hex[..4];
        let prefix3 = &hex[..6];
        let obfuskat3d = format!(
            "0b4sk8d/{}/{}/{}/{}/{}.0b4sk8d",
            prefix0, prefix1, prefix2, prefix3, hex
        );
        result.insert(obfuskat3d, filename);
    }
    Ok(result)
}

fn obfuskat3_command(matches: &ArgMatches) {
    let current_dir = get_cwd();

    let mut result: BTreeMap<String, String> = if Path::new("0b4sk8d.yaml").exists() {
        let yaml = read_file("0b4sk8d.yaml");
        serde_yaml::from_str(&yaml).expect("failed to parse yaml from 0b4sk8d.yaml")
    } else {
        BTreeMap::new()
    };
    let target = matches.value_of("target").unwrap_or(".");

    let new = scan(target).unwrap();
    result.extend(new);

    if write_map_to_yaml(&result, "0b4sk8d.yaml") {
        println!("index written to 0b4sk8d.yaml");
    }

    for (obfuskat3d, filename) in result.iter() {
        if Path::new(obfuskat3d).exists() {
            eprintln!(
                "{} {}: {}",
                style("skipping").color256(247),
                style(obfuskat3d).color256(178),
                style("already exists").color256(247)
            );
            continue;
        }
        let filepath = Path::new(&obfuskat3d);
        let parent = match filepath.parent() {
            Some(p) => p,
            None => Path::new(&current_dir),
        };
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap_or(());
        }

        match fs::rename(filename, obfuskat3d) {
            Ok(_) => {
                println!(
                    "{}{}{}",
                    style("obfuskat3d ").color256(241),
                    style(filename).color256(246),
                    style("...").color256(241),
                )
            }
            Err(error) => {
                eprintln!(
                    "{}{}{}{}{}",
                    style("Error renaming ").color256(198),
                    style(filename).color256(190),
                    style(" to ").color256(198),
                    style(obfuskat3d).color256(159),
                    style(format!("\n\t{}", error)).color256(197),
                );
            }
        };
    }
}

fn unobfuskat3_command(matches: &ArgMatches) {
    let target = matches.value_of("target").unwrap();
    let current_dir = get_cwd();

    if !Path::new(target).exists() {
        eprintln!(
            "{} {}",
            style(target).color256(122),
            style("does not exist!").color256(197)
        );
        std::process::exit(1);
    }
    let yaml = read_file(target);
    let index: BTreeMap<String, String> =
        serde_yaml::from_str(&yaml).expect("failed to parse yaml");

    for (obfuskat3d, filename) in index.iter() {
        if Path::new(filename).exists() {
            eprintln!(
                "{} {}: {}",
                style("skipping").color256(247),
                style(filename).color256(178),
                style("already unobfuskat3d").color256(247)
            );
            continue;
        }
        let filepath = Path::new(&filename);
        let parent = match filepath.parent() {
            Some(p) => p,
            None => Path::new(&current_dir),
        };
        if !parent.exists() {
            println!(
                "{}{}",
                style("unobfuskat3d ").color256(241),
                style(format!("{:?}", parent)).color256(246),
            );
            fs::create_dir_all(parent).unwrap_or(());
        }
        match fs::rename(obfuskat3d, filename) {
            Ok(_) => {
                println!(
                    "{}{}{}",
                    style("unobfuskat3d ").color256(241),
                    style(filename).color256(246),
                    style("...").color256(241),
                )
            }
            Err(error) => {
                eprintln!(
                    "{}{}{}{}{}",
                    style("Error renaming ").color256(198),
                    style(filename).color256(190),
                    style(" to ").color256(198),
                    style(obfuskat3d).color256(159),
                    style(format!("\n\t{}", error)).color256(197),
                );
            }
        };
    }
    delete_file(target);
    delete_directory("0b4sk8d");
}

fn main() {
    let app = App::new("obfuskat3")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Obfuskat3 file tree")
        .subcommand(
            SubCommand::with_name("from")
                .about("the target path whoe file tree will be obfuskat3d")
                .arg(
                    Arg::with_name("target")
                        .help("root path")
                        .default_value(".")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("undo")
                .about("unobfuskat3s the file tree under the given 0b4sk8d.yaml")
                .arg(
                    Arg::with_name("target")
                        .value_name("[path/to/0b4sk8d.yaml]")
                        .help("full path to an existing 0b4sk8d.yaml")
                        .default_value("./0b4sk8d.yaml")
                        .takes_value(true),
                ),
        );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("from", Some(matches)) => {
            obfuskat3_command(&matches);
        }
        ("undo", Some(matches)) => unobfuskat3_command(&matches),
        (cmd, Some(_matches)) => {
            eprintln!("command not implemented: {}", cmd);
        }
        (cmd, None) => {
            eprintln!("unhandled command: {}", cmd);
        }
    }
}
