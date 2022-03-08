use crate::{MessageToMarket};
use gstd::{msg, ActorId};
use primitive_types::{U256};
pub use market_io::*;

pub async fn list_nfts_on_market(market_id: &ActorId, owner: &ActorId, token_id: U256, msg: MessageToMarket) {
    let _market_response: MarketEvent = msg::send_and_wait_for_reply(
        *market_id,
        MarketAction::NFTContractCall{
            owner: *owner,
            token_id,
            msg,
        },
        0,
    )
    .await
    .expect("error in sending message to marketplace");
}

