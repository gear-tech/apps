pub mod utils;
use concert_io::*;
use gstd::Encode;
use gstd::String;
use gtest::{Program, System};
use utils::*;

#[test]
fn create_concert() {
    let system = init_system();
    let concert_program = init_concert(&system);

    create(concert_program, ERC1155_ID, USER, CONCERT_ID, NO_TICKETS);
}

#[test]
fn buy_tickets() {
    let system = init_system();
    let concert_program = init_concert(&system);

    create(concert_program, ERC1155_ID, USER, CONCERT_ID, NO_TICKETS);

    let metadata = vec![TokenMetadata {
        title: Some(String::from("SUM41_TORONTO")),
        title: Some(String::from("SUM 41 Torotno Ticket. Row 4. Seat 4.")),
        title: Some(String::from("sum41.com")),
        title: Some(String::from("UNKNOWN")),
    }];
    buy(concert_program, CONCERT_ID, AMOUNT, metadata);
}

#[test]
fn buy_tickets_failures() {
    let system = init_system();
    let concert_program = init_concert(&system);

    create(concert_program, ERC1155_ID, USER, CONCERT_ID, NO_TICKETS);
}

#[test]
fn hold_concert() {
    let system = init_system();
    let concert_program = init_concert(&system);

    create(concert_program, ERC1155_ID, USER, CONCERT_ID, NO_TICKETS);

    let metadata = vec![TokenMetadata {
        title: Some(String::from("SUM41_TORONTO")),
        title: Some(String::from("SUM 41 Torotno Ticket. Row 4. Seat 4.")),
        title: Some(String::from("sum41.com")),
        title: Some(String::from("UNKNOWN")),
    }];
    buy(concert_program, CONCERT_ID, AMOUNT, metadata);
    hold(concert_program, CONCERT_ID);
    // check for tokens
}
