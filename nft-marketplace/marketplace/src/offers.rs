use crate::{
    nft_messages::{nft_payouts, nft_transfer},
    payment::{check_attached_value, transfer_payment},
    Market,
};
use gstd::{exec, msg, prelude::*, ActorId};
use market_io::*;
use primitive_types::{H256, U256};

fn get_hash(nft_contract_id: &ActorId, ft_contract_id: Option<ActorId>, price: u128) -> H256 {
    let nft_conract_vec: Vec<u8> = <[u8; 32]>::from(*nft_contract_id).into();
    let price_vec: Vec<u8> = price.to_be_bytes().into();
    let ft_contract_vec: Vec<u8> = ft_contract_id
        .map(|id| <[u8; 32]>::from(id).into())
        .unwrap_or_default();
    sp_core_hashing::blake2_256(&[nft_conract_vec, price_vec, ft_contract_vec].concat()).into()
}

impl Market {
    /// Adds a price offer
    /// Requirements:
    /// * NFT item must be listed on the marketplace
    /// * There must be no an active auction
    /// * The user has to attach the value that is equal to the indicated price
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `ft_contract_id`: the FT contract address
    /// * `token_id`: the NFT id
    /// * `price`: the offer price
    pub async fn add_offer(
        &mut self,
        nft_contract_id: &ActorId,
        ft_contract_id: Option<ActorId>,
        token_id: U256,
        price: u128,
    ) {
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        self.check_approved_ft_contract(ft_contract_id);
        self.on_auction(&contract_and_token_id);
        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");
        if price == 0 {
            panic!("Cant offer zero price");
        }

        let hash: H256 = get_hash(nft_contract_id, ft_contract_id, price);
        let mut offers = item.offers.clone();
        if offers.iter().any(|offer| offer.hash == hash) {
            panic!("the offer with these params already exists");
        }

        check_attached_value(ft_contract_id, price);

        transfer_payment(&msg::source(), &exec::program_id(), ft_contract_id, price).await;

        offers.push(Offer {
            hash,
            id: msg::source(),
            ft_contract_id,
            price,
        });
        item.offers = offers;
        msg::reply(
            MarketEvent::OfferAdded {
                nft_contract_id: *nft_contract_id,
                ft_contract_id,
                token_id,
                price,
            },
            0,
        );
    }

    /// Accepts an offer
    /// Requirements:
    /// * NFT item must be listed on the marketplace
    /// * Only owner can accept offer
    /// * There must be no ongoing auction
    /// * The offer with indicated hash must exist
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `token_id`: the NFT id
    /// * `offer_hash`: the offer hash
    pub async fn accept_offer(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
        offer_hash: H256,
    ) {
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        self.on_auction(&contract_and_token_id);
        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");
        if item.owner_id != msg::source() {
            panic!("only owner can accept offer");
        }
        let mut offers = item.offers.clone();
        if let Some(offer) = offers.clone().iter().find(|offer| offer.hash == offer_hash) {
            let treasury_fee = offer.price * self.treasury_fee / 10_000u128;
            transfer_payment(
                &exec::program_id(),
                &self.treasury_id,
                offer.ft_contract_id,
                treasury_fee,
            )
            .await;
            let payouts =
                nft_payouts(nft_contract_id, &item.owner_id, offer.price - treasury_fee).await;
            for (account, amount) in payouts.iter() {
                transfer_payment(&exec::program_id(), account, offer.ft_contract_id, *amount).await;
            }
            nft_transfer(nft_contract_id, &offer.id, token_id).await;
            offers.retain(|offer| offer.hash != offer_hash);
            item.offers = offers;
            item.price = None;
            item.owner_id = offer.id;
            msg::reply(
                MarketEvent::OfferAccepted {
                    nft_contract_id: *nft_contract_id,
                    token_id,
                    new_owner: offer.id,
                    price: offer.price,
                },
                0,
            );
        } else {
            panic!("The offer with that hash does not exist");
        }
    }

    /// Withdraws tokens
    /// Requirements:
    /// * Only the offer creator can withdraw his tokens
    /// * The offer with indicated hash must exist
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `token_id`: the NFT id
    /// * `offer_hash`: the offer hash
    pub async fn withdraw(&mut self, nft_contract_id: &ActorId, token_id: U256, offer_hash: H256) {
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");

        let mut offers = item.offers.clone();
        if let Some(offer) = offers.clone().iter().find(|offer| offer.hash == offer_hash) {
            if msg::source() != offer.id {
                panic!("can't withdraw other user's tokens");
            }
            transfer_payment(
                &exec::program_id(),
                &msg::source(),
                offer.ft_contract_id,
                offer.price,
            )
            .await;
            offers.retain(|offer| offer.hash != offer_hash);
            item.offers = offers;
            msg::reply(
                MarketEvent::TokensWithdrawn {
                    nft_contract_id: *nft_contract_id,
                    token_id,
                    price: offer.price,
                },
                0,
            );
        } else {
            panic!("The offer with that hash does not exist");
        }
    }
}
