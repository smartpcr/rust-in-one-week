use chrono::{DateTime, TimeZone, Utc};

pub(crate) struct Person {
  pub(crate) name: String,
  birth_date: DateTime<Utc>
}

impl Person {
  pub(crate) fn new(name: &String, birth_date: &DateTime<Utc>) -> Person {
    Person {
      name: name.to_string(),
      birth_date: birth_date.clone(),
    }
  }

  pub(crate) fn age(&self) -> u32 {
    let now = Utc::now();
    let duration = now.signed_duration_since(self.birth_date);
    duration.num_days() as u32 / 365
  }
}

mod tests {
  use super::*;

  #[test]
  fn age_should_be_15() {
    let birth_date: DateTime<Utc> = Utc.with_ymd_and_hms(2007, 11, 3, 0, 0, 0).unwrap();
    let person = Person::new(&"John".to_string(), &birth_date);
    assert_eq!(person.age(), 15);
  }
}
