mod gcd;
mod person;

use chrono::{DateTime, TimeZone, Utc};
use ferris_says::say;
use gcd::gcd;
use person::Person;
use std::io::{stdout, BufWriter};

fn main() {
    let stdout = stdout();
    let message = String::from("Hello from rust");
    let width = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(&message, width, &mut writer).unwrap();

    let result = gcd(14, 15);
    println!("The GCD of 14 and 15 is {}", result);

    let birth_date: DateTime<Utc> = Utc.with_ymd_and_hms(2007, 11, 3, 0, 0, 0).unwrap();
    let person = Person::new(&"John".to_string(), &birth_date);
    println!("The age of {} is {}", person.name, person.age());
}
