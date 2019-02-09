use serde_json::Value;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Compare two JSON files.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    let val1 = read_from_file(config.first_input)?;
    let val2 = read_from_file(config.second_input)?;

    let diff = compare(&val1, &val2);
    println!("{:?}", diff);
    Ok(())
}

/// Program configuration.
pub struct Config<'a> {
    first_input: &'a String,  // first input filename
    second_input: &'a String, // second input filename
}

impl<'a> Config<'a> {
    /// Initializes the program configuration.
    pub fn new(args: &'a [String]) -> Result<Config<'a>, &'static str> {
        if args.len() < 3 {
            return Err("Invalid number of arguments: <input1> <input2>");
        }
        Ok(Config {
            first_input: &args[1],
            second_input: &args[2],
        })
    }
}

/// Parse a JSON file.
fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Value, Box<Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let val: Value = serde_json::from_reader(reader)?;
    Ok(val)
}

#[derive(Debug)]
enum NodeDiff<'a> {
    Equal(&'a Value),
    DifferentContent((&'a Value, &'a Value)),
    DifferentVariant((&'a Value, &'a Value)),
    MissingInSecond(&'a Value),
    MissingInFirst(&'a Value),
    Node(HashMap<&'a String, NodeDiff<'a>>),
}

impl<'a> NodeDiff<'a> {
    fn new(lhs: &'a Value, rhs: &'a Value) -> NodeDiff<'a> {
        if lhs == rhs {
            NodeDiff::Equal(lhs)
        } else {
            NodeDiff::DifferentContent((lhs, rhs))
        }
    }
}

/// Compare two JSON nodes.
fn compare<'a>(val1: &'a Value, val2: &'a Value) -> NodeDiff<'a> {
    match (val1, val2) {
        (Value::Null, Value::Null) => NodeDiff::Equal(val1),
        (Value::Bool(_), Value::Bool(_)) => NodeDiff::new(val1, val2),
        (Value::Number(_), Value::Number(_)) => NodeDiff::new(val1, val2),
        (Value::String(_), Value::String(_)) => NodeDiff::new(val1, val2),
        (Value::Array(_), Value::Array(_)) => NodeDiff::new(val1, val2),
        (Value::Object(ref m1), Value::Object(ref m2)) => {
            let mut nodes = HashMap::new();
            // iterate over the nodes of the first document
            for (k, v1) in m1.iter() {
                let v2 = m2.get(k);
                if v2.is_some() {
                    let diff = compare(v1, v2.unwrap());
                    nodes.insert(k, diff);
                } else {
                    nodes.insert(k, NodeDiff::MissingInSecond(v1));
                }
            }
            // iterate over the nodes of the second document
            for (k, v2) in m2.iter() {
                if !m1.contains_key(k) {
                    nodes.insert(k, NodeDiff::MissingInFirst(v2));
                }
            }
            NodeDiff::Node(nodes)
        }
        _ => NodeDiff::DifferentVariant((val1, val2)),
    }
}
