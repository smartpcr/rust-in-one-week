use std::cmp::Ordering;
use crate::seesaw::traits::{Comparable, CompResult, Equatable};

#[derive(Debug, Clone)]
pub struct Person {
    pub id: u8,
    pub weight: u32,
}

impl Person {
    pub fn new(id: u8, weight: u32) -> Person {
        Person {
            id,
            weight,
        }
    }

    pub fn weigh(p1: Person, p2: &Person) -> CompResult {
        p1.cmp(p2)
    }
}

impl Comparable for Person {
    fn cmp(&self, other: &Self) -> CompResult {
        CompResult::from(self.weight.cmp(&other.weight))
    }
}

impl Equatable for Person {
    fn equals(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
