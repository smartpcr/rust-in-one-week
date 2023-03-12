use std::fs;

mod args;

fn main() {
    let args = args::parse_args();

    let data = match fs::read_to_string(&args.filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let regex = regex::Regex::new(&args.target).unwrap();
    match fs::write(&args.output, regex.replace_all(&data, &args.replacement)) {
        Ok(_) => println!("Successfully replaced text in file"),
        Err(e) => {
            eprintln!("Error writing to file: {}", e);
            std::process::exit(1);
        }
    }

    println!("{:?}", args);
}
