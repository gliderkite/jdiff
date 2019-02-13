use serde_json::Value;

use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Compare two JSON files.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    let val1 = parse_json(config.first_input)?;
    let val2 = parse_json(config.second_input)?;

    // compute the delta between the 2 JSON documents
    let delta = compare_values(&val1, &val2);

    // write differences
    delta.write_equal_set(config.output.to_string() + "_eq.json")?;
    delta.write_delta_to_second_set(config.output.to_string() + "_diff_ab.json")?;
    delta.write_delta_to_first_set(config.output.to_string() + "_diff_ba.json")?;

    Ok(())
}

/// Program configuration.
pub struct Config<'a> {
    first_input: &'a String,  // first input filename
    second_input: &'a String, // second input filename
    output: &'a String,       // prefix output filename
}

impl<'a> Config<'a> {
    /// Initializes the program configuration.
    pub fn new(args: &'a [String]) -> Result<Config<'a>, &'static str> {
        if args.len() < 4 {
            return Err("Invalid number of arguments: <input1> <input2> <output>");
        }
        Ok(Config {
            first_input: &args[1],
            second_input: &args[2],
            output: &args[3],
        })
    }
}

/// Parse a JSON file.
fn parse_json<P: AsRef<Path>>(path: P) -> Result<Value, Box<Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let val: Value = serde_json::from_reader(reader)?;
    Ok(val)
}

