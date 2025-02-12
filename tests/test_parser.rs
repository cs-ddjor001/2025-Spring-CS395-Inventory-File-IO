use std::io::{BufRead, BufReader, Read};
use stringreader::StringReader;

use hamcrest2::prelude::*;
use rstest::{fixture, rstest};

use rust_inventory::prelude::*;

#[fixture]
fn test_items() -> [Item; 3] {
    [
        Item::new(0, String::from("Diamond Boots")),
        Item::new(1, String::from("Tomato")),
        Item::new(2, String::from("Unbreaking Gold Shovel")),
    ]
}

#[rstest]
fn test_parser_one_line(test_items: [Item; 3]) {
    let mut sreader = StringReader::new("0 Diamond Boots");

    let actual_items = Parser::read_items(BufReader::new(sreader));

    assert_that!(&actual_items, equal_to(&test_items[0..1]));
}

#[rstest]
fn test_parser_three_lines(test_items: [Item; 3]) {
    let mut sreader = StringReader::new("0 Diamond Boots\n1 Tomato\n2 Unbreaking Gold Shovel");

    let actual_items = Parser::read_items(BufReader::new(sreader));

    assert_that!(&actual_items[0], equal_to(&test_items[0]));
    assert_that!(&actual_items[1], equal_to(&test_items[1]));
    assert_that!(&actual_items[2], equal_to(&test_items[2]));
}
