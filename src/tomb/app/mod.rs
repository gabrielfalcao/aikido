pub mod application;
pub mod components;
pub mod routes;
pub mod state;

use crate::ironpunk;
pub use application::*;
pub use components::{
    menu::{dummy_paragraph, MenuComponent},
    modal::Modal,
};
pub use routes::*;
pub use state::*;

use super::{AES256Secret, AES256Tomb};
use crate::aes256cbc::{Config as AesConfig, Key};

use std::{cell::RefCell, rc::Rc};

pub fn start(
    tomb: AES256Tomb,
    key: Key,
    aes_config: AesConfig,
) -> Result<(), ironpunk::SharedError> {
    let app = Application::new(key, tomb, aes_config.clone());
    let about = About::new(aes_config.clone());
    let configuration = Configuration::new(aes_config.clone());
    let mut router = ironpunk::SharedRouter::new();
    let app = Rc::new(RefCell::new(app));
    let about = Rc::new(RefCell::new(about));
    let configuration = Rc::new(RefCell::new(configuration));

    router.add("/about", about.clone());
    router.add("/configuration", configuration.clone());
    router.add("/:filter", app.clone());
    router.add("/", app.clone());
    ironpunk::start(router)
}
