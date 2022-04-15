use codec::Encode;
use gear_contract_libraries::non_fungible_token::{io::*, token::*};
use gtest::{Program, System};
use nft_io::*;
const USERS: &'static [u64] = &[3, 4, 5];

fn init_with_mint(sys: &System) {
    sys.init_logger();
    let nft = Program::current(&sys);
    let res = nft.send(
        USERS[0],
        InitNFT {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            base_uri: String::from(""),
        },
    );

    assert!(res.log().is_empty());

    let res = nft.send(
        USERS[0],
        MyNFTAction::Mint {
            token_metadata: TokenMetadata {
                title: Some("CryptoKitty".to_string()),
                description: Some("Description".to_string()),
                media: Some("http://".to_string()),
                reference: Some("http://".to_string()),
            },
        }
        .encode(),
    );
    assert!(res.contains(&(
        USERS[0],
        NFTEvent::Transfer {
            from: 0.into(),
            to: USERS[0].into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}

#[test]
fn burn() {
    let sys = System::new();
    init_with_mint(&sys);
    let nft = sys.get_program(1);
    let res = nft.send(
        USERS[0],
        MyNFTAction::Base(NFTAction::Burn { token_id: 0.into() }).encode(),
    );
    assert!(res.contains(&(
        USERS[0],
        NFTEvent::Transfer {
            from: USERS[0].into(),
            to: 0.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn burn_failures() {
    let sys = System::new();
    init_with_mint(&sys);
    let nft = sys.get_program(1);
    // must fail since the token doesn't exist
    let res = nft.send(
        USERS[0],
        MyNFTAction::Base(NFTAction::Burn { token_id: 1.into() }).encode(),
    );
    assert!(res.main_failed());

    // must fail since the caller isn't the token owner
    let res = nft.send(
        USERS[1],
        MyNFTAction::Base(NFTAction::Burn { token_id: 0.into() }).encode(),
    );
    assert!(res.main_failed());
}

#[test]
fn transfer() {
    let sys = System::new();
    init_with_mint(&sys);
    let nft = sys.get_program(1);
    let res = nft.send(
        USERS[0],
        MyNFTAction::Base(NFTAction::Transfer {
            to: USERS[1].into(),
            token_id: 0.into(),
        })
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        NFTEvent::Transfer {
            from: USERS[0].into(),
            to: USERS[1].into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_failures() {
    let sys = System::new();
    init_with_mint(&sys);
    let nft = sys.get_program(1);
    //must fail since the tokens doesn't exist
    let res = nft.send(
        USERS[0],
        MyNFTAction::Base(NFTAction::Transfer {
            to: USERS[1].into(),
            token_id: 100.into(),
        })
        .encode(),
    );
    assert!(res.main_failed());

    //must fail since the caller isn't the is not an authorized source
    let res = nft.send(
        USERS[2],
        MyNFTAction::Base(NFTAction::Transfer {
            to: USERS[1].into(),
            token_id: 0.into(),
        })
        .encode(),
    );
    assert!(res.main_failed());

    //must fail since the `to` is the zero address
    let res = nft.send(
        USERS[0],
        MyNFTAction::Base(NFTAction::Transfer {
            to: 0.into(),
            token_id: 0.into(),
        })
        .encode(),
    );
    assert!(res.main_failed());
}

#[test]
fn approve_and_transfer() {
    let sys = System::new();
    init_with_mint(&sys);
    let nft = sys.get_program(1);

    let res = nft.send(
        USERS[0],
        MyNFTAction::Base(NFTAction::Approve {
            to: USERS[1].into(),
            token_id: 0.into(),
        })
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        NFTEvent::Approval {
            owner: USERS[0].into(),
            approved_account: USERS[1].into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = nft.send(
        USERS[1],
        MyNFTAction::Base(NFTAction::Transfer {
            to: USERS[2].into(),
            token_id: 0.into(),
        })
        .encode(),
    );
    assert!(res.contains(&(
        USERS[1],
        NFTEvent::Transfer {
            from: USERS[0].into(),
            to: USERS[2].into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}
