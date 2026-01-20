use std::{collections::HashMap, path::Path};

use eyre::{Context, ContextCompat, bail};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Layout {
    pub boards: [Board; 5],
}

impl Layout {
    pub fn load_from_file(ltn_file: &Path) -> eyre::Result<Self> {
        let ltn_file_contents = std::fs::read_to_string(ltn_file).context("parsing layout file")?;
        let mut sections = HashMap::new();
        let mut section_name = "";
        let mut section = HashMap::new();
        for line in ltn_file_contents.lines() {
            if let Some(new_section_name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']'))
            {
                sections.insert(section_name, std::mem::take(&mut section));
                section_name = new_section_name;
            } else if let Some((k, v)) = line.split_once('=') {
                section.insert(k.trim(), v.trim());
            }
        }
        sections.insert(section_name, section);

        Ok(Self {
            boards: (0..5)
                .map(|i| {
                    let section_name = format!("Board{i}");
                    Board::from_section(
                        sections
                            .get(section_name.as_str())
                            .wrap_err_with(|| format!("missing section '{section_name}'"))?,
                    )
                })
                .collect::<eyre::Result<Vec<Board>>>()?
                .try_into()
                .unwrap(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    pub keys: [Key; 56],
}

impl Board {
    fn from_section(section: &HashMap<&str, &str>) -> eyre::Result<Self> {
        Ok(Self {
            keys: (0..56)
                .map(|i| {
                    Ok(Key {
                        midi_note: section
                            .get(format!("Key_{i}").as_str())
                            .wrap_err_with(|| format!("missing Key_{i}"))?
                            .parse()?,
                        midi_chan: section
                            .get(format!("Chan_{i}").as_str())
                            .wrap_err_with(|| format!("missing Chan_{i}"))?
                            .parse()?,
                        color: parse_color(
                            section
                                .get(format!("Col_{i}").as_str())
                                .wrap_err_with(|| format!("missing Col_{i}"))?,
                        )?,
                    })
                })
                .collect::<eyre::Result<Vec<Key>>>()?
                .try_into()
                .unwrap(),
        })
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub midi_note: u8,
    pub midi_chan: u8,
    pub color: [u8; 3],
}

fn parse_color(mut s: &str) -> eyre::Result<[u8; 3]> {
    // remove alpha
    if s.len() == 8 {
        s = &s[2..];
    }

    let chars: [char; 6] = s.trim().chars().collect_array().wrap_err("bad color")?;
    Ok([
        hex_digit_to_num(chars[0])? * 16 + hex_digit_to_num(chars[1])?,
        hex_digit_to_num(chars[2])? * 16 + hex_digit_to_num(chars[3])?,
        hex_digit_to_num(chars[4])? * 16 + hex_digit_to_num(chars[5])?,
    ])
}

fn hex_digit_to_num(c: char) -> eyre::Result<u8> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => bail!("bad hex digit {c}"),
    }
}
