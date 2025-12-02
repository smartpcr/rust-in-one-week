use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

pub enum CompResult {
    Equal,
    Greater,
    Less,
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

impl PartialEq for CompResult {
    fn eq(&self, other: &Self) -> bool {
        match self {
            CompResult::Less => match other {
                CompResult::Less => true,
                _ => false,
            },
            CompResult::Equal => match other {
                CompResult::Equal => true,
                _ => false,
            },
            CompResult::Greater => match other {
                CompResult::Greater => true,
                _ => false,
            },
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
    fn compare_to(&self, other: &Self) -> CompResult;
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
