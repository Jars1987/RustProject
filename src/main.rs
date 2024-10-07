use std::collections::HashMap;
use thiserror::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;


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

    fn add (&mut self, record: Record) {
        self.inner.insert(record.id, record);
    }

    fn into_vec (mut self) -> Vec<Record> {
        let mut records: Vec<_> = self.inner.drain().map(|kv| kv.1).collect();
        records.sort_by_key(|rec| rec.id);
        records
    }

    fn next_id (&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop(){
            Some(id) => id +1,
            None => 1,
        }
    }

    fn search_record (&self, name: &str) -> Vec<&Record> {
        self.inner
            .values()
            .filter(|rec| rec.name.to_lowercase().contains(&name.to_lowercase()))
            .collect()
    }

    fn remove (&mut self, id: i64) -> Option<Record> {
        self.inner.remove(&id)
    }

    fn edit (&mut self, id: i64, name: &str, email: Option<String>) -> Option<Record> {
        self.inner.insert(id, Record {
            id,
            name: name.to_string(),
            email
        })
    }
}

fn save_records (file_name: PathBuf, records:Records) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true) //when we truncate we erase all the data, but thats ok cause that has been loaded in "records"
        .open(file_name)?;

    // write accepts bites. Using "b" before the string will pass bytes instead of a string
    // So we are now writing the first sentence on the top of th file
    file.write(b"id,name,email\n")?;

    for record in records.into_vec().into_iter() {
        let email = match record.email {
            Some(email) => email,
            None => "".to_string(),
        };


        let line = format!("{},{},{}\n", record.id, record.name, email);
        file.write(line.as_bytes())?;
    };

    //flush wont return untill the data has been successfully written or it fails
    file.flush()?;
    Ok(())
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("id must be a number: {0}")]
    Invalid(#[from] std::num::ParseIntError),
    #[error("empty record")]
    EmptyRecord,
    #[error("missing field: {0}")]
    MissingField(String),
    
}

fn parse_record (record: &str) -> Result<Record, ParseError> {
    let fields: Vec<&str> = record.split(',').collect();
    let id = match fields.get(0){
        Some(id) => i64::from_str_radix(id, 10)?,
        None => return Err(ParseError::EmptyRecord),
    };
    let name = match fields.get(1).filter(|name| **name != "") {
        Some(name) => name.to_string(),
        None => return Err(ParseError::MissingField("name".to_owned()))
    };
    let email = fields.get(2).map(|email| email.to_string()).filter(|email| email != "");

    Ok(Record {id, name, email})
}

fn parse_records(records: String, verbose: bool) -> Records {
    let mut recs = Records::new();

    for (num, record) in records.split("\n").enumerate() {
        if record != "" {
            match parse_record(record){
                Ok(rec) => recs.add(rec),
                Err(e) => {
                    if verbose {
                         println!(
                            "error on line number {}: {}\n > \"{}\"\n", 
                            num+1,
                            e, 
                            record 
                        );
                    }
                }
            }
        }
    }
    recs
}

fn load_records(file_name: PathBuf, verbose: bool) -> std::io::Result<Records> {
    let mut file = File::open(file_name)?;
    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;
    
    Ok(parse_records(buffer, verbose))

}

#[derive(StructOpt, Debug)]
#[structopt(about = "project 2: contact manager")]
struct Opt {
    #[structopt(short, parse(from_os_str), default_value = "src/p2_data.csv")]
    data_file: PathBuf,
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(short, help = "verbose")]
    verbose: bool,
}

#[derive(StructOpt, Debug)]
//subcommands can't be regular enums, has to be struct enums
enum Command {
    List{},
    Add{
        name: String,
        email: Option<String>,
    },
    Search {
        query: String,
    },
    Remove {
        id: i64
    },
    Edit {
        id: i64,
        name: String,
        email: Option<String>,
    }
}

fn run(opt: Opt) -> Result<(), std::io::Error> {
    match opt.cmd {
        Command::List { .. } => {
            let recs = load_records(opt.data_file, opt.verbose)?;
            for record in recs.into_vec() {
                println!("{:?}", record);
            }
        }   ,
        Command::Add {name, email} => {
            let mut recs = load_records(opt.data_file.clone(), opt.verbose)?;
            let next_id = recs.next_id();
            recs.add(Record {id: next_id, name, email});
            save_records(opt.data_file, recs )?;
        }
        Command::Search { query } => {
            let recs = load_records(opt.data_file, opt.verbose)?;
            let results = recs.search_record(&query);
            if results.is_empty() {
                println!("No records found")
            } else {
                for rec in results { 
                    println!("{:?}", rec)
                }
            }
        }
        Command::Remove { id } => {
            let mut recs = load_records(opt.data_file.clone(), opt.verbose)?;
            if recs.remove(id).is_some(){
                save_records(opt.data_file, recs)?;
                println!("Record deleted")
            } else {
                println!("Record not found")
            }
        }

        Command::Edit { id, name, email } => {
            let mut recs = load_records(opt.data_file.clone(), opt.verbose)?;
            recs.edit(id, &name, email);
            save_records (opt.data_file, recs)?;
        }
    }
    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        println!("an error occurred: {}", e);
    }
}
