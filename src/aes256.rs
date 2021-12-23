use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use console::style;
use std::io::Write;
use std::{fs::File, panic};
use toolz::ioutils::{b64encode, create_file};

use toolz::aes256cbc::Config;
use toolz::aes256cbc::Key;
use toolz::config::YamlFile;
use toolz::{colors, core, ioutils::read_bytes, logger};

pub fn confirm_password() -> Option<String> {
    let password = rpassword::prompt_password_stderr("Password: ").unwrap();
    let confirmation = rpassword::prompt_password_stderr("Confirm password: ").unwrap();

    if password != confirmation {
        logger::err::error(format!(
            "{}",
            style("Password/Confirmation mismatch").color256(202)
        ));
        None
    } else {
        Some(password)
    }
}
fn get_password_from_matches(matches: &ArgMatches) -> String {
    let ask_password = matches.is_present("ask_password");
    let password = if ask_password {
        match confirm_password() {
            Some(password) => password,
            None => String::from(matches.value_of("password").unwrap_or("")),
        }
    } else {
        String::from(matches.value_of("password").unwrap_or(""))
    };
    password
}

fn load_key(matches: &ArgMatches, config: &Config) -> Key {
    let password = get_password_from_matches(matches);
    let key_filename = matches.value_of("key_filename").unwrap_or("");

    if key_filename.len() > 0 {
        Key::import(key_filename).unwrap()
    } else if password.len() > 0 {
        Key::from_password(&password.as_bytes(), config)
    } else {
        logger::err::error(format!(
            "{}{}{}{}{}",
            style("either").color256(195),
            style("--password, --key-filename").color256(190),
            style(" or ").color256(195),
            style("--ask-password").color256(190),
            style(" is required").color256(195),
        ));
        std::process::exit(1);
    }
}

