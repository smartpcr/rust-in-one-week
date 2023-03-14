use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use rand::Rng;
use crate::seesaw::person::Person;
use crate::seesaw::traits::{Comparable, CompResult, iif};

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub people: Vec<Person>
}

impl Group {
    pub fn new(name: String, size: u8) -> Group {
        let mut rng = rand::thread_rng();
        let weight: u32 = rng.gen_range(100..200);
        let person_with_different_weight = rng.gen_range(0..size);
        let different_weight: u32 = iif(
            weight % 2 == 0,
            weight + 5,
            weight - 5);
        Group {
            name,
            people: (0..size).map(|x|
                Person::new(
                    x + 1, // person id starts from 1
                    iif(
                        x == person_with_different_weight,
                        different_weight,
                        weight)))
                .collect()
        }
    }

    pub fn default() -> Group {
        Group {
            name: "default".to_string(),
            people: Vec::new()
        }
    }

    pub fn create(slice: &[Person]) -> Group {
        Group {
            name: format!("{}_{}", slice[0].id, slice[slice.len() - 1].id),
            people: slice.to_vec()
        }
    }

    pub fn total_weight(&self) -> u32 {
        let mut sum_weight = 0;
        for person in &(self.people) {
            sum_weight += person.weight;
        }
        sum_weight
    }

    pub fn split(&self) -> Result<(Group, Group, Group), String> {
        if self.people.len() <= 3 {
            return Err("group size too small".to_string());
        }

        let group_size = self.people.len() / 3;
        let left_group = Group::create(&(self.clone().people)[..group_size]);
        let middle_group = Group::create(&(self.clone().people)[group_size..group_size *2]);
        let right_group: Group;
        if self.people.len() % 3 != 0 {
            let range1 = &(self.clone().people[0..(group_size - self.people.len() % 3)]);
            let range2 = &(self.clone().people[group_size *2..]);
            right_group = Group::create(&[range1, range2].concat());
        }
        else {
            right_group = Group::create(&(self.clone().people)[group_size *2..]);
        }
        let right_group = Group::create(&(self.clone().people)[group_size *2..]);
        return Ok((left_group, middle_group, right_group));
    }
}

impl Comparable for Group {
    fn compare_to(&self, other: &Self) -> CompResult {
        CompResult::from(self.total_weight().cmp(&other.total_weight()))
    }
}

impl ToString for Group {
    fn to_string(&self) -> String {
        let mut result = format!("Group {} {{\n", self.name);
        for person in &(self.people) {
            result.push_str("\t");
            result.push_str(&person.to_string());
            result.push_str("\n");
        }
        result.push_str("}");
        result
    }
}
