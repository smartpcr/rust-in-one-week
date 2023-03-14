use std::cmp::Ordering;
use rand::Rng;
use crate::seesaw::person::Person;
use crate::seesaw::traits::{Comparable, CompResult, iif};

#[derive(Debug, Clone)]
struct Group {
    pub people: Vec<person>
}

impl Group {
    pub fn new(size: u8) -> Group {
        let mut rng = rand::thread_rng();
        let weight: u32 = rng.gen_range(100..200);
        let person_with_different_weight = rng.gen_range(0..size);
        let different_weight: u32 = iif(
            rng.gen_bool(person_with_different_weight as f64),
            weight + 5,
            weight - 5);
        Group {
            people: (0..size).map(|x|
                Person::new(
                    x,
                    iif(
                        x == person_with_different_weight,
                        different_weight,
                        weight)))
                .collect()
        }
    }

    pub fn create(slice: &[person]) -> Group {
        Group {
            people: slice.to_vec()
        }
    }

    pub fn total_weight(&self) -> u32 {
        let mut sumWeight = 0;
        for person in self.people {
            sumWeight += person.weight;
        }
        sumWeight
    }

    pub fn split(&self) -> Result<(&Group, &Group, &Group), String> {
        if self.people.len() <= 3 {
            return Err("group size too small".to_string());
        }
        else if self.people.len() % 3 != 0 {
            return Err("group size should be multiples of 3".to_string());
        }
        let groupSize = self.people.len() / 3;
        let leftGroup = Group::create(self.clone().people&[..groupSize]);
        let middleGroup = Group::create(self.clone().people&[groupSize..groupSize*2]);
        let rightGroup = Group::create(self.clone().people&[groupSize*2..]);
        return Ok((&leftGroup, &middleGroup, &rightGroup));
    }
}

impl Comparable for Group {
    fn cmp(&self, other: &Self) -> CompResult {
        CompResult::from(self.total_weight().cmp(&other.total_weight()))
    }
}
