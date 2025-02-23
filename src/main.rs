use eyre::WrapErr;

use rust_inventory::parser::ParsedLine;
use rust_inventory::prelude::*;

fn main() -> eyre::Result<()> {
    let argv: Vec<String> = std::env::args().collect();

    if argv.len() < 3 {
        eyre::bail!("Usage: {} items_filename inventories_filename", argv[0]);
    }

    let all_items = Parser::read_from_file(&argv[1], |ins| Parser::read_items(ins))?;
    let all_inventory_lines =
        Parser::read_from_file(&argv[2], |ins| Parser::read_inventory_lines(ins))?;

    let logged_inventories = process_inventory_requests(all_inventory_lines, &all_items);

    println!("Processing Log:");
    for (entries, _) in logged_inventories.iter() {
        for entry in entries.iter() {
            println!("{}", entry);
        }
    }
    println!();

    println!("Item List:");
    for item in all_items.iter() {
        println!("  {:>2} {}", item.get_id(), item.get_name());
    }
    println!();

    println!("Storage Summary:");
    for (_, inv) in logged_inventories.iter() {
        println!("{}", inv);
    }

    Ok(())
}

/// # Refactoring Approach
/// 
/// My approach of refactoring the process_inventory_requests() function comes from the principles of SOLID,
/// TDD and functions should only do one thing. My goal was to reorganize this monolithic function into smaller
/// subfunctions that achieve a single, specific task, for the purposes of better maintainabilty, readbility and testability.
/// Note: I didn't write any new tests.
///
/// # Refactoring Justification
///
/// This function was refactored following the Single Responsibility Principle from SOLID.
/// Each extracted function performs a single, spefific subtask, improving code readability, maintainability, and testability.
///
/// - process_lines(): Separates the logic for identifying inventory boundaries.
/// - process_inventories(): Extracts inventory creation, ensuring it is isolated from processing item stacks.
/// - log_inventories(): Manages item processing and logging, delegating specific tasks to helper functions.
/// - process_stacks(): Encapsulates the logic for filtering and transforming `ParsedLine` entries into `ItemStack`s.
/// - process_entries(): Handles the logic for storing or discarding items, ensuring separation of concerns.
/// 
/// # Other Changes
/// 
/// - Inside process_lines(), replaced the match line ... with matches!() for brevity.
/// - Inside process_inventories() and process_stacks(), replaced the flat_map() with filter_map(). My 
/// - reasoning for this is that filter_map() filters out None values while transforming valid inputs, 
/// - making the intent clearer and avoiding unnecessary intermediate collections.
///
/// # Benefits of this Refactoring
/// - Improved Readability: Each function is short and focused, making the code easier to understand.
/// - Better Maintainability: Isolated concerns make modifying or extending functionality simpler.
/// - Enhanced Testability: Smaller functions are easier to test individually.
/// - Reduced Code Duplication: Extracting repeated logic into helper functions minimizes redundancy.
///
/// # Parameters
/// - `all_inventory_lines`: A vector of parsed inventory-related lines.
/// - `known_items`: A slice of known `Item`s to match against.
///
/// # Returns
/// A vector containing tuples of log entries and their corresponding `Inventory` instances.
pub fn process_inventory_requests(
    all_inventory_lines: Vec<ParsedLine>,
    known_items: &[Item],
) -> Vec<(Vec<String>, Inventory)> {
    let lines = process_lines(&all_inventory_lines);

    let inventories = process_inventories(&all_inventory_lines);

    log_inventories(known_items, lines, inventories)
}

fn process_lines(
    all_inventory_lines: &Vec<ParsedLine>,
) -> std::slice::Split<'_, ParsedLine, impl FnMut(&ParsedLine) -> bool> {
    all_inventory_lines.split(|line| matches!(line, ParsedLine::InventoryLine { .. }))
}

fn process_inventories(all_inventory_lines: &Vec<ParsedLine>) -> Vec<Inventory> {
    let inventories: Vec<Inventory> = all_inventory_lines
        .iter()
        .filter_map(|line| match line {
            ParsedLine::InventoryLine { max_size } => Some(Inventory::new(*max_size)),
            _ => None,
        })
        .collect();
    inventories
}

fn log_inventories(
    known_items: &[Item],
    lines: std::slice::Split<'_, ParsedLine, impl FnMut(&ParsedLine) -> bool>,
    inventories: Vec<Inventory>,
) -> Vec<(Vec<String>, Inventory)> {
    let logged_inventories: Vec<(_, Inventory)> = inventories
        .into_iter()
        .zip(lines.skip(1))
        .map(|(mut inv, entries)| {
            let stacks_to_store = process_stacks(known_items, entries);

            let entries = process_entries(stacks_to_store, &mut inv);

            (entries, inv)
        })
        .collect();

    logged_inventories
}

fn process_stacks(known_items: &[Item], entries: &[ParsedLine]) -> Vec<ItemStack> {
    let stacks_to_store: Vec<ItemStack> = entries
        .iter()
        .filter_map(|line| match line {
            ParsedLine::ItemStackLine { id, quantity } => known_items
                .iter()
                .find(|item| item.get_id() == *id)
                .map(|item| ItemStack::new(item.clone(), *quantity)),
            _ => None,
        })
        .collect();
    stacks_to_store
}

fn process_entries(stacks_to_store: Vec<ItemStack>, inv: &mut Inventory) -> Vec<String> {
    let entries: Vec<String> = stacks_to_store
        .into_iter()
        .map(|stack| {
            format!(
                "{:9} ({:>2}) {}",
                if inv.add_items(stack.clone()) {
                    "Stored"
                } else {
                    "Discarded"
                },
                stack.size(),
                stack.get_item().get_name()
            )
        })
        .collect();
    entries
}
