use codec::Encode;
use gear_contract_libraries::erc1155::{io::*};
use gtest::{Program, System};
use erc1155_io::*;
use gstd::{debug, ActorId, String};


const USERS: &'static [u64] = &[3, 4, 5];
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const TOKEN_AMOUNT: u128 = 100;
const NFT_AMOUNT: u128 = 1;
const TOKEN_IDS: &'static [u128] = &[0, 1];

fn init(sys: &System) -> Program {
    sys.init_logger();

    let erc1155 = Program::current(&sys);
    let res = erc1155.send(
        USERS[0],
        InitERC1155 {
            name: String::from("ERC1155 Simple"),
            symbol: String::from("EC"),
            base_uri: String::from("http://erc1155.simple"),
        },
    );

    assert!(res.log().is_empty());
    return erc1155;
}

fn init_with_mint(sys: &System) {
    sys.init_logger();
    let erc1155 = Program::current(&sys);
    let res = erc1155.send(
        USERS[0],
        InitERC1155 {
            name: String::from("ERC1155 Simple"),
            symbol: String::from("EC"),
            base_uri: String::from("http://erc1155.simple"),
        },
    );

    assert!(res.log().is_empty());
    println!("AFTER INIT");
    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Mint{
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ERC1155Event::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: ZERO_ID,
            to: USERS[1].into(),
            id: TOKEN_IDS[0],
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));
}
// mint

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}
// burn
// supply
//