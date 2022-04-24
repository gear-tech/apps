use codec::Encode;
use gear_contract_libraries::erc1155::io::*;
use gtest::{Program, System};

use concert_io::*;

use gstd::{ActorId, String};

const USERS: &'static [u64] = &[3, 4, 5, 0];
const SUM41_CONCERT: u128 = 10;
const SUM41_TICKETS: u128 = 100;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const TOKEN_ID: u128 = 1;
const BALANCE: u128 = 100;

fn init(sys: &System) -> Program {
    sys.init_logger();

    let concert = Program::current(&sys);
    let res = concert.send(
        USERS[0],
        InitConcertERC1155 {
            name: String::from("ERC1155 Concert"),
            symbol: String::from("EC"),
            base_uri: String::from("http://erc1155.concert"),
        },
    );
    assert!(res.log().is_empty());
    return concert;
}

fn init_with_mint(sys: &System) {
    sys.init_logger();

    let concert = Program::current(&sys);

    let res = concert.send(
        USERS[0],
        InitConcertERC1155 {
            name: String::from("ERC1155 Concert"),
            symbol: String::from("EC"),
            base_uri: String::from("http://erc1155.concert"),
        },
    );

    assert!(res.log().is_empty());

    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[1].into(), TOKEN_ID, BALANCE, None)).encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        Event::TransferSingle(TransferSingleReply {
            operator: USERS[0].into(),
            from: ZERO_ID,
            to: USERS[1].into(),
            id: TOKEN_ID,
            amount: BALANCE,
        })
        .encode()
    )));
}

#[test]
fn start_concert() {
    let sys = System::new();
    let concert = init(&sys);

    let res = concert.send(
        USERS[0],
        ConcertAction::Create {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Creation {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode()
    )));
}

#[test]
fn buy_tickets() {
    let sys = System::new();
    let concert = init(&sys);

    let res = concert.send(
        USERS[0],
        ConcertAction::Create {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Creation {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode()
    )));

    let amount: u128 = SUM41_TICKETS - 10;
    let res = concert.send(
        USERS[0],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: amount,
            metadata: std::iter::repeat_with(|| None)
                .take(amount as usize)
                .collect(),
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Purchase {
            concert_id: SUM41_CONCERT,
            amount: amount,
        }
        .encode()
    )));
}

#[test]
fn buy_tickets_failures() {
    let sys = System::new();
    let concert = init(&sys);

    let amount: u128 = SUM41_TICKETS - 10;
    let res = concert.send(
        USERS[0],
        ConcertAction::Create {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Creation {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode()
    )));

    // ZERO_ID
    let res = concert.send(
        USERS[3],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: SUM41_TICKETS - 10,
            metadata: std::iter::repeat_with(|| None)
                .take(amount as usize)
                .collect(),
        }
        .encode(),
    );
    assert!(res.main_failed());

    // AMOUNT < 1

    let res = concert.send(
        USERS[0],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: 0,
            metadata: std::iter::repeat_with(|| None)
                .take(amount as usize)
                .collect(),
        }
        .encode(),
    );
    assert!(res.main_failed());
    // NOT ENOUGH TICKETS

    let res = concert.send(
        USERS[0],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: SUM41_TICKETS + 100,
            metadata: std::iter::repeat_with(|| None)
                .take(amount as usize)
                .collect(),
        }
        .encode(),
    );
    assert!(res.main_failed());
    // METADATA SIZE != AMOUNT

    let res = concert.send(
        USERS[0],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: amount,
            metadata: std::iter::repeat_with(|| None)
                .take((amount - 1) as usize)
                .collect(),
        }
        .encode(),
    );
    assert!(res.main_failed());
}

