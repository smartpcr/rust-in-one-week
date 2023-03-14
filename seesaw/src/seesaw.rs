pub struct SeesawResult {
    pub totalMeasurement: i8,
    pub foundPerson: bool,
    pub personId: u8,
    pub steps: Vec<String>,
}

pub fn solve(size: u8) -> Result<SeesawResult, E> {
    let island = Group::new(size);
    let (g1, g2, g3) = island::split();

    return Err("failed")
}

pub fn solve3(p3: &[Person; 3]) -> SeesawResult {
    let r12 = p3[0].cmp(&p3[1]);
    let r13 = p3[0].cmp(&p3[2]);
    let mut procedure = Vec::new();
    procedure.push(format!("{} {} {}", p3[0].id, r12, p3[1].id));
    procedure.push(format!("{} {} {}", p3[0].id, r13, p3[2].id));

    if r12 == Equal {
        if r13 == Equal {
            return SeesawResult {
                totalMeasurement: 2,
                foundPerson: false,
                personId: 0,
                steps: procedure
            };
        }
        else {
            return SeesawResult {
                totalMeasurement: 2,
                foundPerson: true,
                personId: p3[2].id,
                steps: procedure
            };
        }
    }
    else {
        if r13 == Equal {
            return SeesawResult {
                totalMeasurement: 2,
                foundPerson: true,
                personId: p3[1].id,
                steps: procedure
            };
        }
        else {
            return SeesawResult {
                totalMeasurement: 2,
                foundPerson: true,
                personId: p3[0].id,
                steps: procedure
            };
        }
    }
}
