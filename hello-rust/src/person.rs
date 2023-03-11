struct Person {
  name: String,
  birthDate: Date
}

impl Person {
  fn new(name: &String, birthDate: &Date) -> Person {
    Person {
      name: name.to_string(),
      birthDate: birthDate.clone(),
    }
  }

  fn age(&self) -> u32 {
    let now = Date::now();
    let mut age = now.year - self.birthDate.year;
    if now.month < self.birthDate.month {
      age -= 1;
    } else if now.month == self.birthDate.month {
      if now.day < self.birthDate.day {
        age -= 1;
      }
    }
    age
  }
}