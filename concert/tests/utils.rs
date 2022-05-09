use concert_io::*;
use gstd::Encode;
use gstd::String;
use gtest::{Program, System};

pub const USER: u64 = 193;
pub const ERC1155_ID: u64 = 2;
pub const CONCERT_ID: u128 = 1;
pub const NO_TICKETS: u128 = 100;
pub const AMOUNT: u128 = 1;

pub fn init_system() -> System {
    let system = System::new();
    system.init_logger();

    system
}

pub fn init_concert(sys: &System) -> Program {
    let concert_program = Program::current(&sys);

    assert!(concert_program
        .send(
            USER,
            InitConcert {
                name: String::from("ERC1155 Concert"),
                symbol: String::from("EC"),
                base_uri: String::from("http://erc1155.concert"),
            },
        )
        .log()
        .is_empty());

    concert_program
}

pub fn init_erc1155(sys: &System) {
    let erc1155_program = Program::current(&sys);

    assert!(erc1155_program
        .send(
            USER,
            InitConfig {
                name: String::from("ERC1155 Multitoken"),
                symbol: String::from("MTK"),
                base_uri: String::from("http://erc1155.multitoken"),
            },
        )
        .log()
        .is_empty());

    erc1155_program
}

pub fn create(
    concert: &Program,
    contract_id: ActorId,
    creator: ActorId,
    concert_id: u128,
    no_tickets: u128,
) {
    let res = concert.send(
        USER,
        ConcertAction::Create {
            contract_id: ERC1155_ID,
            creator: ActorId,
            concert_id: CONCERT_ID,
            no_tickets: NO_TICKETS,
        }
        .encode(),
    );

    assert!(res.contains(&(
        USERS[0],
        ConcertEvent::Creation {
            creator: USER,
            concert_id: CONCERT_ID,
            no_tickets: NO_TICKETS,
        }
        .encode()
    )));
}

pub fn buy(
    concert: &Program,
    concert_id: u128,
    amount: u128,
    metadata: Vec<Option<TokenMetadata>>,
) {
    let res = concert.send(
        USER,
        ConcertAction::BuyTicket {
            concert_id,
            amount,
            metadata,
        }
        .encode(),
    );

    assert!(res.contains(&(USER, ConcertEvent::Purchase { concert_id, amount }.encode())));
}

pub fn hold(concert: &Program, concert_id: u128) {
    let res = concert.send(USER, ConcertAction::Hold { concert_id }.encode());
    assert!(res.contains(&(USERS[0], ConcertEvent::Hold { concert_id }.encode())));
}
