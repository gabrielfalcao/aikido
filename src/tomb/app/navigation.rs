use crate::ironpunk::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Navigation {
    location: String,
    history: Vec<String>,
}

impl Navigation {
    pub fn new(location: &str) -> Navigation {
        let location = String::from(location);
        Navigation {
            location: location.clone(),
            history: vec![location],
        }
    }
    pub fn goto(&mut self, location: &str) {
        let location = String::from(location);
        self.history.push(location.clone());
        self.location = location;
    }
    pub fn goback(&mut self) {
        match self.history.pop() {
            Some(location) => {
                self.location = location;
            }
            None => {}
        }
    }
    pub fn get_location(&self) -> String {
        self.location.clone()
    }
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }
    pub fn matches(&self, route: &dyn Route) -> bool {
        route.matches_path(self.location.clone())
    }
}
