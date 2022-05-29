use crate::*;
use gstd::{msg, ActorId};

pub async fn get_root_owner(to: &ActorId, token_id: TokenId) -> ActorId {
    let response: RMRKEvent =
        msg::send_and_wait_for_reply(*to, RMRKAction::RootOwner { token_id }, 0)
            .unwrap()
            .await
            .expect("Error in message to nft contract");

    if let RMRKEvent::RootOwner { root_owner } = response {
        root_owner
    } else {
        panic!("wrong received message");
    }
}

pub async fn add_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) -> ActorId {
    let response: RMRKEvent = msg::send_and_wait_for_reply(
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
    if let RMRKEvent::PendingChild { root_owner, .. } = response {
        return root_owner;
    } else {
        panic!("Wrong received message");
    }
}

pub async fn burn_from_parent(
    child_contract_id: &ActorId,
    child_token_ids: Vec<TokenId>,
    root_owner: &ActorId,
) {
    let _response: RMRKEvent = msg::send_and_wait_for_reply(
        *child_contract_id,
        RMRKAction::BurnFromParent {
            child_token_ids,
            root_owner: *root_owner,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in message to burning RMRK token");
}

pub async fn burn_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) {
    let _response: RMRKEvent = msg::send_and_wait_for_reply(
        *parent_contract_id,
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in burning RMRK token child");
}

pub async fn transfer_child(
    parent_contract_id: &ActorId,
    from: TokenId,
    to: TokenId,
    child_token_id: TokenId,
) {
    let _response: RMRKEvent = msg::send_and_wait_for_reply(
        *parent_contract_id,
        RMRKAction::TransferChild {
            from,
            to,
            child_token_id,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in burning RMRK token child");
}

pub async fn add_accepted_child(
    parent_contract_id: &ActorId,
    parent_token_id: TokenId,
    child_token_id: TokenId,
) {
    let _response: RMRKEvent = msg::send_and_wait_for_reply(
        *parent_contract_id,
        RMRKAction::AddAcceptedChild {
            parent_token_id,
            child_token_id,
        },
        0,
    )
    .unwrap()
    .await
    .expect("Error in burning RMRK token child");
}
