use std::collections::HashMap;
use thiserror::Error;


#[derive(Debug)]
struct Record {
    id: i64,
    name: String,
    email: Option<String>,
}   

#[derive(Debug)]
struct Records {
    inner: HashMap<i64, Record>
}   

impl Records {  
    fn new()-> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn add (mut self, record: Record) {
        self.inner.insert(record.id, record);
    }
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("id must be a number: {0}")]
    Invalid(#[from] std::num::ParseIntError),
    #[error("empty record")]
    EmptyRecord,
    #[error("missing field: {0}")]
    Missing(String),
    
}

fn load_records(file_name: PathBuff, verbose: bool) -> std::io::Result<Records> {
    let mut file = File::open(file_name);
    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;
    
    ok(parse_records(buffer, verbose))

}


fn main() {
    println!("Hello, world!");
}
