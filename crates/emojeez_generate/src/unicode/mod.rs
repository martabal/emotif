use std::error::Error;

use unicode_types::{Emoji, Entry, Group, SkinTone, Status, Version};

use crate::github::Gemoji;
use crate::util;

pub const VERSION_MAJOR: usize = 17;
pub const VERSION_MINOR: usize = 0;
pub const VERSION_PATCH: usize = 0;

type OwnedEntry = Entry<String, Vec<String>>;
type OwnedEmoji = Emoji<String, Vec<String>>;

fn unicode_url() -> String {
    format!(
        "https://unicode.org/Public/{VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}/emoji/emoji-test.txt"
    )
}

pub fn parse_data(data: &str) -> Result<Vec<OwnedEntry>, Box<dyn Error>> {
    let entries = parse_emoji_data(data)?;
    Ok(entries)
}

fn parse_emoji_data(data: &str) -> Result<Vec<OwnedEntry>, Box<dyn Error>> {
    let mut entries = Vec::new();
    let mut group: &str = "";
    let mut subgroup = String::new();

    for line in data.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(g) = line.strip_prefix("# group: ") {
            group = g.trim();
        }
        if let Some(s) = line.strip_prefix("# subgroup: ") {
            s.trim().clone_into(&mut subgroup);
        }
        if line.starts_with('#') {
            continue;
        }
        if group.is_empty() {
            return Err("missing group".into());
        }
        if subgroup.is_empty() {
            return Err("missing subgroup".into());
        }
        let entry = parse_entry(Group::try_from(group.to_string())?, subgroup.clone(), line)?;
        entries.push(entry);
    }
    Ok(entries)
}

fn parse_entry(group: Group, subgroup: String, line: &str) -> Result<OwnedEntry, Box<dyn Error>> {
    let (code_points, rest) = line.split_once(';').ok_or("expected code points")?;
    let (status, rest) = rest.split_once('#').ok_or("expected status")?;
    let mut rest = rest.trim().splitn(3, char::is_whitespace);
    let emoji = rest.next().ok_or("expected emoji")?;
    let unicode_version = rest.next().ok_or("expected unicode version")?;
    let name = rest.next().ok_or("expected name")?;
    if rest.next().is_some() {
        return Err("unexpected extra data".into());
    }

    if emoji != String::from_iter(parse_code_points(code_points)?) {
        return Err("emoji mismatch".into());
    }

    Ok(Entry {
        group,
        subgroup,
        status: parse_status(status.trim())?,
        emoji: emoji.to_owned(),
        unicode_version: parse_unicode_version(unicode_version.trim())?,
        name: name.to_owned(),
        ios_version: None,
        tags: Vec::new(),
        aliases: Vec::new(),
    })
}

fn parse_status(s: &str) -> Result<Status, Box<dyn Error>> {
    match s {
        "fully-qualified" => Ok(Status::FullyQualified),
        "minimally-qualified" => Ok(Status::MinimallyQualified),
        "unqualified" => Ok(Status::Unqualified),
        "component" => Ok(Status::Component),
        _ => Err(format!("invalid status: {s:?}").into()),
    }
}

fn parse_unicode_version(s: &str) -> Result<Version, Box<dyn Error>> {
    let s = s.strip_prefix('E').ok_or("missing 'E'")?;
    let (major, minor) = s.split_once('.').ok_or("missing decimal")?;
    Ok(Version {
        major: major.parse()?,
        minor: minor.parse()?,
    })
}

fn parse_code_points(s: &str) -> Result<Vec<char>, Box<dyn Error>> {
    s.split_ascii_whitespace()
        .map(|cp| {
            let scalar = u32::from_str_radix(cp, 16)?;
            char::from_u32(scalar).ok_or_else(|| "invalid Unicode scalar value".into())
        })
        .collect()
}

