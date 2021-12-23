pub mod ui;
use chrono::prelude::*;

use crate::aes256cbc::{Config as AesConfig, Digest, Key};
use crate::{
    colors,
    config::{YamlFile, YamlFileError},
    ioutils::{b64decode, b64encode},
    logger,
};
use console::style;
use fnmatch_regex::glob_to_regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{borrow::Borrow, fmt};

const DEFAULT_TOMB_PATH: &'static str = "~/.tomb.yaml";

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl YamlFileError for Error {
    fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// The tomburation for the Key.
///
/// It contains the cycles for key, salt and iv used in key derivation.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct AES256Secret {
    pub digest: Digest,
    pub path: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl AES256Secret {
    /// Creates a new tomb based on a key
    pub fn new(path: String, value: Vec<u8>, key: Key) -> AES256Secret {
        AES256Secret {
            digest: key.digest(),
            path,
            value: b64encode(&value),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    pub fn value_bytes(&self) -> Vec<u8> {
        b64decode(&self.value.as_bytes()).unwrap()
    }
    pub fn update(&mut self, path: String, plaintext: Vec<u8>, key: Key) -> Result<(), Error> {
        self.digest = key.digest();
        self.path = path.clone();
        let cyphertext = match key.encrypt(&plaintext) {
            Ok(cypher) => cypher,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}{}",
                    style("cannot encrypt data for path").color256(198),
                    style(path).color256(190),
                    style(" with the provided key.").color256(198),
                    style(format!("\n\t{:?}", error)).color256(197),
                )));
            }
        };
        self.value = b64encode(&cyphertext);
        self.updated_at = Utc::now();
        Ok(())
    }
    pub fn get_base64_string(&self, path: &str, key: Key) -> Result<String, Error> {
        match self.get_bytes(path, key) {
            Ok(bytes) => Ok(b64encode(&bytes)),
            Err(error) => Err(error),
        }
    }
    pub fn get_string(&self, path: &str, key: Key) -> Result<String, Error> {
        match self.get_bytes(path, key) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(value) => Ok(value),
                Err(error) => {
                    return Err(Error::with_message(format!(
                        "{}{}{}{}",
                        style("cannot convert value from key ").color256(198),
                        style(path).color256(190),
                        style(" to a valid utf-8 string.").color256(198),
                        style(format!("\n\t{:?}", error)).color256(197),
                    )));
                }
            },
            Err(error) => Err(error),
        }
    }
    pub fn get_bytes(&self, path: &str, key: Key) -> Result<Vec<u8>, Error> {
        if String::from(path) != self.path {
            return Err(Error::with_message(format!(
                "path {} does not match {}",
                path, self.path
            )));
        }
        match key.decrypt(&self.value_bytes()) {
            Ok(plaintext) => Ok(plaintext),
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}{}",
                    style("cannot decrypt value from secret ").color256(198),
                    style(path).color256(190),
                    style(" with the provided key.").color256(198),
                    style(format!("\n\t{:?}", error)).color256(197),
                )));
            }
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct AES256Tomb {
    pub digest: Digest,
    pub config: AesConfig,
    pub data: BTreeMap<String, AES256Secret>,
}
impl YamlFile<Error> for AES256Tomb {
    fn default() -> Result<AES256Tomb, Error> {
        let filename = shellexpand::tilde(DEFAULT_TOMB_PATH);
        AES256Tomb::import(filename.borrow())
    }
}

