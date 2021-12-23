extern crate clap;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use mac_notification_sys::*;
//use console::style;
use std::panic;
use toolz::{
    aes256cbc::{Config as AesConfig, Key},
    //    colors,
    config::YamlFile,
    logger,
    tomb::{app, AES256Tomb},
};
fn load_key(matches: &ArgMatches) -> Key {
    let tomb_keypath = matches.value_of("key_filename").unwrap();
    match Key::import(tomb_keypath) {
        Ok(key) => key,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn load_tomb(matches: &ArgMatches) -> AES256Tomb {
    let tomb_filepath = matches.value_of("tomb_filename").unwrap();
    match AES256Tomb::import(tomb_filepath) {
        Ok(tomb) => tomb,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn create_command(matches: &ArgMatches) {
    let tomb_filepath = matches.value_of("tomb_filename").unwrap();

    let key = load_key(matches);
    let config = AesConfig::default().unwrap();

    if AES256Tomb::import(tomb_filepath).is_ok() {
        logger::err::warning(format!("file already exists: {}", tomb_filepath));
        std::process::exit(0);
    }
    let tomb = AES256Tomb::new(key.clone(), config.clone());
    match tomb.export(tomb_filepath) {
        Ok(target) => {
            logger::out::ok(format!("saved file: {}", target));
        }
        Err(err) => {
            logger::err::error(format!("failed to save tomb file - {}", err));
            std::process::exit(1);
        }
    };
}

fn save_command(matches: &ArgMatches) {
    let tomb_filepath = matches.value_of("tomb_filename").unwrap();
    let path = matches.value_of("path").expect("missing key path");
    let value = matches.value_of("value").expect("missing value");
    let key = load_key(matches);
    let mut tomb = load_tomb(matches);
    match tomb.add_secret(path, String::from(value), key) {
        Ok(_) => {
            logger::out::ok(format!("added secret: {}", path));
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
    match tomb.export(tomb_filepath) {
        Ok(_) => {
            logger::out::ok(format!("created: {}", tomb_filepath));
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
}
fn get_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    let key = load_key(matches);
    let tomb = load_tomb(matches);
    match tomb.get_string(path, key) {
        Ok(plaintext) => {
            println!("{}", plaintext)
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn copy_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    let sound = matches.value_of("sound").unwrap_or("Glass");
    let key = load_key(matches);
    let tomb = load_tomb(matches);
    match tomb.get_string(path, key) {
        Ok(plaintext) => {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(plaintext).unwrap();
            eprintln!("{} secret copied to clipboard", path);
            send_notification(
                format!("{} secret", path).as_str(),
                &Some("copied to clipboard!"),
                "",
                &Some(sound),
            )
            .unwrap();
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn delete_command(matches: &ArgMatches) {
    let path = matches.value_of("path").expect("missing key path");
    //let key = load_key(matches);
    let mut tomb = load_tomb(matches);
    match tomb.delete_secret(path) {
        Ok(_) => {
            logger::out::ok(format!("deleted secret: {}", path));
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn list_command(matches: &ArgMatches) {
    let pattern = matches.value_of("pattern").expect("missing key pattern");
    // let key = load_key(matches);
    let tomb = load_tomb(matches);
    match tomb.list(pattern) {
        Ok(secrets) => {
            for entry in secrets {
                println!("{}", entry.path)
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
fn ui_command(matches: &ArgMatches) {
    let key = load_key(matches);
    let tomb = load_tomb(matches);
    let aes_config = AesConfig::default().unwrap_or(AesConfig::builtin(None));

    match app::start(tomb, key, aes_config) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

fn main() {
    panic::set_hook(Box::new(|e| {
        eprintln!("{}", e);
    }));

    let app = App::new("tomb")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Tomb file tree")
        .subcommand(
            SubCommand::with_name("save")
                .about("store a secret in the tomb")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("value")
                        .value_name("VALUE")
                        .required(true)
                        .help("the secret value to be saved")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("open the terminal ui")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("ui")
                .about("open the terminal ui")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("get a secret")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("copy")
                .about("copy a secret to the clipboard")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("sound")
                        .long("sound")
                        .help("name of sound to play (MacOS-only)")
                        .short("S")
                        .default_value("Glass")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("delete a secret")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("path")
                        .value_name("KEY PATH")
                        .help("the path to the secret")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("list secrets")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .help("the path to the aes256cbc key to encrypt the tomb secrets")
                        .short("k")
                        .default_value("~/.tomb.key")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tomb_filename")
                        .long("tomb")
                        .short("t")
                        .value_name("FILENAME")
                        .default_value("~/.tomb.yaml")
                        .help("the path to the tomb file containing the encrypted secrets")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("pattern")
                        .value_name("PATTERN")
                        .help("the path to the secret")
                        .default_value("*")
                        .takes_value(true),
                ),
        );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("create", Some(matches)) => {
            create_command(&matches);
        }
        ("save", Some(matches)) => {
            save_command(&matches);
        }
        ("get", Some(matches)) => {
            get_command(&matches);
        }
        ("copy", Some(matches)) => {
            copy_command(&matches);
        }
        ("delete", Some(matches)) => {
            delete_command(&matches);
        }
        ("list", Some(matches)) => {
            list_command(&matches);
        }
        ("ui", Some(matches)) => {
            ui_command(&matches);
        }
        (cmd, Some(_matches)) => {
            eprintln!("command not implemented: {}", cmd);
        }
        (cmd, None) => {
            eprintln!("unhandled command: {}", cmd);
        }
    }
}
