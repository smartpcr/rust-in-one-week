struct BigNumber {
    number: u64,
}

pub trait Display {
    fn display(&self) -> String;
}

impl Display for BigNumber {
    fn display(&self) -> String {
        let string = format!("{:?}", self.number);
        string
    }
}
