use std::fmt::Write;

use unicode_types::{Emoji, Entry, Group, SkinTone, Status, Version};

use crate::util::{struct_name, struct_package};

#[allow(clippy::too_many_lines)]
pub fn generate_rust_code(data: &Vec<unicode_types::Emoji<String, Vec<String>>>) -> String {
    let mut code = String::from(&format!(
        r"use {}::{{{}, {}, {}, {}, {}, {}}};

pub const EMOJIS: &[{}<&'static str, &'static [&'static str]>] = &[
",
        struct_package::<Group>(),
        struct_name::<Emoji<String, Vec<String>>>(),
        struct_name::<Entry<String, Vec<String>>>(),
        struct_name::<Group>(),
        struct_name::<SkinTone>(),
        struct_name::<Status>(),
        struct_name::<Version>(),
        struct_name::<Emoji<String, Vec<String>>>(),
    ));

    for emoji in data {
        let mut indent_count = 1;
        writeln!(
            code,
            "{}{} {{",
            "\t".repeat(indent_count),
            struct_name::<Emoji<String, Vec<String>>>()
        )
        .unwrap();
        indent_count += 1;
        writeln!(
            code,
            "{}entry: {} {{",
            "\t".repeat(indent_count),
            struct_name::<Entry<String, Vec<String>>>()
        )
        .unwrap();
        indent_count += 1;
        writeln!(
            code,
            r#"{}group: {}::{:?},
{}subgroup: "{}",
{}status: {}::{:?},
{}unicode_version: {} {{ major: {}, minor: {} }},
{}emoji: "{}",
{}name: "{}","#,
            "\t".repeat(indent_count),
            struct_name::<Group>(),
            emoji.entry.group,
            "\t".repeat(indent_count),
            emoji.entry.subgroup,
            "\t".repeat(indent_count),
            struct_name::<Status>(),
            emoji.entry.status,
            "\t".repeat(indent_count),
            struct_name::<Version>(),
            emoji.entry.unicode_version.major,
            emoji.entry.unicode_version.minor,
            "\t".repeat(indent_count),
            emoji.entry.emoji,
            "\t".repeat(indent_count),
            emoji.entry.name,
        )
        .unwrap();

        if let Some(ios) = &emoji.entry.ios_version {
            writeln!(
                code,
                "{}ios_version: Some({} {{ major: {}, minor: {} }}),",
                "\t".repeat(indent_count),
                struct_name::<Version>(),
                ios.major,
                ios.minor
            )
            .unwrap();
        } else {
            writeln!(code, "{}ios_version: None,", "\t".repeat(indent_count),).unwrap();
        }

        write!(code, "{}tags: &[", "\t".repeat(indent_count)).unwrap();
        for (i, tag) in emoji.entry.tags.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            write!(code, "\"{tag}\"").unwrap();
        }
        writeln!(code, "],").unwrap();

        write!(code, "{}aliases: &[", "\t".repeat(indent_count)).unwrap();
        for (i, alias) in emoji.entry.aliases.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            write!(code, "\"{alias}\"").unwrap();
        }
        writeln!(code, "],").unwrap();

        indent_count -= 1;
        writeln!(code, "{}}},", "\t".repeat(indent_count)).unwrap();

        writeln!(
            code,
            "{}skin_tones: {},",
            "\t".repeat(indent_count),
            emoji.skin_tones
        )
        .unwrap();

        if let Some(tone) = &emoji.skin_tone {
            writeln!(
                code,
                "{}skin_tone: Some({}::{:?}),",
                "\t".repeat(indent_count),
                struct_name::<SkinTone>(),
                tone
            )
            .unwrap();
        } else {
            writeln!(code, "{}skin_tone: None,", "\t".repeat(indent_count)).unwrap();
        }

        writeln!(code, "{}variations: &[],", "\t".repeat(indent_count)).unwrap();
        indent_count -= 1;
        writeln!(code, "{}}},", "\t".repeat(indent_count)).unwrap();
    }

    code.push_str("];\n");
    code
}
