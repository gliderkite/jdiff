use serde_json::Value;

use std::cmp;
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
enum Delta<'a> {
    Equal(&'a Value),
    DifferentContent((&'a Value, &'a Value)),
    DifferentVariant((&'a Value, &'a Value)),
    MissingInSecond(&'a Value),
    MissingInFirst(&'a Value),
    List(Vec<Delta<'a>>),
    Map(HashMap<&'a String, Delta<'a>>),
}

impl<'a> Delta<'a> {
    fn new(lhs: &'a Value, rhs: &'a Value) -> Delta<'a> {
        if lhs == rhs {
            Delta::Equal(lhs)
        } else {
            Delta::DifferentContent((lhs, rhs))
        }
    }
}

/// Compare two JSON nodes.
fn compare<'a>(val1: &'a Value, val2: &'a Value) -> Delta<'a> {
    match (val1, val2) {
        (Value::Null, Value::Null) => Delta::Equal(val1),
        (Value::Bool(_), Value::Bool(_)) => Delta::new(val1, val2),
        (Value::Number(_), Value::Number(_)) => Delta::new(val1, val2),
        (Value::String(_), Value::String(_)) => Delta::new(val1, val2),
        (Value::Array(ref v1), Value::Array(ref v2)) => {
            // compare according to the index of the nodes in the array
            let mut v = Vec::with_capacity(cmp::max(v1.len(), v2.len()));
            for (val1, val2) in v1.iter().zip(v2.iter()) {
                let diff = compare(val1, val2);
                v.push(diff);
            }
            let missing_in_second = v1.len() > v2.len();
            let it = if missing_in_second {
                v1.iter().skip(v2.len())
            } else {
                v2.iter().skip(v1.len())
            };
            for val in it {
                if missing_in_second {
                    v.push(Delta::MissingInSecond(val));
                } else {
                    v.push(Delta::MissingInFirst(val));
                }
            }
            Delta::List(v)
        }
        (Value::Object(ref m1), Value::Object(ref m2)) => {
            // compare according to the key of the nodes in the map
            let mut nodes = HashMap::new();
            // iterate over the nodes of the first document
            for (k, v1) in m1.iter() {
                let v2 = m2.get(k);
                if v2.is_some() {
                    let diff = compare(v1, v2.unwrap());
                    nodes.insert(k, diff);
                } else {
                    nodes.insert(k, Delta::MissingInSecond(v1));
                }
            }
            // iterate over the nodes of the second document
            for (k, v2) in m2.iter() {
                if !m1.contains_key(k) {
                    nodes.insert(k, Delta::MissingInFirst(v2));
                }
            }
            Delta::Map(nodes)
        }
        _ => Delta::DifferentVariant((val1, val2)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_valid_json() {
        assert!(read_from_file("test/data/basic_variants_eq_1a.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_eq_1b.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1a.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1b.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1c.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1d.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1e.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1f.json").is_ok());
        assert!(read_from_file("test/data/basic_variants_diff_1g.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_eq_1a.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_eq_1b.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_eq_2a.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_eq_2b.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_diff_1a.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_diff_1b.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_diff_2a.json").is_ok());
        assert!(read_from_file("test/data/nested_variants_diff_2b.json").is_ok());
    }

    #[test]
    fn cannot_read_invalid_json() {
        assert!(read_from_file("test/data/does_not_exist.json").is_err());
        assert!(read_from_file("test/data/invalid_user.json").is_err());
    }

    #[test]
    fn compare_same_basic_variants() {
        let node1a = read_from_file("test/data/basic_variants_eq_1a.json").unwrap();
        let node1b = read_from_file("test/data/basic_variants_eq_1b.json").unwrap();
        assert_eq!(node1a, node1b);
    }

    #[test]
    fn compare_different_basic_variants() {
        let mut nodes = Vec::new();
        nodes.push(read_from_file("test/data/basic_variants_eq_1a.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1a.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1b.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1c.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1d.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1e.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1f.json").unwrap());
        nodes.push(read_from_file("test/data/basic_variants_diff_1g.json").unwrap());

        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                assert_ne!(nodes[i], nodes[j]);
            }
        }
    }

    #[test]
    fn compare_same_nested_variants() {
        let node1a = read_from_file("test/data/nested_variants_eq_1a.json").unwrap();
        let node1b = read_from_file("test/data/nested_variants_eq_1b.json").unwrap();
        assert_eq!(node1a, node1b);

        let node2a = read_from_file("test/data/nested_variants_eq_2a.json").unwrap();
        let node2b = read_from_file("test/data/nested_variants_eq_2b.json").unwrap();
        assert_eq!(node2a, node2b);
    }

    #[test]
    fn compare_different_nested_variants() {
        let mut nodes = Vec::new();
        nodes.push(read_from_file("test/data/nested_variants_eq_1a.json").unwrap());
        nodes.push(read_from_file("test/data/nested_variants_diff_1a.json").unwrap());
        nodes.push(read_from_file("test/data/nested_variants_diff_1b.json").unwrap());
        nodes.push(read_from_file("test/data/nested_variants_diff_2a.json").unwrap());
        nodes.push(read_from_file("test/data/nested_variants_diff_2b.json").unwrap());

        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                assert_ne!(nodes[i], nodes[j]);
            }
        }
    }
}
