#![no_std]

//use ft_io::*;
use gstd::{msg, ActorId};
use codec::{Decode, Encode};
use scale_info::TypeInfo;


//// The available  states during the escrow
enum State {
    PendingPayment,
    PendingDelivery,
    Finished,
}

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug, Default)]
struct Escrow {
    //Current escrow State
    current_state: State,
    //Buyer address
    buyer: ActorId,
    //Seller payable address, it'll receive funds after the sender makes a deposit
    seller: ActorId,
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
                from: ZERO_ID,
                to: msg::source(),
                balance,
            },
            0,
        );

        self.current_state = State::Finished;
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Deposit(u128),
    Confirm_delivery(u128),
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    let ft: Escrow =Escrow::default();
    match action {
        Action::Deposit(_) => {
            ft.deposit();
        }
    }
}
