#![feature(const_btree_new)]

#[path = "../src/lib.rs"]
mod lib;

#[cfg(test)]
use codec::Encode;
use gstd::String;
use gtest::{Program, System, Log};

const USERS: &'static [u64] = &[3, 4, 5];

fn init_with_mint(sys: &System) {
    sys.init_logger();

    let ft = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/erc1155.wasm",
    );

    let init_config = lib::InitConfig {
        name: String::from("MyToken"),
        symbol: String::from("MTK"),
        base_uri: String::from("baidu.so"),
    };

    let res = ft.send(USERS[0], init_config);

    assert!(res.log().is_empty());

    let res = ft.send(USERS[0], lib::Action::Mint(USERS[1].into(), 1, 1000000));

    let logs = res.log();
    println!("logs: {:?}", logs);

    assert!(res.contains(&(
        USERS[0],
        lib::Event::TransferSingle(lib::TransferSingleReply {
            operator: 0.into(),
            from: 0.into(),
            to: USERS[1].into(),
            id: 1,
            amount: 1000000,
        })
        .encode()
    )));
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}
