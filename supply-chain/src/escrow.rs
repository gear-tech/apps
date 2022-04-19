//! This module contains assestive functions for manipulating escrow contracts.

use escrow_io::{EscrowAction, EscrowEvent};
use gstd::{msg, ActorId};

pub async fn create(escrow_program_id: ActorId, seller: ActorId, amount: u128) -> u128 {
    let reply = msg::send_and_wait_for_reply(
        escrow_program_id,
        EscrowAction::Create {
            buyer: msg::source(),
            seller,
            amount,
        },
        0,
    )
    .unwrap()
    .await
    .unwrap();

    if let EscrowEvent::Created { contract_id } = reply {
        contract_id
    } else {
        panic!("EscrowEvent must be EscrowEvent::Created");
    }
}

pub async fn deposit(escrow_program_id: ActorId, contract_id: u128) -> (ActorId, u128) {
    let reply =
        msg::send_and_wait_for_reply(escrow_program_id, EscrowAction::Deposit { contract_id }, 0)
            .unwrap()
            .await
            .unwrap();

    if let EscrowEvent::Deposited { buyer, amount } = reply {
        (buyer, amount)
    } else {
        panic!("EscrowEvent must be EscrowEvent::Deposited");
    }
}

pub async fn confirm(escrow_program_id: ActorId, contract_id: u128) -> (ActorId, u128) {
    let reply =
        msg::send_and_wait_for_reply(escrow_program_id, EscrowAction::Confirm { contract_id }, 0)
            .unwrap()
            .await
            .unwrap();

    if let EscrowEvent::Confirmed { seller, amount } = reply {
        (seller, amount)
    } else {
        panic!("EscrowEvent must be EscrowEvent::Confirmed");
    }
}
