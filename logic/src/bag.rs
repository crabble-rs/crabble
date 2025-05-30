#[cfg(test)]
use crate::language::Language;
use crate::{language::Distribution, HandTile};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Debug)]
pub struct Bag(Vec<HandTile>);

impl Bag {
    pub fn empty() -> Self {
        Bag(Vec::new())
    }

    pub fn full(distribution: &Distribution) -> Self {
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
    let mut bag = Bag::full(&lang.distribution);

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
    let mut bag = Bag::full(&lang.distribution);

    let mut number_of_e = 0;
    while !bag.is_empty() {
        if let Some(HandTile::Letter('e')) = bag.take() {
            number_of_e += 1;
        }
    }

    assert_eq!(number_of_e, 18);
    assert_eq!(lang.values.get(HandTile::Letter('y')), 8);
}
