use std::error::Error;

use serde::Deserialize;
use unicode_types::Version;

use crate::util;

pub const VERSION_MAJOR: usize = 4;
pub const VERSION_MINOR: usize = 1;
pub const VERSION_PATCH: usize = 0;

fn genmoji_url() -> String {
    format!(
        "https://raw.githubusercontent.com/github/gemoji/v{VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}/db/emoji.json"
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct Gemoji {
    pub emoji: String,
    pub aliases: Vec<String>,
    pub ios_version: Version,
    pub tags: Vec<String>,
}

pub fn build() -> Result<Vec<Gemoji>, Box<dyn Error>> {
    let buf = util::cached_download(&genmoji_url())?;
    let emojis: Vec<Gemoji> = serde_json::from_str(&buf)?;
    Ok(emojis)
}
