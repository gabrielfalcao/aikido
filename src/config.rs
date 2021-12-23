use crate::colors;

use console::style;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn absolute_path(path: &str) -> Result<String, String> {
    match Path::new(path).canonicalize() {
        Ok(current_dir) => match current_dir.as_os_str().to_str() {
            Some(path) => Ok(String::from(path)),
            None => Ok(String::from(path)),
        },
        Err(_) => Ok(String::from(path)),
    }
}

pub trait YamlFileError {
    fn with_message(message: String) -> Self;
}

pub trait YamlFile<Error: YamlFileError> {
    fn from_yaml<'a>(data: String) -> Result<Self, Error>
    where
        Self: DeserializeOwned,
        Self: Clone,
        Self: PartialEq,
    {
        let cfg: Self = match serde_yaml::from_str(&data) {
            Ok(config) => config,
            Err(error) => {
                return Err(Error::with_message(format!(
                    "failed to deserialize yaml config: {}",
                    error
                )))
            }
        };
        Ok(cfg)
    }
    /// Serialize YamlFile to String
    fn to_yaml(&self) -> Result<String, Error>
    where
        Self: Serialize,
    {
        match serde_yaml::to_string(&self) {
            Ok(val) => Ok(val),
            Err(e) => Err(Error::with_message(format!(
                "failed to encode key to yaml: {}",
                e
            ))),
        }
    }

    /// Loads the default config somehow
    fn default() -> Result<Self, Error>
    where
        Self: DeserializeOwned;

    /// Loads the default config from a yaml file
    fn import(filename: &str) -> Result<Self, Error>
    where
        Self: DeserializeOwned,
        Self: Clone,
        Self: PartialEq,
    {
        let filename = match absolute_path(filename) {
            Ok(filename) => filename,
            Err(err) => return Err(Error::with_message(err)),
        };
        match fs::read_to_string(filename.as_str()) {
            Ok(yaml) => YamlFile::from_yaml(yaml),
            Err(error) => {
                return Err(Error::with_message(format!(
                    "{}{}{}",
                    style("failed to read file ").color256(colors::ERR_MSG),
                    style(filename).color256(colors::ERR_VAR),
                    style(format!("\n\t{}", error)).color256(colors::ERR_HLT),
                )))
            }
        }
    }
    /// Store YAML-serialized key into a file
    fn export(&self, filename: &str) -> Result<String, Error>
    where
        Self: Serialize,
    {
        let filename = match absolute_path(filename) {
            Ok(filename) => filename,
            Err(err) => return Err(Error::with_message(err)),
        };

        let yaml = match self.to_yaml() {
            Ok(val) => val,
            Err(error) => return Err(error),
        };
        let mut file = File::create(filename.as_str()).expect("failed to create new file");
        file.write(yaml.as_ref()).unwrap();
        Ok(filename)
    }
}