/// Represents the possible delta between two JSON nodes.
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
    /// Creates a new instance of `Delta` according to the given values.
    fn new(lhs: &'a Value, rhs: &'a Value) -> Delta<'a> {
        if lhs == rhs {
            Delta::Equal(lhs)
        } else {
            Delta::DifferentContent((lhs, rhs))
        }
    }

    /// Parse the given delta and filter according with the given filter logic.
    /// Returns a `Value` that can be serialized into a JSON file.
    fn to_value<F>(&self, filter_node: &F) -> Value
    where
        F: Fn(&Delta<'a>) -> Option<Value>,
    {
        match filter_node(self) {
            Some(val) => val,
            None => match self {
                Delta::List(list) => {
                    let mut array = Vec::with_capacity(list.len());
                    for delta in list.iter() {
                        let diff = delta.to_value(filter_node);
                        match diff {
                            Value::Null => (),
                            _ => array.push(diff),
                        }
                    }
                    if array.is_empty() {
                        Value::Null
                    } else {
                        Value::Array(array)
                    }
                }
                Delta::Map(map) => {
                    let mut object = serde_json::Map::with_capacity(map.len());
                    for (key, delta) in map.iter() {
                        let diff = delta.to_value(filter_node);
                        match diff {
                            Value::Null => (),
                            _ => {
                                object.insert(key.to_string(), diff);
                            }
                        }
                    }
                    if object.is_empty() {
                        Value::Null
                    } else {
                        Value::Object(object)
                    }
                }
                _ => Value::Null,
            },
        }
    }

    /// Writes the given delta to the given output file according to the given
    /// filter logic.
    fn write_delta<P, F>(&self, output: P, filter: &F) -> Result<(), Box<Error>>
    where
        P: AsRef<Path>,
        F: Fn(&Delta<'a>) -> Option<Value>,
    {
        let value = self.to_value(filter);
        let json = serde_json::to_string_pretty(&value)?;
        fs::write(output, json)?;
        Ok(())
    }

    /// Filter for only the nodes that are equal in both JSON documents, and writes
    /// the result into the given JSON output file.
    fn write_equal_set<P: AsRef<Path>>(&self, output: P) -> Result<(), Box<Error>> {
        // filter for the intersection set of the equal nodes
        let equal = |d: &Delta| -> Option<Value> {
            if let Delta::Equal(val) = d {
                Some((*val).clone())
            } else {
                None
            }
        };
        self.write_delta(output, &equal)
    }

    /// Filter for only the nodes that are different between the two JSON documents
    /// or missing in the second JSON document, and writes the result into the given
    /// JSON output file.
    fn write_delta_to_second_set<P: AsRef<Path>>(&self, output: P) -> Result<(), Box<Error>> {
        let delta_to_second = |d: &Delta| -> Option<Value> {
            match d {
                Delta::DifferentContent((v1, v2)) | Delta::DifferentVariant((v1, v2)) => {
                    Some(Value::Array(vec![(*v1).clone(), (*v2).clone()]))
                }
                Delta::MissingInSecond(v) => Some((*v).clone()),
                _ => None,
            }
        };
        self.write_delta(output, &delta_to_second)
    }

    /// Filter for only the nodes that are different between the two JSON documents
    /// or missing in the first JSON document, and writes the result into the given
    /// JSON output file.
    fn write_delta_to_first_set<P: AsRef<Path>>(&self, output: P) -> Result<(), Box<Error>> {
        let delta_to_first = |d: &Delta| -> Option<Value> {
            match d {
                Delta::DifferentContent((v1, v2)) | Delta::DifferentVariant((v1, v2)) => {
                    Some(Value::Array(vec![(*v2).clone(), (*v1).clone()]))
                }
                Delta::MissingInFirst(v) => Some((*v).clone()),
                _ => None,
            }
        };
        self.write_delta(output, &delta_to_first)
    }
}

/// Compare two JSON nodes.
fn compare_values<'a>(val1: &'a Value, val2: &'a Value) -> Delta<'a> {
    match (val1, val2) {
        (Value::Null, Value::Null) => Delta::Equal(val1),
        (Value::Bool(_), Value::Bool(_)) => Delta::new(val1, val2),
        (Value::Number(_), Value::Number(_)) => Delta::new(val1, val2),
        (Value::String(_), Value::String(_)) => Delta::new(val1, val2),
        (Value::Array(ref v1), Value::Array(ref v2)) => {
            // comparison where the "key" is the index of the nodes in the array
            let mut v = Vec::with_capacity(cmp::max(v1.len(), v2.len()));
            for (val1, val2) in v1.iter().zip(v2.iter()) {
                let diff = compare_values(val1, val2);
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
                    let diff = compare_values(v1, v2.unwrap());
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
        assert!(parse_json("test/data/basic_variants_eq_1a.json").is_ok());
        assert!(parse_json("test/data/basic_variants_eq_1b.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1a.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1b.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1c.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1d.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1e.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1f.json").is_ok());
        assert!(parse_json("test/data/basic_variants_diff_1g.json").is_ok());
        assert!(parse_json("test/data/nested_variants_eq_1a.json").is_ok());
        assert!(parse_json("test/data/nested_variants_eq_1b.json").is_ok());
        assert!(parse_json("test/data/nested_variants_eq_2a.json").is_ok());
        assert!(parse_json("test/data/nested_variants_eq_2b.json").is_ok());
        assert!(parse_json("test/data/nested_variants_diff_1a.json").is_ok());
        assert!(parse_json("test/data/nested_variants_diff_1b.json").is_ok());
        assert!(parse_json("test/data/nested_variants_diff_2a.json").is_ok());
        assert!(parse_json("test/data/nested_variants_diff_2b.json").is_ok());
    }

    #[test]
    fn cannot_read_invalid_json() {
        assert!(parse_json("test/data/does_not_exist.json").is_err());
        assert!(parse_json("test/data/invalid_user.json").is_err());
    }

    #[test]
    fn compare_same_basic_variants() {
        let node1a = parse_json("test/data/basic_variants_eq_1a.json").unwrap();
        let node1b = parse_json("test/data/basic_variants_eq_1b.json").unwrap();
        assert_eq!(node1a, node1b);
    }

    #[test]
    fn compare_different_basic_variants() {
        let mut nodes = Vec::new();
        nodes.push(parse_json("test/data/basic_variants_eq_1a.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1a.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1b.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1c.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1d.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1e.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1f.json").unwrap());
        nodes.push(parse_json("test/data/basic_variants_diff_1g.json").unwrap());

        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                assert_ne!(nodes[i], nodes[j]);
            }
        }
    }

    #[test]
    fn compare_same_nested_variants() {
        let node1a = parse_json("test/data/nested_variants_eq_1a.json").unwrap();
        let node1b = parse_json("test/data/nested_variants_eq_1b.json").unwrap();
        assert_eq!(node1a, node1b);

        let node2a = parse_json("test/data/nested_variants_eq_2a.json").unwrap();
        let node2b = parse_json("test/data/nested_variants_eq_2b.json").unwrap();
        assert_eq!(node2a, node2b);
    }

    #[test]
    fn compare_different_nested_variants() {
        let mut nodes = Vec::new();
        nodes.push(parse_json("test/data/nested_variants_eq_1a.json").unwrap());
        nodes.push(parse_json("test/data/nested_variants_diff_1a.json").unwrap());
        nodes.push(parse_json("test/data/nested_variants_diff_1b.json").unwrap());
        nodes.push(parse_json("test/data/nested_variants_diff_2a.json").unwrap());
        nodes.push(parse_json("test/data/nested_variants_diff_2b.json").unwrap());

        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                assert_ne!(nodes[i], nodes[j]);
            }
        }
    }
}
