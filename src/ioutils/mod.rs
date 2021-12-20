use console::style;

use crate::colors;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::{env, fmt, fs};

pub type Result<T> = std::result::Result<T, UserFriendlyError>;

#[derive(Debug, Clone)]
pub struct UserFriendlyError {
    pub message: String,
}

impl fmt::Display for UserFriendlyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
pub fn read_file(filename: &str) -> String {
    let mut file = File::open(filename).expect(format!("failed to open {}", filename).as_str());
    let mut text = String::new();
    file.read_to_string(&mut text)
        .expect("failed to parse text");
    text
}

pub fn delete_file(target: &str) -> bool {
    match fs::remove_file(target) {
        Ok(_) => {
            println!(
                "{}{}",
                style("deleted index ").color256(241),
                style(target).color256(246),
            );
            true
        }
        Err(error) => {
            eprintln!(
                "{}{}{}",
                style("Error deleting ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            );
            false
        }
    }
}
pub fn delete_directory(target: &str) -> bool {
    match fs::remove_dir_all(target) {
        Ok(_) => {
            println!(
                "{}{}",
                style("deleted empty directory: ").color256(241),
                style(target).color256(246),
            );
            true
        }
        Err(error) => {
            eprintln!(
                "{}{}{}",
                style("Error deleting ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            );
            false
        }
    }
}
pub fn open_write(target: &str) -> Option<std::fs::File> {
    match OpenOptions::new().create(true).write(true).open(target) {
        Ok(file) => Some(file),
        Err(error) => {
            eprintln!(
                "{}{}{}{}",
                style("failed to open ").color256(colors::ERR_MSG),
                style(target).color256(colors::ERR_VAR),
                style("in write mode").color256(colors::ERR_MSG),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            );
            None
        }
    }
}

pub fn write_map_to_yaml(map: &BTreeMap<String, String>, filename: &str) -> bool {
    let mut file = open_write(filename).unwrap();
    let yaml = match serde_yaml::to_string(map) {
        Ok(s) => s,
        Err(error) => {
            eprintln!(
                "{}{}{}",
                style("failed to serialze data to yaml ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            );
            return false;
        }
    };
    match file.write_all(yaml.as_bytes()) {
        Ok(_) => {
            return true;
        }
        Err(error) => {
            eprintln!(
                "{}{}{}",
                style("failed to write yaml data to: ").color256(colors::ERR_MSG),
                style(filename).color256(colors::ERR_VAR),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            );
            return false;
        }
    }
}

pub fn get_cwd() -> String {
    match env::current_dir() {
        Ok(pbuf) => match pbuf.as_path().canonicalize() {
            Ok(current_dir) => match current_dir.as_os_str().to_str() {
                Some(path) => String::from(path),
                None => {
                    eprintln!(
                        "{}",
                        style("failed convert cwd path to string").color256(colors::ERR_HLT),
                    );
                    std::process::exit(1);
                }
            },
            Err(error) => {
                eprintln!(
                    "{}{}",
                    style("failed to calculate absolute path of current working directory")
                        .color256(colors::ERR_MSG),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                );
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!(
                "{}{}",
                style("failed to retrieve current working directory").color256(colors::ERR_HLT),
                style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
            );
            std::process::exit(1);
        }
    }
}
