use chrono::DateTime;
use hello_rust::person::Person;

#[test]
fn age_should_be_5() {
  let date = Date { year: 2015, month: 1, day: 1 };
  let person = Person::new(&"John".to_string(), &date);
  assert_eq!(person.age(), 5);
}