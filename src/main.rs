use serde_json::Value;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    let val1 = read_from_file("data/user1.json").unwrap();
    let val2 = read_from_file("data/user2.json").unwrap();
    //println!("{:?}", val1 == val2);

    let diff = compare(&val1, &val2, 0);
    println!("\n{:?}", diff);
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

//impl<'a> NodeDiff<'a> {
//    fn new(lhs: &'a Value, rhs: &'a Value) -> NodeDiff<'a> {
//        if lhs == rhs {
//            NodeDiff::Equal(lhs)
//        } else {
//            NodeDiff::DifferentContent((lhs, rhs))
//        }
//    }
//}

fn compare<'a>(val1: &'a Value, val2: &'a Value, depth: i32) -> NodeDiff<'a> {
    for _ in 0..depth {
        print!("\t");
    }
    let cmp = match (val1, val2) {
        (Value::Null, Value::Null) => NodeDiff::Equal(val1),
        (Value::Bool(b1), Value::Bool(b2)) => {
            if b1 == b2 {
                NodeDiff::Equal(val1)
            } else {
                NodeDiff::DifferentContent((val1, val2))
            }
        }
        (Value::Number(ref n1), Value::Number(ref n2)) => {
            if n1 == n2 {
                NodeDiff::Equal(val1)
            } else {
                NodeDiff::DifferentContent((val1, val2))
            }
        }
        (Value::String(ref s1), Value::String(ref s2)) => {
            if s1 == s2 {
                NodeDiff::Equal(val1)
            } else {
                NodeDiff::DifferentContent((val1, val2))
            }
        }
        (Value::Array(ref v1), Value::Array(ref v2)) => {
            if v1 == v2 {
                NodeDiff::Equal(val1)
            } else {
                NodeDiff::DifferentContent((val1, val2))
            }
        }
        (Value::Object(ref m1), Value::Object(ref m2)) => {
            println!("Object:");
            let mut nodes = HashMap::new();
            // iterate over the nodes of the first document
            for (k, v1) in m1.iter() {
                let v2 = m2.get(k);
                if v2.is_some() {
                    let diff = compare(v1, v2.unwrap(), depth + 1);
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
    };
    //println!("{:?}", cmp);
    cmp
}

fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Value, Box<Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u: Value = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(u)
}