#[test]
fn hold_concert() {
    let sys = System::new();
    let concert = init(&sys);

    let res = concert.send(
        USERS[0],
        ConcertAction::Create {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Creation {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode()
    )));

    let amount: u128 = 1;
    let res = concert.send(
        USERS[0],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: amount,
            metadata: std::iter::repeat_with(|| None)
                .take(amount as usize)
                .collect(),
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Purchase {
            concert_id: SUM41_CONCERT,
            amount: amount,
        }
        .encode()
    )));

    // hold :)
    // 1 here
    let res = concert.send(
        USERS[1],
        ConcertAction::Hold {
            concert_id: SUM41_CONCERT,
        }
        .encode(),
    );
    assert!(res.contains(&(
        USERS[1],
        ConcertEvent::Hold {
            concert_id: SUM41_CONCERT,
        }
        .encode()
    )));
}

#[test]
fn hold_concert_failures() {
    let sys = System::new();
    let concert = init(&sys);

    let res = concert.send(
        USERS[0],
        ConcertAction::Create {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Creation {
            creator: USERS[1].into(),
            concert_id: SUM41_CONCERT,
            no_tickets: SUM41_TICKETS,
        }
        .encode()
    )));

    let amount: u128 = 1;
    let res = concert.send(
        USERS[0],
        ConcertAction::BuyTicket {
            concert_id: SUM41_CONCERT,
            amount: amount,
            metadata: std::iter::repeat_with(|| None)
                .take(amount as usize)
                .collect(),
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Purchase {
            concert_id: SUM41_CONCERT,
            amount: amount,
        }
        .encode()
    )));

    // hold :)
    let res = concert.send(
        USERS[2],
        ConcertAction::Hold {
            concert_id: SUM41_CONCERT,
        }
        .encode(),
    );
    assert!(res.main_failed());
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}

#[test]
fn mint_failures() {
    let sys = System::new();
    let concert = init(&sys);
    let meta = TokenMetadata {
        title: Some(String::from("Kitty")),
        description: Some(String::from("Just a test kitty")),
        media: Some(String::from("www.example.com/nconcerts/kitty.png")),
        reference: Some(String::from("www.example.com/nconcerts/kitty")),
    };

    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[1].into(), TOKEN_ID, BALANCE, None)).encode(),
    );
    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(
            ZERO_ID.into(),
            TOKEN_ID,
            BALANCE,
            Some(meta.clone()),
        ))
        .encode(),
    );

    // must fail since we provided a zero_id user
    assert!(res.main_failed());

    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(
            USERS[1].into(),
            TOKEN_ID,
            BALANCE,
            Some(meta.clone()),
        ))
        .encode(),
    );
    // must fail since we provided metadata for balance > 1
    assert!(res.main_failed());
}

#[test]
fn balance() {
    let sys = System::new();
    let concert = init(&sys);
    concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[1].into(), TOKEN_ID, BALANCE, None)).encode(),
    );

    let res = concert.send(
        USERS[1],
        ConcertAction::Base(Action::BalanceOf(USERS[1].into(), TOKEN_ID)).encode(),
    );

    assert!(res.contains(&(USERS[1], Event::Balance(BALANCE).encode())));
}

