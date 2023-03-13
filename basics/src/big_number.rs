struct big_number {
    number: u64,
}

pub trait Display {
    fn display(&self) -> String;
}

impl Display for big_number {
    fn display(&self) -> String {
        let string = format!("{:?}", self.number);
        string
    }
}
