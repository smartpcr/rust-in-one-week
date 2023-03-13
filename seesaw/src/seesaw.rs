use std::arch::asm;
use rand::Rng;

struct Person {
    id: u8,
    weight: u32,
}

impl Person {
    fn new(id: u8, weight: u32) -> Person {
        Person {
            id,
            weight,
        }
    }
}

fn  weigh(p1: &Person, p2: &Person) -> u8 {
    iif(p1.weight==p2.weight, 0, iif(p1.weight>p2.weight, 1, -1))
}

struct Island {
    people: Vec<Person>
}

struct Result {
    count: i8,
    personIdWithDifferentWeight: i8,
    unableToSolve: bool
}

// create instance of seesaw_people given size, only person's weight is eigher heavier or lighter
impl Island {
    fn new(size: u8) -> Island {
        let mut rng = rand::thread_rng();
        let weight: u32 = rng.gen_range(100..200);
        let person_with_different_weight = rng.gen_range(0..size);
        let different_weight: u32 = iif(
            rng.gen_bool(person_with_different_weight as f64),
            weight+5,
            weight-5);
        Island {
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

    fn create(slice: &[Person]) -> Island {
        Island {
            people: slice.to_vec()
        }
    }

    fn total_weight(&self) -> u32 {
        let mut sumWeight = 0;
        for person in self.people {
            sumWeight += person.weight;
        }
        sumWeight
    }

    fn weigh(&self, p2: Vec<Person>) -> u8 {
        iif(
            self.total_weight()==p2.total_weight(), 0,
            iif(self.total_weight>p2.total_weight(), 1, -1))
    }

    fn solve(size: u8) -> Result {
        if size % 3 != 0 {
            return Result {
                count: 0,
                personIdWithDifferentWeight: -1,
                unableToSolve: true
            }
        }

        let island = Island::new(size);
        let groupSize = size / 3;
        if  groupSize == 1 {
            let weigh11 = weigh(island.people&[0], island.people&[1]);
            return Result {
                count: 2,
                unableToSolve: false
            }
        }
        else {
            let leftSlice = island.people&[..groupSize];
            let leftGroup = Island::create(leftSlice);
            let middleSlice = island.people&[groupSize..groupSize+groupSize];
            return Result {
                count: 0,
                unableToSolve: true
            }
        }

    }
}

fn iif<T: Copy>(condition: bool, true_value: T, false_value: T) -> T {
    if condition {
        true_value
    } else {
        false_value
    }
}
