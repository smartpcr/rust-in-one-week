use std::env;
use text_colorizer::Colorize;

#[derive(Debug)]
pub struct Arguments {
    pub target: String,
    pub replacement: String,
    pub filename: String,
    pub output: String,
}

impl Arguments {
    pub fn new() -> Result<Arguments, &'static str> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 4 {
            eprintln!("{} - not enough arguments", args[0].green());
            eprintln!("Usage: {} {} {} {} {} {}",
                args[0].green(),
                "target".yellow(),
                "replacement".yellow(),
                "filename".yellow(),
                "output".yellow(),
                "[optional]".blue()
            );
            return Err("not enough arguments");
        }

        let target = args[1].clone();
        let replacement = args[2].clone();
        let filename = args[3].clone();

        let output = if args.len() == 5 {
            args[4].clone()
        } else {
            filename.clone()
        };

        Ok(Arguments {
            target,
            replacement,
            filename,
            output,
        })
    }
}

pub fn parse_args() -> Arguments {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 4 {
        eprintln!("{} - not enough arguments", "quick-replace".green());
        eprintln!("Usage: {} {} {} {} {} {}",
                  "quick-replace".green(),
                  "target".yellow(),
                  "replacement".yellow(),
                  "filename".yellow(),
                  "output".yellow(),
                  "[optional]".blue()
        );
        std::process::exit(1);
    }

    Arguments {
        target: args[0].clone(),
        replacement: args[1].clone(),
        filename: args[2].clone(),
        output: args[3].clone(),
    }
}
