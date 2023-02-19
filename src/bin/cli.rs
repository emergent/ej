use std::{error::Error, fmt};

#[derive(Debug)]
enum MyError {
    ArgNotFound,
    FileNotFound,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MyError {}

fn main() -> Result<(), MyError> {
    let args = std::env::args();

    let Some(filename) = args.into_iter().nth(1) else {
        return Err(MyError::ArgNotFound);
    };

    let json_string = std::fs::read_to_string(filename).map_err(|_| MyError::FileNotFound)?;
    match ej::from_json_str(&json_string) {
        Ok(res) => println!("{:#?}", res),
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}
