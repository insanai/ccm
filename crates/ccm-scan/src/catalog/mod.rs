//! Declarative syntax metadata, separate from the specialized counting policies.
mod data0;
mod data1;
mod data2;
mod data3;
mod data4;
mod data5;
mod data6;
mod data7;
mod data8;
mod scan;
use crate::{Language, LanguageInfo};
pub(super) use scan::count;
use std::collections::BTreeMap;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub(super) struct Record {
    pub info: LanguageInfo,
    pub lines: &'static [&'static str],
    pub blocks: &'static [(&'static str, &'static str)],
    pub quotes: &'static [(&'static str, &'static str, bool)],
    pub nested: bool,
    pub insensitive: bool,
}

pub(super) fn records() -> &'static [Record] {
    static RECORDS: LazyLock<Vec<Record>> = LazyLock::new(|| {
        [
            data0::RECORDS,
            data1::RECORDS,
            data2::RECORDS,
            data3::RECORDS,
            data4::RECORDS,
            data5::RECORDS,
            data6::RECORDS,
            data7::RECORDS,
            data8::RECORDS,
        ]
        .into_iter()
        .flatten()
        .copied()
        .collect()
    });
    &RECORDS
}

/// Returns the catalog language with this exact display name.
pub(super) fn by_name(name: &str) -> Option<Language> {
    records()
        .iter()
        .find(|record| record.info.name == name)
        .map(|record| record.info.language)
}

pub(super) fn lookup(key: &str, filename: bool) -> Option<Language> {
    static INDEX: LazyLock<(BTreeMap<String, Language>, BTreeMap<String, Language>)> =
        LazyLock::new(|| {
            let mut extensions = BTreeMap::new();
            let mut filenames = BTreeMap::new();
            for record in records() {
                for &extension in record.info.extensions {
                    extensions
                        .entry(extension.to_ascii_lowercase())
                        .or_insert(record.info.language);
                }
                for &name in record.info.filenames {
                    filenames
                        .entry(name.to_owned())
                        .or_insert(record.info.language);
                }
            }
            (extensions, filenames)
        });
    if filename {
        INDEX.1.get(key).copied()
    } else {
        INDEX.0.get(&key.to_ascii_lowercase()).copied()
    }
}
