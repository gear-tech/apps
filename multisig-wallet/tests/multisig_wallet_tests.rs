use codec::Encode;
use gtest::{Program, System};
use multisig_wallet_io::*;
const USERS: &'static [u64] = &[3, 4, 5, 6];

fn init_with_mint(sys: &System) {
    // sys.init_logger();
    //
    // let nft = Program::from_file(
    //     &sys,
    //     "../target/wasm32-unknown-unknown/release/multisig_wallet.wasm",
    // );
    //
    // let res = nft.send(
    //     USERS[0],
    //     MWInitConfig {
    //         owners: vec![],
    //         required: 0
    //     },
    // );
    //
    // assert!(res.log().is_empty());
    //
    // let res = nft.send(USERS[0], Action::Mint);
    // assert!(res.contains(&(
    //     USERS[0],
    //     Event::Transfer {
    //         from: 0.into(),
    //         to: USERS[0].into(),
    //         token_id: 0.into(),
    //     }
    //         .encode()
    // )));
}
