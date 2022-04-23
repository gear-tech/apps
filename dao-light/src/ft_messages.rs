use ft_io::*;
use gstd::{msg, ActorId};

pub async fn transfer_from_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let _transfer_response: FTEvent = msg::send_and_wait_for_reply(
        *token_id,
        FTAction::Transfer {
            from: *from,
            to: *to,
            amount,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in transfer");
}

pub async fn transfer_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let _transfer_response: FTEvent = msg::send_and_wait_for_reply(
        *token_id,
        FTAction::Transfer {
            from: *from,
            to: *to,
            amount,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in transfer");
}

pub async fn approve_tokens(token_id: &ActorId, to: &ActorId, amount: u128) {
    let _approve_response: FTEvent =
        msg::send_and_wait_for_reply(*token_id, FTAction::Approve { to: *to, amount }, 0)
            .unwrap()
            .await
            .expect("Error in approve tokens");
}

pub async fn balance(token_id: &ActorId, account: &ActorId) -> u128 {
    let balance_response: FTEvent =
        msg::send_and_wait_for_reply(*token_id, FTAction::BalanceOf(*account), 0)
            .unwrap()
            .await
            .expect("Error in balance");
    if let FTEvent::Balance(balance_response) = balance_response {
        balance_response
    } else {
        0
    }
}