fn generate_command(matches: &ArgMatches) {
    let ask_password = matches.is_present("ask_password");
    let key_cycles = matches
        .value_of("key_cycles")
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(1000);
    let salt_cycles = matches
        .value_of("salt_cycles")
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(1000);
    let iv_cycles = matches
        .value_of("iv_cycles")
        .unwrap_or("")
        .parse::<u32>()
        .unwrap_or(1000);

    // test
    // C-x C-s to save, just like in emacs
    let vec: [u32; 3] = [key_cycles, salt_cycles, iv_cycles];
    let custom_config = Config::from_vec(&vec);
    let password = if ask_password {
        match confirm_password() {
            Some(password) => password,
            None => String::from(matches.value_of("password").unwrap_or("")),
        }
    } else {
        String::from(matches.value_of("password").unwrap_or(""))
    };
    let key = Key::from_password(&password.as_bytes(), &custom_config);

    let filename = matches.value_of("key_filename").unwrap();
    //let key_yaml = key.to_yaml();
    let key_path = match key.export(filename) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };
    logger::err::ok(format!("generated key: {}", style(key_path).color256(214)));
}
fn encrypt_command(matches: &ArgMatches, config: &Config) {
    let key = load_key(matches, config);
    let cyphertext_filename = matches.value_of("cyphertext_filename").unwrap();
    let plaintext_string = matches.value_of("string").unwrap_or("");
    let plaintext_filename = matches.value_of("plaintext_filename").unwrap_or("");

    let fail_if_already_encrypted = !matches.is_present("try");

    match key.owns_file(plaintext_filename) {
        Ok(owns_file) => {
            if owns_file {
                std::process::exit(match fail_if_already_encrypted {
                    true => 1,
                    false => 0,
                });
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let plaintext = if plaintext_filename.len() > 0 {
        read_bytes(plaintext_filename).unwrap()
    } else if plaintext_string.len() > 0 {
        plaintext_string.as_bytes().to_vec()
    } else {
        logger::err::error(format!(
            "{}{}{}{}{}",
            style("either").color256(195),
            style("--string").color256(190),
            style(" or ").color256(195),
            style("--input-filename").color256(190),
            style(" is required").color256(195),
        ));
        std::process::exit(1);
    };

    let cyphertext = match key.encrypt(&plaintext) {
        Ok(cypher) => cypher,
        Err(error) => {
            logger::err::error(format!(
                "{}{}{}",
                style("cannot encrypt contents of file ").color256(colors::ERR_MSG),
                style(plaintext_filename).color256(colors::ERR_VAR),
                style(format!("\n\t{:?}", error)).color256(colors::ERR_HLT),
            ));
            std::process::exit(1);
        }
    };
    let mut file = match create_file(cyphertext_filename) {
        Ok(file) => file,
        Err(error) => {
            logger::err::error(format!(
                "cannot write encrypted data to: {}\n{}",
                style(cyphertext_filename).color256(207),
                error
            ));
            std::process::exit(1);
        }
    };
    match file.write(&cyphertext) {
        Ok(_) => {
            logger::err::ok(format!(
                "wrote encrypted data in: {}",
                style(cyphertext_filename).color256(207)
            ));
        }
        Err(error) => {
            logger::err::error(format!(
                "cannot write encrypted data to: {}\n{}",
                style(cyphertext_filename).color256(207),
                error
            ));
            std::process::exit(1);
        }
    }
}

fn decrypt_command(matches: &ArgMatches, config: &Config) {
    // let key_filename = matches.value_of("key_filename").unwrap_or("");
    let key = load_key(matches, config);

    let cyphertext_filename = matches.value_of("cyphertext_filename").unwrap();
    let plaintext_filename = matches.value_of("plaintext_filename").unwrap_or("");
    let fail_if_cannot_decrypt = !matches.is_present("try");

    match key.owns_file(cyphertext_filename) {
        Ok(owns_file) => {
            if !owns_file {
                std::process::exit(match fail_if_cannot_decrypt {
                    true => 1,
                    false => 0,
                });
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let cyphertext = read_bytes(cyphertext_filename).unwrap();

    match key.decrypt(&cyphertext) {
        Ok(decrypted_data) => {
            if plaintext_filename.len() > 0 {
                let mut file = File::create(plaintext_filename).expect("failed to create new file");
                file.write(&decrypted_data)
                    .expect("failed to write to output filename");
                logger::err::ok(format!(
                    "wrote plaintext data in: {}",
                    style(plaintext_filename).color256(190)
                ));
            } else {
                println!("{}", b64encode(&decrypted_data));
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
fn check_command(matches: &ArgMatches, config: &Config) {
    // let key_filename = matches.value_of("key_filename").unwrap_or("");

    let key = load_key(matches, config);
    let cyphertext_filename = matches.value_of("cyphertext_filename").unwrap();
    let fail_if_not_encrypted = !matches.is_present("try");

    match key.owns_file(cyphertext_filename) {
        Ok(true) => {
            logger::err::ok(format!(
                "{}{}{}",
                cyphertext_filename,
                style(" is ").color256(logger::colors::TYPE_OK),
                style("encrypted.").color256(156),
            ));
            std::process::exit(0);
        }
        Ok(false) => {
            logger::err::error(format!(
                "{}{}{}",
                cyphertext_filename,
                style(" is ").color256(logger::colors::TYPE_ERROR),
                style("encrypted.").color256(202),
            ));
            std::process::exit(match fail_if_not_encrypted {
                true => 1,
                false => 0,
            });
        }
        Err(error) => {
            logger::err::error(format!("{}", error));
            std::process::exit(1);
        }
    };
}

fn main() {
    panic::set_hook(Box::new(|e| {
        eprintln!("PANIC: {:?}", e);
    }));
    let config = Config::default().expect("cannot read default config: ~/.toolz.yaml");
    let key_cycles = config.cycles.key.to_string();
    let salt_cycles = config.cycles.salt.to_string();
    let iv_cycles = config.cycles.iv.to_string();
    let app = App::new("aes256")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(core::VERSION)
        .author(core::AUTHOR)
        .about("perform aes-256-cbc encryption/decryption based on PBKDF2 of password")
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .short("n")
                .takes_value(false),
        )
        // .subcommand(
        //     SubCommand::with_name("ask")
        //         .about("ask for password and confirmation")
        //         .arg(
        //             Arg::with_name("force")
        //                 .long("force")
        //                 .short("f")
        //                 .takes_value(false),
        //         ),
        // )
        .subcommand(
            SubCommand::with_name("generate")
                .about("generate key")
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required_unless_one(&["key_filename", "ask_password"])
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .short("p")
                        .required_unless_one(&["password", "ask_password"])
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("key_cycles")
                        .default_value(&key_cycles)
                        .long("key")
                        .short("K")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("salt_cycles")
                        .default_value(&salt_cycles)
                        .long("salt")
                        .short("S")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("iv_cycles")
                        .default_value(&iv_cycles)
                        .long("iv")
                        .short("I")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("encrypt")
                .about("encrypt file or string")
                .arg(
                    Arg::with_name("try")
                        .long("try")
                        .short("t")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("string")
                        .long("string")
                        .short("s")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("plaintext_filename")
                        .long("input-filename")
                        .short("i")
                        .required_unless("string")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("cyphertext_filename")
                        .long("output-filename")
                        .short("o")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required_unless_one(&["key_filename", "ask_password"])
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .required(false)
                        .short("p")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required_unless_one(&["password", "ask_password"])
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("decrypt")
                .about("decrypt file")
                .arg(
                    Arg::with_name("try")
                        .long("try")
                        .short("t")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .short("p")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("plaintext_filename")
                        .long("output-filename")
                        .short("o")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("cyphertext_filename")
                        .long("input-filename")
                        .short("i")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("check")
                .about("check file")
                .arg(
                    Arg::with_name("try")
                        .long("try")
                        .short("t")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("password")
                        .long("password")
                        .short("P")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("key_filename")
                        .long("key-filename")
                        .short("k")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ask_password")
                        .long("ask-password")
                        .short("p")
                        .required(false)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("cyphertext_filename")
                        .long("input-filename")
                        .short("i")
                        .required(true)
                        .takes_value(true),
                ),
        );

    let matches = app.get_matches();
    //let dry_run = matches.is_present("dry_run");

    match matches.subcommand() {
        ("generate", Some(matches)) => {
            generate_command(matches);
        }
        ("encrypt", Some(matches)) => {
            encrypt_command(matches, &config);
        }
        ("decrypt", Some(matches)) => {
            decrypt_command(matches, &config);
        }
        ("check", Some(matches)) => {
            check_command(matches, &config);
        }
        (cmd, Some(_matches)) => {
            logger::err::warning(format!("command not implemented: {}", cmd));
        }
        (cmd, None) => {
            logger::err::warning(format!("unhandled command: {}", cmd));
        }
    }
}
