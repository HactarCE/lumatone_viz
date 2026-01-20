use std::path::Path;

use eyre::{Context, ContextCompat, bail};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Layout {
    pub boards: [Board; 5],
}

impl Layout {
    pub fn load_from_file(file: &Path) -> eyre::Result<Self> {
        let ini = ini::Ini::load_from_file(file).context("parsing layout file")?;
        Ok(Self {
            boards: (0..5)
                .map(|i| {
                    let section_name = format!("Board{i}");
                    Board::from_section(
                        ini.section(Some(&section_name))
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
    fn from_section(section: &ini::Properties) -> eyre::Result<Self> {
        Ok(Self {
            keys: (0..56)
                .map(|i| {
                    Ok(Key {
                        midi_note: section
                            .get(format!("Key_{i}"))
                            .wrap_err("missing midi note")?
                            .parse()?,
                        midi_chan: section
                            .get(format!("Chan_{i}"))
                            .wrap_err("missing midi channel")?
                            .parse()?,
                        color: parse_color(
                            section
                                .get(format!("Col_{i}"))
                                .wrap_err("missing note color")?,
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
