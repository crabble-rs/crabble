use std::{collections::HashMap, fmt::Display};

use crate::{CrabbleError, HandTile};

static LANGUAGE_DATA: &[(&str, &str)] = &[
    ("english", include_str!("../../data/english/letters.csv")),
    ("dutch", include_str!("../../data/dutch/letters.csv")),
];

#[derive(Debug)]
pub struct Language {
    pub name: String,
    pub distribution: Distribution,
    pub values: LetterValues,
}

impl Language {
    pub fn by_name(lang: &str) -> Result<Self, CrabbleError> {
        let Some((_, data)) = LANGUAGE_DATA.iter().find(|(l, _)| *l == lang) else {
            return Err(CrabbleError::InvalidLanguage);
        };
        Language::parse_csv(lang, data)
    }

    pub fn parse_csv(name: &str, csv: &str) -> Result<Self, CrabbleError> {
        let mut vec = Vec::new();
        let mut values = HashMap::new();

        let mut lines = csv.lines();
        let _ = lines.next();

        for line in lines {
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split(',');
            let letter = parts.next();
            let amount = parts.next();
            let value = parts.next();

            let (Some(letter), Some(amount), Some(value)) = (letter, amount, value) else {
                return Err(CrabbleError::InvalidLanguage);
            };

            let tile = if letter == " " {
                HandTile::Joker
            } else {
                let mut chars = letter.chars();
                let first = chars.next().ok_or(CrabbleError::InvalidLanguage)?;
                if chars.next().is_some() {
                    return Err(CrabbleError::InvalidLanguage);
                }
                HandTile::Letter(first)
            };

            let amount = amount.parse().map_err(|_| CrabbleError::InvalidLanguage)?;
            let value = value.parse().map_err(|_| CrabbleError::InvalidLanguage)?;

            vec.push((tile, amount));
            values.insert(tile, value);
        }

        Ok(Language {
            name: name.into(),
            distribution: Distribution(vec),
            values: LetterValues(values),
        })
    }
}

#[derive(Debug)]
pub struct LetterValues(HashMap<HandTile, usize>);

impl LetterValues {
    pub fn get(&self, tile: HandTile) -> usize {
        self.0[&tile]
    }
}

#[derive(Debug)]
pub struct Distribution(Vec<(HandTile, usize)>);

impl Distribution {
    pub fn iter(&self) -> impl Iterator<Item = (HandTile, usize)> + '_ {
        self.0.iter().map(|(t, a)| (*t, *a))
    }
}

impl Display for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (tile, amount) in self.iter() {
            writeln!(f, "{tile:?} => {amount}")?;
        }
        Ok(())
    }
}
