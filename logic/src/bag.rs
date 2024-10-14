use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, fmt::Display};

use crate::HandTile;

static LANGUAGE_DATA: &[(&str, &str)] = &[
    ("english", include_str!("../../data/english/letters.csv")),
    ("dutch", include_str!("../../data/dutch/letters.csv")),
];

struct Language {
    name: String,
    distribution: Distribution,
    values: LetterValues,
}

impl Language {
    pub fn by_name(lang: &str) -> Result<Self, ()> {
        let Some((_, data)) = LANGUAGE_DATA.iter().find(|(l, _)| *l == lang) else {
            return Err(());
        };
        Language::parse_csv(lang, data)
    }

    pub fn parse_csv(name: &str, csv: &str) -> Result<Self, ()> {
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
                return Err(());
            };

            let tile = if letter == " " {
                HandTile::Joker
            } else {
                let mut chars = letter.chars();
                let first = chars.next().ok_or(())?;
                if chars.next().is_some() {
                    return Err(());
                }
                HandTile::Letter(first)
            };

            let amount = amount.parse().map_err(|_| ())?;
            let value = value.parse().map_err(|_| ())?;

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

pub struct LetterValues(HashMap<HandTile, usize>);

impl LetterValues {
    fn get(&self, tile: HandTile) -> usize {
        self.0[&tile]
    }
}

pub struct Distribution(Vec<(HandTile, usize)>);

impl Distribution {
    fn iter(&self) -> impl Iterator<Item = (HandTile, usize)> + '_ {
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

pub struct Bag(Vec<HandTile>);

impl Bag {
    pub fn empty() -> Self {
        Bag(Vec::new())
    }

    pub fn full(distribution: Distribution) -> Self {
        let mut vec = Vec::new();
        for (tile, amount) in distribution.iter() {
            for _ in 0..amount {
                vec.push(tile);
            }
        }
        Self(vec)
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.0.shuffle(&mut rng);
    }

    pub fn take(&mut self) -> Option<HandTile> {
        self.0.pop()
    }

    pub fn put(&mut self, tile: HandTile) {
        self.0.push(tile);

        let mut rng = thread_rng();
        let idx = rng.gen_range(0..self.0.len());

        let final_idx = self.0.len() - 1;
        self.0.swap(idx, final_idx)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[test]
fn test_english() {
    let lang = Language::by_name("english").unwrap();
    let mut bag = Bag::full(lang.distribution);

    let mut number_of_e = 0;
    while !bag.is_empty() {
        if let Some(HandTile::Letter('e')) = bag.take() {
            number_of_e += 1;
        }
    }

    assert_eq!(number_of_e, 12);
    assert_eq!(lang.values.get(HandTile::Letter('q')), 10);
}

#[test]
fn test_dutch() {
    let lang = Language::by_name("dutch").unwrap();
    let mut bag = Bag::full(lang.distribution);

    let mut number_of_e = 0;
    while !bag.is_empty() {
        if let Some(HandTile::Letter('e')) = bag.take() {
            number_of_e += 1;
        }
    }

    assert_eq!(number_of_e, 18);
    assert_eq!(lang.values.get(HandTile::Letter('y')), 8);
}
