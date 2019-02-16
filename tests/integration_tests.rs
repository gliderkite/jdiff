use jdiff;
use serde_json::Value;

use std::fs;

#[test]
fn compare_same_file() {
    let args: Vec<String> = vec![
        String::new(),
        "tests/data/user1.json".to_string(),
        "tests/data/user1.json".to_string(),
        "tests/output1".to_string(),
    ];
    // run comparison
    let config = jdiff::Config::new(&args).unwrap();
    jdiff::run(config);
    // check output
    let user = jdiff::parse_json("tests/data/user1.json").unwrap();
    let equal = jdiff::parse_json("tests/output1_eq.json").unwrap();
    assert_eq!(equal, user);
    let diff_ab = jdiff::parse_json("tests/output1_diff_ab.json").unwrap();
    assert_eq!(Value::Null, diff_ab);
    let diff_ba = jdiff::parse_json("tests/output1_diff_ba.json").unwrap();
    assert_eq!(Value::Null, diff_ba);
    // cleanup
    fs::remove_file("tests/output1_eq.json").unwrap();
    fs::remove_file("tests/output1_diff_ab.json").unwrap();
    fs::remove_file("tests/output1_diff_ba.json").unwrap();
}

#[test]
fn compare_different_file() {
    let args: Vec<String> = vec![
        String::new(),
        "tests/data/user1.json".to_string(),
        "tests/data/user2.json".to_string(),
        "tests/output2".to_string(),
    ];
    // run comparison
    let config = jdiff::Config::new(&args).unwrap();
    jdiff::run(config);
    // check output
    let original = jdiff::parse_json("tests/data/user1_user2_eq.json").unwrap();
    let equal = jdiff::parse_json("tests/output2_eq.json").unwrap();
    assert_eq!(equal, original);
    let original = jdiff::parse_json("tests/data/user1_user2_diff.json").unwrap();
    let equal = jdiff::parse_json("tests/output2_diff_ab.json").unwrap();
    assert_eq!(equal, original);
    let original = jdiff::parse_json("tests/data/user2_user1_diff.json").unwrap();
    let equal = jdiff::parse_json("tests/output2_diff_ba.json").unwrap();
    assert_eq!(equal, original);
    // cleanup
    fs::remove_file("tests/output2_eq.json").unwrap();
    fs::remove_file("tests/output2_diff_ab.json").unwrap();
    fs::remove_file("tests/output2_diff_ba.json").unwrap();
}
