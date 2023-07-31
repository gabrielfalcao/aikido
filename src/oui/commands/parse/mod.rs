#![allow(unused)]
use crate::errors::Error;
use std::process::{Command, Output};
// use std::fs::OpenOptions;
// use std::io::Cursor;
// use std::path::Path;

pub enum Format {
    Yaml,
    Json,
}

pub fn default_output_file() -> String {
    "/dev/stdout".to_string()
}

pub fn command(format: Format) -> Result<Vec<String>, Error> {
    let format = match format {
        Format::Yaml => "--yaml",
        Format::Json => "--json",
    };
    match Command::new("oui").arg("parse").arg(format).output() {
        Ok(rs) => {
            if rs.status.success() {
                Ok(String::from_utf8(rs.stdout)?.split("\n").map(|x| x.to_string()).filter(|x| x.len() > 0).collect())
            } else {
                Err(Error::CommandExecutionErr((
                    match rs.status.code() {
                        Some(c) => c,
                        None => 0x54,
                    },
                    format!("failed to run `oui parse {}`: {}", format, String::from_utf8(rs.stderr)?),
                )))
            }
        }
        Err(or) => Err(Error::IOErr(or)),
    }
}
