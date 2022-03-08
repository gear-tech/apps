use codec::Encode;
use fungible_token_messages::*;
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

fn before_each_test(sys: &System) {
    // init_ft(&sys);
    init_nft(&sys);
    let ft = sys.get_program(1);
    let nft = sys.get_program(2);
    let market = sys.get_program(3);
}
