pub use ft_io::{FTAction, FTEvent, InitConfig as InitFT};
pub use gstd::prelude::*;
pub use gtest::{Program, System};
pub use nft_example_io::{Action as NFTAction, Event as NFTEvent, InitConfig as InitNFT};
pub use primitive_types::U256;
pub use supply_chain_io::*;

pub mod check;
pub mod fail;

pub const FT_PROGRAM_ID: u64 = 1;
pub const NFT_PROGRAM_ID: u64 = 2;
pub const SUPPLY_CHAIN_PROGRAM_ID: u64 = 3;
pub const PRODUCER: [u64; 2] = [4, 5];
pub const DISTRIBUTOR: [u64; 2] = [6, 7];
pub const RETAILER: [u64; 2] = [8, 9];
pub const CONSUMER: [u64; 2] = [10, 11];
pub const FOREIGN_USER: u64 = 1337;
pub const ITEM_ID: [u128; 2] = [0, 1];

pub fn init_system() -> System {
    let system = System::new();
    system.init_logger();

    system
}

pub fn init_ft_program(system: &System) -> Program {
    let ft_program = Program::from_file(
        &system,
        "../target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    assert!(ft_program
        .send(
            FOREIGN_USER,
            InitFT {
                name: "MyToken".into(),
                symbol: "MTK".into(),
            },
        )
        .log()
        .is_empty());

    ft_program
}

pub fn init_nft_program(system: &System) -> Program {
    let nft_program = Program::from_file(
        &system,
        "../target/wasm32-unknown-unknown/release/nft_example.wasm",
    );

    assert!(nft_program
        .send(
            FOREIGN_USER,
            InitNFT {
                name: "Item".into(),
                symbol: "ITM".into(),
                base_uri: "".into()
            },
        )
        .log()
        .is_empty());

    nft_program
}

pub fn init_supply_chain_program(system: &System) -> Program {
    let supply_chain_program = Program::current(system);

    assert!(supply_chain_program
        .send(
            FOREIGN_USER,
            InitSupplyChain {
                ft_program_id: FT_PROGRAM_ID.into(),
                nft_program_id: NFT_PROGRAM_ID.into(),

                producers: BTreeSet::from([PRODUCER[0].into(), PRODUCER[1].into()]),
                distributors: BTreeSet::from([DISTRIBUTOR[0].into(), DISTRIBUTOR[1].into()]),
                retailers: BTreeSet::from([RETAILER[0].into(), RETAILER[1].into()]),
            },
        )
        .log()
        .is_empty());

    supply_chain_program
}

pub fn check_ft_balance(ft_program: &Program, actor: u64, amount: u128) {
    assert!(ft_program
        .send(FOREIGN_USER, FTAction::BalanceOf(actor.into()))
        .contains(&(FOREIGN_USER, FTEvent::Balance(amount).encode())));
}

pub fn check_nft_balance(ft_program: &Program, actor: u64, amount: U256) {
    assert!(ft_program
        .send(FOREIGN_USER, NFTAction::BalanceOf(actor.into()))
        .contains(&(FOREIGN_USER, NFTEvent::BalanceOf(amount).encode())));
}

pub fn mint(ft_program: &Program, actor: u64, amount: u128) {
    assert!(ft_program.send(actor, FTAction::Mint(amount)).contains(&(
        actor,
        FTEvent::Transfer {
            from: 0.into(),
            to: actor.into(),
            amount,
        }
        .encode()
    )));
}

pub fn check_nft_owner(nft_program: &Program, id: U256, owner: u64) {
    assert!(nft_program
        .send(FOREIGN_USER, NFTAction::OwnerOf(id))
        .contains(&(FOREIGN_USER, NFTEvent::OwnerOf(owner.into()).encode())));
}
