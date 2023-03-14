use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

pub enum CompResult {
    Equal,
    Greater,
    Less
}

impl From<Ordering> for CompResult {
    fn from(value: Ordering) -> Self {
        match value {
            Ordering::Less => CompResult::Less,
            Ordering::Greater => CompResult::Greater,
            Ordering::Equal => CompResult::Equal,
        }
    }
}

impl Display for CompResult {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            CompResult::Less => write!(f, "<"),
            CompResult::Equal => write!(f, "="),
            CompResult::Greater => write!(f, ">"),
        }
    }
}

pub trait Comparable {
    fn cmp(&self, other: &Self) -> CompResult;
}

pub trait Equatable {
    fn equals(&self, other: &Self) -> bool;
}

pub fn iif<T: Copy>(condition: bool, true_value: T, false_value: T) -> T {
    if condition {
        true_value
    } else {
        false_value
    }
}
