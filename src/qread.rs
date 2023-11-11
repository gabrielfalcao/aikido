use image;
use rqrr;
use clap::{Parser};
use crate::errors::Error;



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    path: String
}


fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let img = image::open(cli.path)?.to_luma();
    let mut img = rqrr::PreparedImage::prepare(img);
    let grids = img.detect_grids();
    let (meta, content) = grids[0].decode()?;
    println!("{}", meta);
    println!("{}", content);
}
