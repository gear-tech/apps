use codec::Encode;
use multitoken_io::*;
use gear_contract_libraries::multitoken::io::*;
use gstd::{ActorId, String};
use gtest::{Program, System};

const USERS: &'static [u64] = &[3, 4, 5, 0];
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const TOKEN_AMOUNT: u128 = 100;
const TOKENS_TO_BURN: u128 = 50;
const TOKEN_ID: u128 = 0;

fn init(sys: &System) -> Program {
    sys.init_logger();

    let mtk = Program::current(&sys);
    let res = mtk.send(
        USERS[0],
        InitMTK {
            name: String::from("MTK Simple"),
            symbol: String::from("MTK"),
            base_uri: String::from("http://mtk.simple"),
        },
    );

    assert!(res.log().is_empty());
    return mtk;
}

fn init_with_mint(sys: &System) {
    sys.init_logger();
    let mtk = Program::current(&sys);
    let res = mtk.send(
        USERS[0],
        InitMTK {
            name: String::from("MTK Simple"),
            symbol: String::from("MTK"),
            base_uri: String::from("http://mtk.simple"),
        },
    );

    assert!(res.log().is_empty());

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        MTKEvent::TransferSingle(TransferSingleReply {
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
    let mtk = init(&sys);

    // Must fail since minting to ZERO_ID
    let res = mtk.send(
        0,
        MyMTKAction::Mint {
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
    let res = mtk.send(
        0,
        MyMTKAction::Mint {
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
    let mtk = init(&sys);

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::MintBatch(
            USERS[0].into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = MTKEvent::TransferBatch {
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
    let mtk = init(&sys);

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        MTKEvent::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: ZERO_ID,
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Burn {
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        MTKEvent::TransferSingle(TransferSingleReply {
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
    let mtk = init(&sys);

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        MTKEvent::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: ZERO_ID,
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    // Must fail since we do not have enough tokens
    let res = mtk.send(
        USERS[0],
        MyMTKAction::Burn {
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
    let mtk = init(&sys);

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::MintBatch(
            USERS[0].into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = MTKEvent::TransferBatch {
        operator: USERS[0].into(),
        from: ZERO_ID,
        to: USERS[0].into(),
        ids: vec![1u128, 2u128],
        values: vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::BurnBatch(
            vec![1u128, 2u128],
            vec![TOKENS_TO_BURN, TOKENS_TO_BURN],
        ))
        .encode(),
    );

    let codec = MTKEvent::TransferBatch {
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
    let mtk = init(&sys);
    let res = mtk.send(
        USERS[0],
        MyMTKAction::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        MTKEvent::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: ZERO_ID,
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::BalanceOf(USERS[0].into(), TOKEN_ID)).encode(),
    );

    assert!(res.contains(&(USERS[0], MTKEvent::Balance(TOKEN_AMOUNT).encode())));
}

#[test]
fn balance_of_batch() {
    let sys = System::new();
    let mtk = init(&sys);

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::MintBatch(
            USERS[0].into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = MTKEvent::TransferBatch {
        operator: USERS[0].into(),
        from: ZERO_ID,
        to: USERS[0].into(),
        ids: vec![1u128, 2u128],
        values: vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));

    let accounts: Vec<ActorId> = vec![USERS[0].into(), USERS[0].into()];

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::BalanceOfBatch(accounts, vec![1u128, 2u128])).encode(),
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

    let codec = MTKEvent::BalanceOfBatch(replies).encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn transfer_from() {
    let sys = System::new();
    let mtk = init(&sys);

    mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::Mint(
            USERS[1].into(),
            TOKEN_ID,
            TOKEN_AMOUNT,
            None,
        ))
        .encode(),
    );

    let from = USERS[1];
    let to = USERS[2];

    let res = mtk.send(
        from,
        MyMTKAction::Base(MTKAction::TransferFrom(
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

    let codec = MTKEvent::TransferSingle(reply).encode();
    assert!(res.contains(&(from, codec)));
}

#[test]
fn transfer_from_failures() {
    let sys = System::new();
    let mtk = init(&sys);

    mtk.send(
        USERS[0],
        MyMTKAction::Base(MTKAction::Mint(
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

    let failed_res = mtk.send(
        from,
        MyMTKAction::Base(MTKAction::TransferFrom(
            from.into(),
            ZERO_ID.into(),
            TOKEN_ID,
            10,
        ))
        .encode(),
    );
    // must fail since we're sending to ZERO_ID
    assert!(failed_res.main_failed());

    let failed_res = mtk.send(
        from,
        MyMTKAction::Base(MTKAction::TransferFrom(
            from.into(),
            to.into(),
            TOKEN_ID,
            TOKEN_AMOUNT + 100,
        ))
        .encode(),
    );
    // must fail since we're sending >balance
    assert!(failed_res.main_failed());

    let failed_res = mtk.send(
        from,
        MyMTKAction::Base(MTKAction::TransferFrom(
            from.into(),
            from.into(),
            TOKEN_ID,
            TOKEN_AMOUNT - 100,
        ))
        .encode(),
    );
    // must fail since same addresses
    assert!(failed_res.main_failed());

    let failed_res = mtk.send(
        invalid_user,
        MyMTKAction::Base(MTKAction::TransferFrom(
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
    let mtk = init(&sys);

    let from = USERS[0];
    let to = USERS[1];
    let newuser = USERS[2];

    mtk.send(
        from,
        MyMTKAction::Base(MTKAction::MintBatch(
            to.into(),
            vec![1u128, 2u128],
            vec![TOKEN_AMOUNT, TOKEN_AMOUNT],
            vec![None, None],
        ))
        .encode(),
    );

    let ret = mtk.send(
        to,
        MyMTKAction::Base(MTKAction::BatchTransferFrom(
            to.into(),
            newuser.into(),
            vec![1u128, 2u128],
            vec![5u128, 10u128],
        ))
        .encode(),
    );

    let codec = MTKEvent::TransferBatch {
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
    let mtk = init(&sys);

    let res = mtk.send(
        USERS[0],
        MyMTKAction::Mint {
            amount: TOKEN_AMOUNT,
            token_metadata: None,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        MTKEvent::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: ZERO_ID,
            to: USERS[0].into(),
            id: TOKEN_ID,
            amount: TOKEN_AMOUNT,
        })
        .encode()
    )));

    let res = mtk.send(USERS[0], MyMTKAction::Supply { id: TOKEN_ID }.encode());

    assert!(res.contains(&(
        USERS[0],
        MyMTKEvent::Supply {
            amount: TOKEN_AMOUNT,
        }
        .encode()
    )));
}
