#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entry<S, C>
where
    S: AsRef<str> + Clone,
    C: AsRef<[S]> + Clone,
{
    pub group: Group,
    pub subgroup: S,
    pub status: Status,
    pub unicode_version: Version,
    pub emoji: S,
    pub name: S,
    pub ios_version: Option<Version>,
    pub tags: C,
    pub aliases: C,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct Version {
    pub major: i16,
    pub minor: i16,
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (major_str, minor_str) = s.split_once('.').unwrap();

        let major = major_str
            .parse::<i16>()
            .map_err(|_| format!("Invalid major version: {major_str}"))?;

        let minor = minor_str
            .parse::<i16>()
            .map_err(|_| format!("Invalid minor version: {minor_str}"))?;

        Ok(Self { major, minor })
    }
}

impl TryFrom<String> for Version {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Emoji<S, C>
where
    S: AsRef<str> + Clone,
    C: AsRef<[S]> + Clone,
{
    pub entry: Entry<S, C>,
    pub skin_tones: usize,
    pub skin_tone: Option<SkinTone>,
    pub variations: C,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// A qualified emoji character, or an emoji sequence in which each emoji character is qualified.
    ///
    /// <https://www.unicode.org/reports/tr51/#def_fully_qualified_emoji>
    FullyQualified,

    /// An emoji sequence in which the first character is qualified but the sequence is not fully qualified.
    ///
    /// <https://www.unicode.org/reports/tr51/#def_minimally_qualified_emoji>
    MinimallyQualified,

    /// An emoji that is neither fully-qualified nor minimally qualified.
    ///
    /// <https://www.unicode.org/reports/tr51/#def_unqualified_emoji>
    Unqualified,

    /// Not an emoji, but defines building block code point(s) for emojis.
    Component,
}

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Five symbol modifier characters that provide for a range of skin tones for human emoji were released in Unicode Version 8.0.
///
/// <https://www.unicode.org/reports/tr51/#Diversity>
pub enum SkinTone {
    Default,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
    LightAndMediumLight,
    LightAndMedium,
    LightAndMediumDark,
    LightAndDark,
    MediumLightAndLight,
    MediumLightAndMedium,
    MediumLightAndMediumDark,
    MediumLightAndDark,
    MediumAndLight,
    MediumAndMediumLight,
    MediumAndMediumDark,
    MediumAndDark,
    MediumDarkAndLight,
    MediumDarkAndMediumLight,
    MediumDarkAndMedium,
    MediumDarkAndDark,
    DarkAndLight,
    DarkAndMediumLight,
    DarkAndMedium,
    DarkAndMediumDark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The groupings include: faces, people, body-parts, emotion, clothing, animals, plants, food, places, transport, and so on.
///
/// The ordering also groups more naturally for the purpose of selection in input palettes.
///
/// <https://www.unicode.org/reports/tr51/#Sorting>
pub enum Group {
    SmileysAndEmotion,
    PeopleAndBody,
    AnimalsAndNature,
    FoodAndDrink,
    TravelAndPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
    Component,
}

#[cfg(feature = "serde")]
impl Serialize for Group {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl TryFrom<String> for Group {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<&str> for Group {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        for g in [
            Self::SmileysAndEmotion,
            Self::PeopleAndBody,
            Self::AnimalsAndNature,
            Self::FoodAndDrink,
            Self::TravelAndPlaces,
            Self::Activities,
            Self::Objects,
            Self::Symbols,
            Self::Flags,
            Self::Component,
        ] {
            if g.as_str() == s {
                return Ok(g);
            }
        }
        Err(format!("invalid group: {s}"))
    }
}

impl Group {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SmileysAndEmotion => "Smileys & Emotion",
            Self::PeopleAndBody => "People & Body",
            Self::AnimalsAndNature => "Animals & Nature",
            Self::FoodAndDrink => "Food & Drink",
            Self::TravelAndPlaces => "Travel & Places",
            Self::Activities => "Activities",
            Self::Objects => "Objects",
            Self::Symbols => "Symbols",
            Self::Flags => "Flags",
            Self::Component => "Component",
        }
    }
}

impl<S, C> Emoji<S, C>
where
    S: AsRef<str> + Clone,
    C: AsRef<[S]> + Clone,
{
    pub fn matches_search(&self, q: &str) -> bool {
        if q.is_empty() {
            return true;
        }
        let q = q.to_lowercase();
        if self.entry.emoji.as_ref().to_lowercase().contains(&q) {
            return true;
        }
        if self.entry.name.as_ref().to_lowercase().contains(&q) {
            return true;
        }
        if self.entry.subgroup.as_ref().to_lowercase().contains(&q) {
            return true;
        }
        for t in self.entry.tags.as_ref() {
            if t.as_ref().to_lowercase().contains(&q) {
                return true;
            }
        }
        for a in self.entry.aliases.as_ref() {
            if a.as_ref().to_lowercase().contains(&q) {
                return true;
            }
        }
        false
    }
}
