// Example for AES-128 CBC mode
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use console::style;
use std::io::Write;
use std::{fs::File, panic};
use toolz::ioutils::{b64encode, create_file};

use toolz::aes256cbc::Key;
use toolz::aes256cbc::{get_default_config_path, Config};
use toolz::config::YamlFile;
use toolz::{colors, core, ioutils::read_bytes, logger};


use libaes::Cipher;

let my_key = b"This is the key!"; // key is 16 bytes, i.e. 128-bit
let plaintext = b"A plaintext";
let iv = b"This is 16 bytes";

// Create a new 128-bit cipher
let cipher = Cipher::new_128(my_key);

// Encryption
let encrypted = cipher.cbc_encrypt(iv, plaintext);

// Decryption
let decrypted = cipher.cbc_decrypt(iv, &encrypted[..]);
