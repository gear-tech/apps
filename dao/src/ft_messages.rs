use fungible_token_messages::*;
use gstd::{msg, ActorId};
const GAS_RESERVE: u64 = 800_000_000;

pub async fn transfer_from_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let transfer_data = TransferFromInput {
        owner: *from,
        to: *to,
        amount,
    };

    let _transfer_response: Event = msg::send_and_wait_for_reply(
        *token_id,
        Action::TransferFrom(transfer_data),
        GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in transfer");
}

pub async fn transfer_tokens(token_id: &ActorId, to: &ActorId, amount: u128) {
    let transfer_data = TransferInput { to: *to, amount };
    let _transfer_response: Event =
        msg::send_and_wait_for_reply(*token_id, Action::Transfer(transfer_data), GAS_RESERVE, 0)
            .await
            .expect("Error in transfer");
}

pub async fn balance(token_id: &ActorId, account: &ActorId) -> u128 {
    let balance_response: Event =
        msg::send_and_wait_for_reply(*token_id, Action::BalanceOf(*account), GAS_RESERVE, 0)
            .await
            .expect("Error in balance");
    if let Event::Balance(balance_response) = balance_response {
        balance_response
    } else {
        0
    }
}
