#![allow(unused)]
use aikido::pcap::list::{command as pcap_list, Kind};
use aikido::Error;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Climate {
    #[command(subcommand)]
    change: Change,
}

#[derive(Subcommand)]
enum Pcap {
    Freeze(PcapList),
}

#[derive(Args)]
struct PcapList {
    #[arg(short, long)]
    output_file: Option<String>,
}

#[derive(Subcommand)]
enum Change {
    #[command(subcommand)]
    Pcap(Pcap),
}

fn default_output_file() -> String {
    "/dev/stdout".to_string()
}

fn main() {
    let climate = Climate::parse();
    let command = match climate.change {
        Change::Pcap(instruction) => {
            match instruction {
                Pcap::Freeze(args) => {

                }
            }
        }
    };
}
