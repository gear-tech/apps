use codec::Encode;
use ft_io::*;
use gstd::ActorId;
use gtest::{Program, System};
use market_io::*;
use nft_io::*;
use sp_core::{hashing, H256 as spH256};
pub const USERS: &'static [u64] = &[4, 5, 6, 7];
pub const TREASURY_ID: u64 = 8;

pub fn init_ft(sys: &System) {
    let ft = Program::from_file(
        &sys,
        "../../fungible-token/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft.send(
        USERS[0],
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );

    assert!(res.log().is_empty());
}

pub fn init_nft(sys: &System) {
    sys.init_logger();
    let nft = Program::from_file(
        &sys,
        "../nft/target/wasm32-unknown-unknown/release/nft.wasm",
    );

    let res = nft.send(
        USERS[0],
        InitNFT {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            base_uri: "".to_string(),
            supply: 100.into(),
            royalties: None,
        },
    );
    assert!(res.log().is_empty());
}

pub fn init_market(sys: &System) {
    sys.init_logger();
    let market = Program::current(&sys);
    let res = market.send(
        USERS[0],
        InitMarket {
            owner_id: USERS[0].into(),
            treasury_id: TREASURY_ID.into(),
            treasury_fee: 100,
        },
    );
    assert!(res.log().is_empty());
}

pub fn add_market_data(
    market: &Program,
    ft_contract_id: Option<ActorId>,
    user: u64,
    token_id: u128,
    price: Option<u128>,
) {
    // lists nft on the market
    let res = market.send(
        user,
        MarketAction::AddMarketData {
            nft_contract_id: 2.into(),
            ft_contract_id,
            token_id: token_id.into(),
            price,
        },
    );
    assert!(res.contains(&(
        user,
        MarketEvent::MarketDataAdded {
            nft_contract_id: 2.into(),
            owner: user.into(),
            token_id: token_id.into(),
            price,
        }
        .encode()
    )));
}

pub fn get_hash(nft_contract_id: &ActorId, ft_contract_id: Option<ActorId>, price: u128) -> spH256 {
    let buf: [u8; 32] = (*nft_contract_id).into();
    let nft_conract_vec: Vec<u8> = buf.iter().flat_map(|val| val.to_be_bytes()).collect();
    let mut price_vec: Vec<u8> = vec![];
    for i in 0..3 {
        price_vec.extend_from_slice(&(price >> (32 * i) as u32).to_be_bytes());
    }
    let hash = if let Some(ft) = ft_contract_id {
        let buf: [u8; 32] = ft.into();
        let ft_contract_vec: Vec<u8> = buf.iter().flat_map(|val| val.to_be_bytes()).collect();
        hashing::blake2_256(&[nft_conract_vec, price_vec, ft_contract_vec].concat()).into()
    } else {
        hashing::blake2_256(&[nft_conract_vec, price_vec].concat()).into()
    };
    hash
}
