use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::H256;
const GAS_RESERVE: u64 = 500_000_000;

pub async fn transfer_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let transfer_data = TransferData {
        from: H256::from_slice(from.as_ref()),
        to: H256::from_slice(to.as_ref()),
        amount,
    };

    let transfer_response: TokenEvent = msg::send_and_wait_for_reply(
        *token_id,
        TokenAction::Transfer(transfer_data),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .unwrap();
    if let TokenEvent::Transfer(transfer_response) = transfer_response {
        if transfer_response.amount != amount {
            panic!("error in transfer");
        }
    }
}

pub async fn balance(token_id: &ActorId, account: &ActorId) -> u128 {
    let balance_response: TokenEvent = msg::send_and_wait_for_reply(
        *token_id,
        TokenAction::BalanceOf(H256::from_slice(account.as_ref())),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .unwrap();
    if let TokenEvent::Balance(balance_response) = balance_response {
        balance_response
    } else {
        0
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum TokenEvent {
    Transfer(TransferData),
    Approval,
    TotalIssuance,
    Balance(u128),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum TokenAction {
    Mint,
    Burn,
    Transfer(TransferData),
    TransferFrom,
    Approve,
    IncreaseAllowance,
    DecreaseAllowance,
    TotalIssuance,
    BalanceOf(H256),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferData {
    pub from: H256,
    pub to: H256,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveData {
    pub owner: H256,
    pub spender: H256,
    pub amount: u128,
}
