use ft_io::*;
use gstd::{msg, ActorId};

/// Transfers `amount` tokens from `sender` account to `recipient` account.
/// Arguments:
/// * `from`: sender account
/// * `to`: recipient account
/// * `amount`: amount of tokens
pub async fn transfer_tokens(
    token_address: &ActorId,
    from: &ActorId,
    to: &ActorId,
    amount_tokens: u128,
) {
    msg::send_and_wait_for_reply::<FTEvent, _>(
        *token_address,
        FTAction::Transfer {
            from: *from,
            to: *to,
            amount: amount_tokens,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in transfer");
}
