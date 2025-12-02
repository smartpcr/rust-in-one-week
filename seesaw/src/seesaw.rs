use crate::seesaw::group::Group;
use crate::seesaw::person::Person;
use crate::seesaw::traits::{CompResult, Comparable};

mod group;
mod person;
mod traits;

pub struct FindPersonResult {
    pub total_measurement: i8,
    pub found_person: bool,
    pub person_id: u8,
    pub steps: Vec<String>,
}

pub struct FindGroupResult {
    pub total_measurement: i8,
    pub found_group: bool,
    pub group_name: String,
    pub steps: Vec<String>,
}

pub fn solve(size: u8) -> Result<FindPersonResult, String> {
    let group = Group::new("new_group".to_string(), size);
    println!("created group: {}", group.to_string());

    let steps: Vec<String> = Vec::new();
    return solve_group(&group, 0, steps);
}

fn solve_group(
    g: &Group,
    prev_measurements: i8,
    steps: Vec<String>,
) -> Result<FindPersonResult, String> {
    if g.people.len() == 1 {
        return Ok(FindPersonResult {
            total_measurement: prev_measurements,
            found_person: true,
            person_id: g.people[0].id,
            steps,
        });
    } else if g.people.len() == 3 {
        let mut group_of3 = [Person::new(0, 0); 3];
        for i in 0..3 {
            group_of3[i] = g.people[i].clone();
        }
        let result = solve3Persons(&group_of3, prev_measurements, steps);
        return Ok(result);
    } else if g.people.len() == 4 {
        let mut group_of4 = [Person::new(0, 0); 4];
        for i in 0..4 {
            group_of4[i] = g.people[i].clone();
        }
        let result = solve4Persons(&group_of4, prev_measurements, steps);
        return Ok(result);
    } else {
        let (left, middle, right) = g.split()?;
        let mut procedure = steps.clone();
        procedure.push(format!("split group {} into 3 groups", g.name));
        // construct array of 3 Groups
        let group_of3 = vec![left, middle, right];
        let group_array = group_of3.as_slice();
        let rg3 = solve3Groups(group_array, prev_measurements, procedure.clone());
        procedure.append(&mut rg3.steps.clone());
        if rg3.found_group {
            let g = group_of3
                .iter()
                .find(|&x| x.name == rg3.group_name)
                .unwrap();
            let result = solve_group(&g, prev_measurements + rg3.total_measurement, procedure);
            return result;
        } else {
            return Err("failed".to_string());
        }
    }
}

fn solve3Groups(g3: &[Group], prev_measurements: i8, steps: Vec<String>) -> FindGroupResult {
    assert_eq!(g3.len(), 3);

    let r12 = g3[0].compare_to(&g3[1]);
    let r13 = g3[0].compare_to(&g3[2]);
    let mut procedure = Vec::new();
    procedure.extend(steps.clone());
    procedure.push(format!("{} {} {}", g3[0].name, r12, g3[1].name));
    procedure.push(format!("{} {} {}", g3[0].name, r13, g3[2].name));

    if r12 == CompResult::Equal {
        if r13 == CompResult::Equal {
            return FindGroupResult {
                total_measurement: 2 + prev_measurements,
                found_group: false,
                group_name: String::from(""),
                steps: procedure,
            };
        } else {
            return FindGroupResult {
                total_measurement: 2 + prev_measurements,
                found_group: true,
                group_name: g3[2].name.clone(),
                steps: procedure,
            };
        }
    } else {
        if r13 == CompResult::Equal {
            return FindGroupResult {
                total_measurement: 2 + prev_measurements,
                found_group: true,
                group_name: g3[1].name.clone(),
                steps: procedure,
            };
        } else {
            return FindGroupResult {
                total_measurement: 2 + prev_measurements,
                found_group: true,
                group_name: g3[0].name.clone(),
                steps: procedure,
            };
        }
    }
}

fn solve3Persons(p3: &[Person; 3], prev_measurements: i8, steps: Vec<String>) -> FindPersonResult {
    let r12 = p3[0].compare_to(&p3[1]);
    let r13 = p3[0].compare_to(&p3[2]);
    let mut procedure = Vec::new();
    procedure.extend(steps.clone());
    procedure.push(format!("{} {} {}", p3[0].id, r12, p3[1].id));
    procedure.push(format!("{} {} {}", p3[0].id, r13, p3[2].id));

    if r12 == CompResult::Equal {
        if r13 == CompResult::Equal {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: false,
                person_id: 0,
                steps: procedure,
            };
        } else {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p3[2].id,
                steps: procedure,
            };
        }
    } else {
        if r13 == CompResult::Equal {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p3[1].id,
                steps: procedure,
            };
        } else {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p3[0].id,
                steps: procedure,
            };
        }
    }
}

fn solve4Persons(p4: &[Person; 4], prev_measurements: i8, steps: Vec<String>) -> FindPersonResult {
    let r12 = p4[0].compare_to(&p4[1]);
    let r13 = p4[0].compare_to(&p4[2]);
    let mut procedure = Vec::new();
    procedure.extend(steps.clone());
    procedure.push(format!("{} {} {}", p4[0].id, r12, p4[1].id));
    procedure.push(format!("{} {} {}", p4[0].id, r13, p4[2].id));

    if r12 == CompResult::Equal {
        if r13 == CompResult::Equal {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p4[3].id,
                steps: procedure,
            };
        } else {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p4[2].id,
                steps: procedure,
            };
        }
    } else {
        if r13 == CompResult::Equal {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p4[1].id,
                steps: procedure,
            };
        } else {
            return FindPersonResult {
                total_measurement: 2 + prev_measurements,
                found_person: true,
                person_id: p4[0].id,
                steps: procedure,
            };
        }
    }
}

fn create_result(person_id: u8, total_count: i8, procedure: Vec<String>) -> FindPersonResult {
    FindPersonResult {
        total_measurement: total_count,
        found_person: true,
        person_id,
        steps: procedure,
    }
}
