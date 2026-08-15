use std::{
    collections::{hash_map, HashMap},
    io,
};

use serde::Deserialize;

const DEFAULT_LANGS_CONFIG: &str = include_str!("../config/langs.toml");

#[derive(Debug, Deserialize)]
pub struct Language {
    pub compile: String,
    pub compile_args: Box<[String]>,
    pub source: String,
    pub source_exts: Box<[String]>,
    pub run: String,
}

pub struct LanguageMap {
    langs: Box<[Language]>,
    source_exts: HashMap<String, usize>,
}

impl LanguageMap {
    pub fn load() -> io::Result<Self> {
        let mut langs = Vec::new();
        let mut source_exts = HashMap::new();

        let table: toml::Table = toml::from_str(DEFAULT_LANGS_CONFIG)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        for (key, value) in table {
            let lang: Language = value.try_into().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse config for '{key}': {e}"),
                )
            })?;

            let index = langs.len();
            for source_ext in &lang.source_exts {
                match source_exts.entry(source_ext.to_ascii_lowercase()) {
                    hash_map::Entry::Occupied(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Conflicting source extension '{source_ext}'"),
                        ));
                    }
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(index);
                    }
                }
            }
            langs.push(lang);
        }

        Ok(Self {
            langs: langs.into_boxed_slice(),
            source_exts,
        })
    }

    pub fn get_by_source_ext(&self, source_ext: &str) -> Option<&Language> {
        self.source_exts
            .get(&source_ext.to_ascii_lowercase())
            .copied()
            .map(|index| &self.langs[index])
    }
}