pub fn build() -> Result<Vec<OwnedEmoji>, Box<dyn Error>> {
    let gemojis: Vec<Gemoji> = crate::github::build()?;
    let mut emojis: Vec<OwnedEmoji> = Vec::new();
    let data = util::cached_download(&unicode_url())?;
    for mut entry in parse_data(&data).map_err(|e| format!("Failed to parse data: {e}"))? {
        if entry.group == Group::Component {
            continue;
        }

        if let Some(g) = gemojis.iter().find(|g| g.emoji == entry.emoji) {
            entry.ios_version = Some(g.ios_version);
            entry.tags.clone_from(&g.tags);
            entry.aliases.clone_from(&g.aliases);
        }

        match entry.status {
            Status::Component => unreachable!(),
            Status::MinimallyQualified | Status::Unqualified => {
                let last = emojis.last_mut().ok_or_else(|| {
                    format!(
                        "failed to find fully qualified variation for '{}'",
                        entry.name
                    )
                })?;
                last.variations.push(entry.emoji);
            }
            Status::FullyQualified => {
                let skin_tone = parse_skin_tone(&entry)?;

                match skin_tone {
                    None | Some(SkinTone::Default) => {
                        emojis.push(Emoji {
                            entry,
                            skin_tones: 1,
                            skin_tone,
                            variations: Vec::new(),
                        });
                    }
                    Some(skin_tone) => {
                        let i = {
                            let (i, def) = emojis
                                .iter_mut()
                                .enumerate()
                                .rev()
                                .find(|(_, e)| {
                                    matches!(e.skin_tone, None | Some(SkinTone::Default))
                                        && e.entry.group == entry.group
                                        && e.entry.subgroup == entry.subgroup
                                })
                                .ok_or_else(|| {
                                    format!(
                                        "failed to find the default skin tone for '{}'",
                                        entry.name
                                    )
                                })?;
                            def.skin_tone = Some(SkinTone::Default);
                            def.skin_tones += 1;
                            i
                        };

                        let j = emojis[i..].partition_point(|e| e.skin_tone < Some(skin_tone));
                        emojis.insert(
                            i + j,
                            Emoji {
                                entry,
                                skin_tones: 1,
                                skin_tone: Some(skin_tone),
                                variations: Vec::new(),
                            },
                        );
                    }
                }
            }
        }
    }

    Ok(emojis)
}

fn parse_skin_tone(entry: &Entry<String, Vec<String>>) -> Result<Option<SkinTone>, String> {
    let skin_tones: Vec<_> = entry
        .emoji
        .chars()
        .filter_map(|c| match c {
            '\u{1f3fb}' => Some(SkinTone::Light),
            '\u{1f3fc}' => Some(SkinTone::MediumLight),
            '\u{1f3fd}' => Some(SkinTone::Medium),
            '\u{1f3fe}' => Some(SkinTone::MediumDark),
            '\u{1f3ff}' => Some(SkinTone::Dark),
            _ => None,
        })
        .collect();

    let skin_tone = match *skin_tones.as_slice() {
        [] => return Ok(None),
        [a] => a,
        [SkinTone::Light, SkinTone::MediumLight] => SkinTone::LightAndMediumLight,
        [SkinTone::Light, SkinTone::Medium] => SkinTone::LightAndMedium,
        [SkinTone::Light, SkinTone::MediumDark] => SkinTone::LightAndMediumDark,
        [SkinTone::Light, SkinTone::Dark] => SkinTone::LightAndDark,
        [SkinTone::MediumLight, SkinTone::Light] => SkinTone::MediumLightAndLight,
        [SkinTone::MediumLight, SkinTone::Medium] => SkinTone::MediumLightAndMedium,
        [SkinTone::MediumLight, SkinTone::MediumDark] => SkinTone::MediumLightAndMediumDark,
        [SkinTone::MediumLight, SkinTone::Dark] => SkinTone::MediumLightAndDark,
        [SkinTone::Medium, SkinTone::Light] => SkinTone::MediumAndLight,
        [SkinTone::Medium, SkinTone::MediumLight] => SkinTone::MediumAndMediumLight,
        [SkinTone::Medium, SkinTone::MediumDark] => SkinTone::MediumAndMediumDark,
        [SkinTone::Medium, SkinTone::Dark] => SkinTone::MediumAndDark,
        [SkinTone::MediumDark, SkinTone::Light] => SkinTone::MediumDarkAndLight,
        [SkinTone::MediumDark, SkinTone::MediumLight] => SkinTone::MediumDarkAndMediumLight,
        [SkinTone::MediumDark, SkinTone::Medium] => SkinTone::MediumDarkAndMedium,
        [SkinTone::MediumDark, SkinTone::Dark] => SkinTone::MediumDarkAndDark,
        [SkinTone::Dark, SkinTone::Light] => SkinTone::DarkAndLight,
        [SkinTone::Dark, SkinTone::MediumLight] => SkinTone::DarkAndMediumLight,
        [SkinTone::Dark, SkinTone::Medium] => SkinTone::DarkAndMedium,
        [SkinTone::Dark, SkinTone::MediumDark] => SkinTone::DarkAndMediumDark,
        [a, b] if a == b => a,
        _ => {
            return Err(format!(
                "unrecognized skin tone combination: {skin_tones:?}",
            ));
        }
    };

    Ok(Some(skin_tone))
}