#[test]
fn balance_of_batch() {
    let sys = System::new();
    let concert = init(&sys);

    concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[1].into(), 1, BALANCE, None)).encode(),
    );

    concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[2].into(), 2, BALANCE, None)).encode(),
    );

    let accounts: Vec<ActorId> = vec![USERS[1].into(), USERS[2].into()];

    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::BalanceOfBatch(accounts, vec![1u128, 2u128])).encode(),
    );

    let reply1 = BalanceOfBatchReply {
        account: USERS[1].into(),
        id: 1,
        amount: BALANCE,
    };

    let reply2 = BalanceOfBatchReply {
        account: USERS[2].into(),
        id: 2,
        amount: BALANCE,
    };

    let replies = vec![reply1, reply2];

    let codec = Event::BalanceOfBatch(replies).encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn mint_batch() {
    let sys = System::new();
    let concert = init(&sys);

    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::MintBatch(
            USERS[1].into(),
            vec![1u128, 2u128],
            vec![BALANCE, BALANCE],
            vec![None, None],
        ))
        .encode(),
    );

    let codec = Event::TransferBatch {
        operator: USERS[0].into(),
        from: ZERO_ID,
        to: USERS[1].into(),
        ids: vec![1u128, 2u128],
        values: vec![BALANCE, BALANCE],
    }
    .encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn safe_transfer_from() {
    let sys = System::new();
    let concert = init(&sys);

    concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[1].into(), TOKEN_ID, BALANCE, None)).encode(),
    );

    let from = USERS[1];
    let to = USERS[2];

    let res = concert.send(
        from,
        ConcertAction::Base(Action::TransferFrom(from.into(), to.into(), TOKEN_ID, 10)).encode(),
    );

    let reply = TransferSingleReply {
        operator: from.into(),
        from: from.into(),
        to: to.into(),
        id: TOKEN_ID,
        amount: 10,
    };

    let codec = Event::TransferSingle(reply).encode();
    assert!(res.contains(&(from, codec)));

    let failed_res = concert.send(
        from,
        ConcertAction::Base(Action::TransferFrom(
            from.into(),
            ZERO_ID.into(),
            TOKEN_ID,
            10,
        ))
        .encode(),
    );

    assert!(failed_res.main_failed());

    // check two accounts balance
    let accounts: Vec<ActorId> = vec![from.into(), to.into()];
    let ids: Vec<u128> = vec![1, 1];
    let res = concert.send(
        USERS[0],
        ConcertAction::Base(Action::BalanceOfBatch(accounts, ids)).encode(),
    );

    let reply1 = BalanceOfBatchReply {
        account: from.into(),
        id: TOKEN_ID,
        amount: BALANCE - 10,
    };

    let reply2 = BalanceOfBatchReply {
        account: to.into(),
        id: TOKEN_ID,
        amount: 10,
    };

    let replies = vec![reply1, reply2];
    let codec = Event::BalanceOfBatch(replies).encode();

    assert!(res.contains(&(USERS[0], codec)));
}

#[test]
fn safe_transfer_from_failures() {
    let sys = System::new();
    let concert = init(&sys);

    concert.send(
        USERS[0],
        ConcertAction::Base(Action::Mint(USERS[1].into(), TOKEN_ID, BALANCE, None)).encode(),
    );

    let from = USERS[1];
    let to = USERS[2];
    let invalid_user = USERS[3];

    let failed_res = concert.send(
        from,
        ConcertAction::Base(Action::TransferFrom(
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
        ConcertAction::Base(Action::TransferFrom(
            from.into(),
            to.into(),
            TOKEN_ID,
            BALANCE + 100,
        ))
        .encode(),
    );
    // must fail since we're sending >balance
    assert!(failed_res.main_failed());

    let failed_res = concert.send(
        from,
        ConcertAction::Base(Action::TransferFrom(
            from.into(),
            from.into(),
            TOKEN_ID,
            BALANCE - 100,
        ))
        .encode(),
    );
    // must fail since same addresses
    assert!(failed_res.main_failed());

    let failed_res = concert.send(
        invalid_user,
        ConcertAction::Base(Action::TransferFrom(
            from.into(),
            to.into(),
            TOKEN_ID,
            BALANCE - 100,
        ))
        .encode(),
    );
    // must fail since same addresses
    assert!(failed_res.main_failed());
}

#[test]
fn safe_batch_transfer_from() {
    let sys = System::new();
    let concert = init(&sys);

    let from = USERS[0];
    let to = USERS[1];
    let newuser = USERS[2];

    concert.send(
        from,
        ConcertAction::Base(Action::MintBatch(
            to.into(),
            vec![1u128, 2u128],
            vec![BALANCE, BALANCE],
            vec![None, None],
        ))
        .encode(),
    );

    let ret = concert.send(
        to,
        ConcertAction::Base(Action::BatchTransferFrom(
            to.into(),
            newuser.into(),
            vec![1u128, 2u128],
            vec![5u128, 10u128],
        ))
        .encode(),
    );

    let codec = Event::TransferBatch {
        operator: to.into(),
        from: to.into(),
        to: newuser.into(),
        ids: vec![1u128, 2u128],
        values: vec![5u128, 10u128],
    }
    .encode();

    assert!(ret.contains(&(to, codec)));
}
