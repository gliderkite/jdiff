# jdiff
A simple and fast JSON comparison CLI utility.

[![Build Status](https://travis-ci.com/gliderkite/jdiff.svg?branch=master)](https://travis-ci.com/gliderkite/jdiff)


## Overview

`jdiff` is a basic JSON comparison utility written in `Rust` that, given two input
JSON files, outputs three different JSON files containing the delta (JSON node
differences) between the input files.
`jdiff` is built on top of [serde_json](https://docs.serde.rs/serde_json/index.html) 
for the JSON parsing and output serialization, allowing
[great performance](https://github.com/serde-rs/json-benchmark) (this utility
itself has been optimized for speed - if you need to parse huge files and
optimize for minimal memory usage this utility may not be for you).

The output of `jdiff` consist in three different files:
- `<output_prefix>_eq.json`: Contains only the JSON nodes that are equal between
    the two input files.
- `<output_prefix>_diff_ab.json`: Contains the JSON nodes that are different
    between the two input files (different value), as well as those nodes that
    exists in the fist file but are missing in the second.
- `<output_prefix>_diff_ba.json`: Contains the JSON nodes that are different
    between the two input files (different value), as well as those nodes that
    exists in the second file but are missing in the first.


### Examples

First input file:

```json
{
    "name": "John Doe",
    "banned": false,
    "age": 45,
    "profession": "Engineer",
    "address": [ "54 Old Street", "US" ],
    "user1": true
}
```

Second input file:

```json
{
    "name": "Rupert Wesley",
    "banned": true,
    "age": 45,
    "profession": "Engineer",
    "address": [ "3 Winston Road", "US" ],
    "user2": true
}
```

The output will be (the order of the nodes in the output does not follow any
specific order):

- Nodes with same values for same key `<output_prefix>_eq.json`:
```json
{
  "address": [
    "US"
  ],
  "age": 45,
  "profession": "Engineer"
}
```

- Nodes with different values between first and second or missing in second
    `<output_prefix>_diff_ab.json`:
```json
{
  "address": [
    [
      "54 Old Street",
      "3 Winston Road"
    ]
  ],
  "banned": [
    false,
    true
  ],
  "name": [
    "John Doe",
    "Rupert Wesley"
  ],
  "user1": true
}
```

- Nodes with different values between second and first or missing in first
    `<output_prefix>_diff_ba.json`:
```json
{
  "address": [
    [
      "3 Winston Road",
      "54 Old Street"
    ]
  ],
  "banned": [
    true,
    false
  ],
  "name": [
    "Rupert Wesley",
    "John Doe"
  ],
  "user2": true
}
```

## How to use

After building the project with `cargo build --release`, you will find the
executable in the `target/release` directory. You can run the utility with:

```bash
./jdiff <input1.json> <input2.json> <output_prefix>
```

If the input file are formatted correctly, the above command will generate the
output files: `output_prefix_eq.json`, `output_prefix_diff_ab.json` and
`output_prefix_diff_ba.json`.
