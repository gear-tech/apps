use crate::*;
use gstd::{exec, msg, prelude::*, ActorId};

pub async fn get_root_owner(to: &ActorId, token_id: TokenId) -> ActorId {
    let response: RMRKEvent =
        msg::send_and_wait_for_reply(*to, RMRKAction::RootOwner { token_id }, 0)
            .unwrap()
            .await
            .expect("Error in message to nft contract");

    if let RMRKEvent::RootOwner { root_owner } = response {
        return root_owner;
    } else {
        panic!("wrong received message");
    }
}

pub async fn add_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
)  {
    let _response: RMRKEvent = msg::send_and_wait_for_reply(
        *parent_contract_id,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in message to nft contract");

}

pub async fn burn_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) -> ChildStatus {
    let response: RMRKEvent = msg::send_and_wait_for_reply(
        *parent_contract_id,
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in message to nft contract");

    if let RMRKEvent::ChildBurnt {  child_status, .. } = response {
        return child_status;
    } else {
        panic!("wrong received message");
    }
}
