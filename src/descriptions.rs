use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use crate::{
    binaries::Binary,
};

pub struct Description {
    pub value: String,
}

type Descriptions = HashMap<String, Arc<Description>>;

static DESCRIPTIONS: LazyLock<Mutex<Descriptions>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl Description {
    pub fn new(value: String) -> Self {
        let striped = strip_ansi_escapes::strip(value);
        let value = String::from_utf8(striped).unwrap();
        let mut view: &str = value.as_ref();

        if matches!(value.chars().next(), Some('.')) {
            let space = value
                .char_indices()
                .find(|&(_, c)| c == ' ');

            if let Some((index, _)) = space {
                view = &view[index..];
            }
        }

        Self {
            value: view
                .trim()
                .replace('\n', " ")
                .to_owned(),
        }
    }
}

impl From<String> for Description {
    fn from(value: String) -> Self {
        Self { value }
    }
}

pub fn insert_description(binary: &Binary, description: Arc<Description>) {
    let mut descriptions = DESCRIPTIONS.lock().unwrap();
    descriptions.insert(
        binary.name.to_string(),
        description
    );
}

pub fn remove_description(binary: &Binary) {
    let mut descriptions = DESCRIPTIONS.lock().unwrap();
    descriptions.remove(&binary.name);
}

pub fn get_description(binary: &Binary) -> Option<Arc<Description>> {
    let descriptions = DESCRIPTIONS.lock().unwrap();

    descriptions
        .get(&binary.name)
        .map(|v| v.clone())
}
