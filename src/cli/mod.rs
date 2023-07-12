#![allow(unused)]
use aikido::brew::list::{command as brew_list, Kind};
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
enum Brew {
    Freeze(BrewList),
}

#[derive(Args)]
struct BrewList {
    #[arg(short, long)]
    output_file: Option<String>,
}

#[derive(Subcommand)]
enum Change {
    #[command(subcommand)]
    Brew(Brew),
}

fn default_output_file() -> String {
    "/dev/stdout".to_string()
}

fn main() {
    let climate = Climate::parse();
    let command = match climate.change {
        Change::Brew(instruction) => {
            match instruction {
                Brew::Freeze(args) => {
                    let formulas = brew_list(Kind::Formula).unwrap();
                    // println!("{:?}", output_file);
                    println!("{:?}", formulas);
                }
            }
        }
    };
}
