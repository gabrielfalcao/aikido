#![allow(unused)]
use aikido::brew::commands::list as brew_list;
use aikido::oui::commands::parse as oui_parse;
use aikido::pcap::commands::list as pcap_list;
use aikido::Error;
use clap::{Args, Parser, Subcommand};
use eui48::MacAddress;
use oui::OuiDatabase;
use shellexpand;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;


pub fn absolute_path(src: &str) -> String {
    String::from(shellexpand::tilde(src))
}

pub fn read_file(filename: &str) -> Vec<u8> {
    let mut reader = BufReader::new(File::open(filename).unwrap());
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).unwrap();
    bytes
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Climate {
    #[command(subcommand)]
    change: Change,
}

#[derive(Subcommand)]
enum Pcap {
    Parse(PcapList),
}

#[derive(Args)]
struct PcapList {
    #[arg(short, long)]
    output_file: Option<String>,
}

#[derive(Subcommand)]
enum Brew {
    Parse(BrewList),
}

#[derive(Args)]
struct BrewList {
    #[arg(short, long)]
    output_file: Option<String>,
}
#[derive(Subcommand)]
enum Oui {
    #[command(about = "Parse QRCode from image in the local filesystem")]
    Parse(OuiParse),
}

#[derive(Args)]
struct OuiParse {
    #[arg(short, long)]
    output_file: Option<String>,
    // #[arg(action = clap::ArgAction::Append)]
    #[arg(required(true))]
    input_file: String,
}

#[derive(Subcommand)]
enum Change {
    // #[command(subcommand)]
    // Brew(Brew),
    // #[command(subcommand)]
    // Pcap(Pcap),
    #[command(subcommand, about = "QRCode Operations")]
    Oui(Oui),
}

fn default_output_file() -> String {
    "/dev/stdout".to_string()
}

fn main() {
    let climate = Climate::parse();
    let command = match climate.change {
        Change::Oui(instruction) => match instruction {
            Oui::Parse(args) => {
                let db = OuiDatabase::new_from_file(absolute_path(&args.input_file)).unwrap();
                let dump = db.export().unwrap();
                println!("{:?}", dump);
            }
        },
    };
}
