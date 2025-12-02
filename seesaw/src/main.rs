mod seesaw;

fn main() {
    let size = 12;
    println!("creating sessaw of {} people", size);

    let result = seesaw::solve(size);
    match result {
        Ok(r) => {
            println!("found person #{} in {} steps", r.person_id, r.steps.len());
            for step in r.steps {
                println!("{}", step);
            }
        }
        Err(e) => println!("error: {}", e),
    }
}
