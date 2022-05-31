use gstd::{ActorId, msg};

pub fn assert_owner_message(owner: &ActorId, message: &str) {
    if msg::source() != *owner {
        panic!("{}: Not owner message", message)
    }
}

pub fn assert_not_zero(value: u128, message: &str) {
    if value == 0 {
        panic!("{}: Zero value", message)
    }
}