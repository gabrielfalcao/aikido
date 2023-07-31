#![allow(unused)]
use crate::errors::Error;
use std::process::{Command, Output};
// use std::fs::OpenOptions;
// use std::io::Cursor;
// use std::path::Path;

pub enum Kind {
    Formula,
    Cask,
}

pub fn default_output_file() -> String {
    "/dev/stdout".to_string()
}

pub fn command(kind: Kind) -> Result<Vec<String>, Error> {
    let kind = match kind {
        Kind::Formula => "--formula",
        Kind::Cask => "--cask",
    };
    match Command::new("brew").arg("list").arg(kind).output() {
        Ok(rs) => {
            if rs.status.success() {
                Ok(String::from_utf8(rs.stdout)?.split("\n").map(|x| x.to_string()).filter(|x| x.len() > 0).collect())
            } else {
                Err(Error::CommandExecutionErr((
                    match rs.status.code() {
                        Some(c) => c,
                        None => 0x54,
                    },
                    format!("failed to run `brew list {}`: {}", kind, String::from_utf8(rs.stderr)?),
                )))
            }
        }
        Err(or) => Err(Error::IOErr(or)),
    }
}
