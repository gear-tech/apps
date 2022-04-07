#![no_std]

use ft_io::*;
use gstd::{msg, ActorId};

//// The available  states during the escrow
enum State {
    PendingPayment,
    PendingDelivery,
    Finished,
}

//Default status for State enum data type
impl Default for State {
    fn default() -> Self {
        Self::PendingPayment
    }
}

#[derive(Default)]
struct Escrow {
    //Current escrow State
    current_state: State,
    //Buyer address
    buyer: ActorId,
    //Seller payable address, it'll receive funds after the sender makes a deposit
    seller: ActorId,
    //Amount to transfer
    amount: u128,
}

impl Escrow {
    //Validates the sender is the same than buyer
    fn only_buyer(&mut self) -> bool {
        return msg::source() != self.buyer;
    }

    //The buyer makes a deposit only when a payment is pending
    //and the seller receives the funds
    fn deposit(&mut self) {
        if matches!(self.current_state, State::PendingPayment) {
            panic!("Already Payed");
        }
        if !self.only_buyer() {
            panic!("Only buyer is able to call this function");
        }
        self.current_state = State::PendingDelivery;
    }

    //The buyer confirms the delivery only when the delivery is
    //pending, the balance is transfered to the seller
    //and  the  transaction is finished.
    fn confirm_delivery(&mut self) {
        if matches!(self.current_state, State::PendingDelivery) {
            panic!("Cannot confirm delivery");
        }
        if !self.only_buyer() {
            panic!("Only buyer is able to call this function");
        }

        msg::reply(
            Event::Transfer {
                from: self.buyer,
                to: msg::source(),
                amount: self.amount,
            },
            0,
        );

        self.current_state = State::Finished;
    }
}

