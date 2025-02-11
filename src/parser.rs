use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use crate::error::*;
use crate::inventory::Inventory;
use crate::items::{Item, ItemStack};

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedLine {
    ItemStackLine { id: u64, quantity: usize },
    InventoryLine { max_size: usize },
    CommentLine { raw_line: String },
    InvalidLine { raw_line: String },
}

pub struct Parser;

impl Parser {
    /// Open a file and read in data based on a supplied closure
    ///
    /// # Arguments
    ///
    ///   * `filename` - file from which to read
    ///   * `parse_fn` - parsing function to use
    pub fn read_from_file<T, F>(filename: &str, parse_fn: F) -> Result<T, ParserError>
    where
        F: Fn(BufReader<File>) -> T,
    {
        let file = File::open(filename)?;
        let ins = BufReader::new(file);
        let all_things = parse_fn(ins);

        Ok(all_things)
    }

    /// Read Items from an input buffer.
    ///
    /// # Arguments
    ///
    ///  * `ins` - input source
    ///
    pub fn read_items<B: BufRead>(ins: B) -> Vec<Item> {
        ins.lines()
            .flatten()
            .map(|line| {
                line.splitn(2, ' ')
                    .map(|token| token.trim().to_owned())
                    .collect::<Vec<_>>()
            })
            .map(|tokens| {
                let id = tokens[0].parse::<u64>().unwrap_or(0);
                let name = tokens[1].to_owned();

                (id, name)
            })
            .map(|(id, name)| Item::new(id, name))
            .collect()
    }

    /// Read inventories from an input buffer.
    ///
    /// # Arguments
    ///
    ///  * `ins` - input source
    ///
    pub fn read_inventory_lines<B: BufRead>(ins: B) -> Vec<ParsedLine> {
        ins.lines()
            .flatten()
            .map(|line| -> Vec<String> {
                line.trim().split_whitespace().map(String::from).collect()
            })
            .map(|tokens| match tokens[0].as_ref() {
                "#" => match tokens[1].parse() {
                    Ok(max_size) => ParsedLine::InventoryLine { max_size },
                    Err(_) => ParsedLine::InvalidLine {
                        raw_line: tokens.join(" "),
                    },
                },
                "-" => match (tokens[1].parse(), tokens[2].parse()) {
                    (Ok(id), Ok(quantity)) => ParsedLine::ItemStackLine { id, quantity },
                    _ => ParsedLine::InvalidLine {
                        raw_line: tokens.join(" "),
                    },
                },
                "//" => {
                    let line = tokens.join(" ");
                    ParsedLine::CommentLine { raw_line: line }
                }
                _ => {
                    let line = tokens.join(" ");
                    ParsedLine::InvalidLine { raw_line: line }
                }
            })
            .collect::<Vec<_>>()
    }
}
