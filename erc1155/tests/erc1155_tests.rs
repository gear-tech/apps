use codec::Encode;
use erc1155_io::*;
use gear_contract_libraries::erc1155::io::*;
use gstd::{ActorId, String};
use gtest::{Program, System};

const USERS: &'static [u64] = &[3, 4, 5, 0];
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const TOKEN_AMOUNT: u128 = 100;
const TOKENS_TO_BURN: u128 = 50;
const TOKEN_ID: u128 = 0;

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

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Mint {
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
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}

#[test]
fn mint_failures() {
    let sys = System::new();
    let erc1155 = init(&sys);

    // Must fail since minting to ZERO_ID
    let res = erc1155.send(
        0,
        MyERC1155Action::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );
    assert!(res.main_failed());

    // Must fail since provided meta for amount > 1
    let meta = TokenMetadata {
        title: Some(String::from("Kitty")),
        description: Some(String::from("Just a test kitty")),
        media: Some(String::from("www.example.com/erc1155/kitty.png")),
        reference: Some(String::from("www.example.com/erc1155/kitty")),
    };
    let res = erc1155.send(
        0,
        MyERC1155Action::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: Some(meta.clone()),
        }
        .encode(),
    );
    assert!(res.main_failed());
}

#[test]
fn mint_batch() {
    let sys = System::new();
    let erc1155 = init(&sys);

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::MintBatch(
            USERS[0].into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = ERC1155Event::TransferBatch {
        operator: USERS[0].into(),
        from: ZERO_ID,
        to: USERS[0].into(),
        ids: vec![1u128, 2u128],
        values: vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn burn() {
    let sys = System::new();
    let erc1155 = init(&sys);

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Mint {
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
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Burn {
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ERC1155Event::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: USERS[0].into(),
            to: ZERO_ID,
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));
}

#[test]
fn burn_failures() {
    let sys = System::new();
    let erc1155 = init(&sys);

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Mint {
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
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    // Must fail since we do not have enough tokens
    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Burn {
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT + 1,
        }
        .encode(),
    );

    assert!(res.main_failed());
}

#[test]
fn burn_batch() {
    let sys = System::new();
    let erc1155 = init(&sys);

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::MintBatch(
            USERS[0].into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = ERC1155Event::TransferBatch {
        operator: USERS[0].into(),
        from: ZERO_ID,
        to: USERS[0].into(),
        ids: vec![1u128, 2u128],
        values: vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::BurnBatch(
            vec![1u128, 2u128],
            vec![TOKENS_TO_BURN, TOKENS_TO_BURN],
        ))
        .encode(),
    );

    let codec = ERC1155Event::TransferBatch {
        operator: USERS[0].into(),
        from: USERS[0].into(),
        to: ZERO_ID,
        ids: vec![1u128, 2u128],
        values: vec![TOKENS_TO_BURN, TOKENS_TO_BURN],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn balance() {
    let sys = System::new();
    let erc1155 = init(&sys);
    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Mint {
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
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::BalanceOf(USERS[0].into(), TOKEN_ID)).encode(),
    );

    assert!(res.contains(&(USERS[0], ERC1155Event::Balance(TOKEN_AMOUNT).encode())));
}

#[test]
fn balance_of_batch() {
    let sys = System::new();
    let erc1155 = init(&sys);

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::MintBatch(
            USERS[0].into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = ERC1155Event::TransferBatch {
        operator: USERS[0].into(),
        from: ZERO_ID,
        to: USERS[0].into(),
        ids: vec![1u128, 2u128],
        values: vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));

    let accounts: Vec<ActorId> = vec![USERS[0].into(), USERS[0].into()];

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::BalanceOfBatch(accounts, vec![1u128, 2u128])).encode(),
    );

    let reply1 = BalanceOfBatchReply {
        account: USERS[0].into(),
        id: 1,
        amount: TOKEN_AMOUNT,
    };

    let reply2 = BalanceOfBatchReply {
        account: USERS[0].into(),
        id: 2,
        amount: TOKEN_AMOUNT,
    };

    let replies = vec![reply1, reply2];

    let codec = ERC1155Event::BalanceOfBatch(replies).encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn transfer_from() {
    let sys = System::new();
    let erc1155 = init(&sys);

    erc1155.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::Mint(
            USERS[1].into(),
            TOKEN_ID,
            TOKEN_AMOUNT,
            None,
        ))
        .encode(),
    );

    let from = USERS[1];
    let to = USERS[2];

    let res = erc1155.send(
        from,
        MyERC1155Action::Base(ERC1155Action::TransferFrom(
            from.into(),
            to.into(),
            TOKEN_ID,
            10,
        ))
        .encode(),
    );

    let reply = TransferSingleReply {
        operator: from.into(),
        from: from.into(),
        to: to.into(),
        id: TOKEN_ID,
        amount: 10,
    };

    let codec = ERC1155Event::TransferSingle(reply).encode();
    assert!(res.contains(&(from, codec)));
}

#[test]
fn transfer_from_failures() {
    let sys = System::new();
    let concert = init(&sys);

    concert.send(
        USERS[0],
        MyERC1155Action::Base(ERC1155Action::Mint(
            USERS[1].into(),
            TOKEN_ID,
            TOKEN_AMOUNT,
            None,
        ))
        .encode(),
    );

    let from = USERS[1];
    let to = USERS[2];
    let invalid_user = USERS[3];

    let failed_res = concert.send(
        from,
        MyERC1155Action::Base(ERC1155Action::TransferFrom(
            from.into(),
            ZERO_ID.into(),
            TOKEN_ID,
            10,
        ))
        .encode(),
    );
    // must fail since we're sending to ZERO_ID
    assert!(failed_res.main_failed());

    let failed_res = concert.send(
        from,
        MyERC1155Action::Base(ERC1155Action::TransferFrom(
            from.into(),
            to.into(),
            TOKEN_ID,
            TOKEN_AMOUNT + 100,
        ))
        .encode(),
    );
    // must fail since we're sending >balance
    assert!(failed_res.main_failed());

    let failed_res = concert.send(
        from,
        MyERC1155Action::Base(ERC1155Action::TransferFrom(
            from.into(),
            from.into(),
            TOKEN_ID,
            TOKEN_AMOUNT - 100,
        ))
        .encode(),
    );
    // must fail since same addresses
    assert!(failed_res.main_failed());

    let failed_res = concert.send(
        invalid_user,
        MyERC1155Action::Base(ERC1155Action::TransferFrom(
            from.into(),
            to.into(),
            TOKEN_ID,
            TOKEN_AMOUNT - 100,
        ))
        .encode(),
    );
    // must fail since same addresses
    assert!(failed_res.main_failed());
}

#[test]
fn batch_transfer_from() {
    let sys = System::new();
    let concert = init(&sys);

    let from = USERS[0];
    let to = USERS[1];
    let newuser = USERS[2];

    concert.send(
        from,
        MyERC1155Action::Base(ERC1155Action::MintBatch(
            to.into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let ret = concert.send(
        to,
        MyERC1155Action::Base(ERC1155Action::BatchTransferFrom(
            to.into(),
            newuser.into(),
            vec![1u128, 2u128],
            vec![5u128, 10u128],
        ))
        .encode(),
    );

    let codec = ERC1155Event::TransferBatch {
        operator: to.into(),
        from: to.into(),
        to: newuser.into(),
        ids: vec![1u128, 2u128],
        values: vec![5u128, 10u128],
    }
    .encode();

    assert!(ret.contains(&(to, codec)));
}

#[test]
fn supply() {
    let sys = System::new();
    let erc1155 = init(&sys);

    let res = erc1155.send(
        USERS[0],
        MyERC1155Action::Mint {
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
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    let res = erc1155.send(USERS[0], MyERC1155Action::Supply { id: TOKEN_ID }.encode());

    assert!(res.contains(&(
        USERS[0],
        MyERC1155Event::Supply {
            amount: TOKEN_AMOUNT,
        }
        .encode()
    )));
}
