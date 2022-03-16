use ft_io::*;
use gstd::{msg, ActorId};

pub async fn transfer_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let _transfer_response: Event = msg::send_and_wait_for_reply(
        *token_id,
        Action::Transfer{
            from: *from,
            to: *to,
            amount
        },
        0,
    )
    .await
    .expect("Error in transfer");
}

