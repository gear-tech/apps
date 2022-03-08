use codec::Encode;
use market_io::*;
use nft_io::*;

use gtest::{Program, System};
use primitive_types::U256;

fn init_nft(sys: &System) {
    sys.init_logger();
    let nft = Program::from_file(
        &sys,
        "../../../apps/target/wasm32-unknown-unknown/release/nft.wasm",
    );

    let res = nft.send(
        100001,
        InitNFT {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            base_uri: "".to_string(),
            price: 100,
            supply: 100.into(),
            royalties: None,
        },
    );

    assert!(res.log().is_empty());
}

fn init_market(sys: &System) {
    sys.init_logger();
    let market = Program::from_file(
        &sys,
        "../../../apps/target/wasm32-unknown-unknown/release/nft_marketplace.wasm",
    );

    let res = market.send(
        100001,
        InitMarket {
            owner_id: 3.into(),
            treasury_id: 10.into(),
            treasury_fee: 100,
            approved_ft_token: 1.into(),
            offer_history_length: None,
        },
    );

    assert!(res.log().is_empty());

    let res = market.send(100001, MarketAction::AddNftContract(1.into()));
    assert!(res.log().is_empty());
}

fn mint_nft(nft: &Program) {
    let res = nft.send(
        4,
        NFTAction::Mint {
            media: "".to_string(),
            reference: "".to_string(),
        },
    );
    assert!(!res.main_failed());
}

// #[ignore]
// #[test]
// fn lists_all_nfts() {
//     let sys = System::new();
//     sys.init_logger();
//     init_nft(&sys);
//     init_market(&sys);
//     let nft = sys.get_program(1);
//     let market = sys.get_program(2);

//     // User mints NFTs in the nft-contract
//     let mut tokens: Vec<U256> = Vec::new();
//     for i in 0..9 {
//         tokens.push(i.into());
//         mint_nft(&nft);
//     };

//     // User lists his NFTs on marketplace
//     let res = nft.send(
//         4,
//         NFTAction::Approve {
//             to: 2.into(),
//             token_ids: None,
//             message: Some(MessageToMarket {
//                 msg_type: "sale".to_string(),
//                 price: Some(100),
//                 bid_period: None,
//             }),
//         }
//     );
//     assert!(res.contains(&(
//         4,
//         NFTEvent::Approval {
//             owner: 4.into(),
//             spender: 2.into(),
//             token_ids: tokens,
//         }
//         .encode()
//     )));

//     // Checks that items has appeared on the market
//     for i in 0..9 {
//         let res = market.send(
//             4,
//             MarketAction::Item {
//                 nft_contract_id: 1.into(),
//                 token_id: i.into(),
//             }
//         );
//         assert!(res.contains(&(
//             4,
//             MarketEvent::Item {
//                 owner_id: 4.into(),
//                 nft_contract_id: 1.into(),
//                 token_id: i.into(),
//                 price: Some(100),
//             }
//             .encode()
//         )));
//     };
// }

// #[ignore]
// #[test]
// fn lists_selected_nfts() {
//     let sys = System::new();
//     sys.init_logger();
//     init_nft(&sys);
//     init_market(&sys);
//     let nft = sys.get_program(1);
//     let market = sys.get_program(2);

//     // User mints NFTs in the nft-contract
//     let tokens = vec![1.into(), 3.into(), 5.into()];
//     for _i in 0..9 {
//         mint_nft(&nft);
//     };

//     // User lists his NFTs on marketplace
//     let res = nft.send(
//         4,
//         NFTAction::Approve {
//             to: 2.into(),
//             token_ids: Some(tokens.clone()),
//             message: Some(MessageToMarket {
//                 msg_type: "sale".to_string(),
//                 price: Some(100),
//                 bid_period: None,
//             }),
//         }
//     );
//     assert!(res.contains(&(
//         4,
//         NFTEvent::Approval {
//             owner: 4.into(),
//             spender: 2.into(),
//             token_id: tokens.clone(),
//         }
//         .encode()
//     )));

//     //Checks that items has appeared on the market
//     for token in tokens.iter() {
//         let res = market.send(
//             4,
//             MarketAction::Item {
//                 nft_contract_id: 1.into(),
//                 token_id:  *token,
//             }
//         );
//         assert!(res.contains(&(
//             4,
//             MarketEvent::Item {
//                 owner_id: 4.into(),
//                 nft_contract_id: 1.into(),
//                 token_id: *token,
//                 price: Some(100),
//             }
//             .encode()
//         )));
//     };
// }