impl AES256Tomb {
    /// Creates a new tomb based on a key
    pub fn new(key: Key, config: AesConfig) -> AES256Tomb {
        AES256Tomb {
            digest: key.digest(),
            data: BTreeMap::new(),
            config,
        }
    }
    pub fn list(&self, pattern: &str) -> Result<Vec<AES256Secret>, Error> {
        let regex = match glob_to_regex(pattern) {
            Ok(regex) => regex,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}",
                    style("invalid pattern ").color256(colors::ERR_MSG),
                    style(pattern).color256(colors::ERR_VAR),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                )))
            }
        };
        let mut result = Vec::new();
        for (path, secret) in &self.data {
            if regex.is_match(&path) {
                result.push(secret.clone());
            }
        }
        Ok(result)
    }

    pub fn delete_secret(&mut self, path: &str) -> Result<(), Error> {
        match self.data.remove(&String::from(path)) {
            Some(_) => Ok(()),
            None => Err(Error::with_message(format!(
                "{}{}",
                style("key not found ").color256(colors::ERR_MSG),
                style(path).color256(colors::ERR_VAR),
            ))),
        }
    }
    pub fn add_secret(&mut self, path: &str, plaintext: String, key: Key) -> Result<(), Error> {
        self.add_secret_from_bytes(path, Vec::from(plaintext), key)
    }
    pub fn add_secret_from_bytes(
        &mut self,
        path: &str,
        plaintext: Vec<u8>,
        key: Key,
    ) -> Result<(), Error> {
        let cyphertext = match key.encrypt(&plaintext) {
            Ok(cypher) => cypher,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}{}",
                    style("cannot encrypt data for path").color256(198),
                    style(path).color256(190),
                    style(" with the provided key.").color256(198),
                    style(format!("\n\t{:?}", error)).color256(197),
                )));
            }
        };
        let secret = AES256Secret::new(String::from(path), cyphertext, key);
        self.data.insert(String::from(path), secret);
        Ok(())
    }
    pub fn derive_key(&self, password: &str) -> Key {
        Key::from_password(password.as_bytes(), &self.config)
    }

    pub fn get(&self, path: &str) -> Result<AES256Secret, Error> {
        match self.data.get(&String::from(path)) {
            Some(secret) => Ok(secret.clone()),
            None => Err(Error::with_message(format!(
                "{}{}",
                style("key not found: ").color256(198),
                style(path).color256(190),
            ))),
        }
    }
    pub fn get_base64_string(&self, path: &str, key: Key) -> Result<String, Error> {
        self.get(path)?.get_base64_string(path, key)
    }
    pub fn get_string(&self, path: &str, key: Key) -> Result<String, Error> {
        self.get(path)?.get_string(path, key)
    }
    pub fn get_bytes(&self, path: &str, key: Key) -> Result<Vec<u8>, Error> {
        self.get(path)?.get_bytes(path, key)
    }
}

#[cfg(test)]
mod tests {
    use crate::aes256cbc::Config as AesConfig;
    use crate::aes256cbc::Key;
    use crate::tomb::AES256Tomb;
    use k9::assert_equal;

    fn generate_key() -> (Key, AesConfig) {
        let config = AesConfig::builtin(None);
        let password = String::from("123456");
        (Key::from_password(&password.as_bytes(), &config), config)
    }
    #[test]
    fn test_create_tomb_and_manage_secrets() {
        let (key, config) = generate_key();

        let mut tomb = AES256Tomb::new(key.clone(), config);
        tomb.add_secret_from_bytes(
            "my-secret",
            Vec::from("some bytes"),
            tomb.derive_key("123456"),
        )
        .expect("secret should be added");
        tomb.add_secret(
            "another-secret",
            String::from("more bytes"),
            tomb.derive_key("123456"),
        )
        .expect("secret should be added");

        let plaintext = tomb
            .get_bytes("my-secret", tomb.derive_key("123456"))
            .expect("secret should have been stored by previous statement(s)");

        assert_equal!(plaintext, Vec::from("some bytes"));

        let plaintext = tomb
            .get_string("another-secret", tomb.derive_key("123456"))
            .expect("secret should have been stored by previous statement(s)");

        assert_equal!(plaintext, String::from("more bytes"));

        let secrets = tomb.list("*").expect("failed to list *");
        assert_equal!(secrets.len(), 2);

        let first = tomb.list("my-*").expect("failed to list my-*");
        assert_equal!(first.len(), 1);

        let last = tomb.list("another-*").expect("failed to list another-*");
        assert_equal!(last.len(), 1);
    }
}
