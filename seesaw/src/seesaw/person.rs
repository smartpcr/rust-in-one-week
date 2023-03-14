use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use crate::seesaw::traits::{Comparable, CompResult, Equatable};

#[derive(Debug, Copy, Clone)]
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
        p1.compare_to(p2)
    }
}

impl Comparable for Person {
    fn compare_to(&self, other: &Self) -> CompResult {
        CompResult::from(self.weight.cmp(&other.weight))
    }
}

impl Equatable for Person {
    fn equals(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl ToString for Person {
    fn to_string(&self) -> String {
        format!("Person {{ id: {}, weight: {} }}", self.id, self.weight)
    }
}
