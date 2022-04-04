use gstd::{Encode, String};
use gtest::{Program, System};

const ZERO_ID: u64 = 0;
const FT_ID: u64 = 2;
const BUYER_ID: u64 = 69;
const SELLER_ID: u64 = 96;
const ESCROW_AMOUNT: u128 = 80000;

fn init_system() -> System {
    let system = System::new();
    system.init_logger();

    system
}

fn init_fungible_tokens(sys: &System, amount: u128) -> Program {
    let ft_program = Program::from_file(
        &sys,
        "../target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft_program.send(
        BUYER_ID,
        ft_io::InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );
    assert!(res.log().is_empty());

    let res = ft_program.send(BUYER_ID, ft_io::Action::Mint(amount));
    assert!(res.contains(&(
        BUYER_ID,
        ft_io::Event::Transfer {
            from: ZERO_ID.into(),
            to: BUYER_ID.into(),
            amount,
        }
        .encode()
    )));

    ft_program
}

fn init_escrow(sys: &System) -> Program {
    let escrow_program = Program::current(&sys);

    let res = escrow_program.send(
        SELLER_ID,
        escrow_io::InitConfig {
            seller: SELLER_ID.into(),
            buyer: BUYER_ID.into(),
            amount: ESCROW_AMOUNT,
            ft_program_id: FT_ID.into(),
        },
    );
    assert!(res.log().is_empty());

    escrow_program
}

#[test]
fn simple_escrow() {
    const ESCROW_AMOUNT_REMAINDER: u128 = 20000;

    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system, ESCROW_AMOUNT + ESCROW_AMOUNT_REMAINDER);

    let mut res = escrow_program.send(BUYER_ID, escrow_io::Action::Deposit);
    assert!(res.contains(&(
        BUYER_ID,
        escrow_io::Event::Deposit {
            buyer: BUYER_ID.into(),
            amount: ESCROW_AMOUNT,
        }
        .encode()
    )));

    res = escrow_program.send(BUYER_ID, escrow_io::Action::ConfirmDelivery);
    assert!(res.contains(&(
        BUYER_ID,
        escrow_io::Event::ConfirmDelivery {
            amount: ESCROW_AMOUNT,
            seller: SELLER_ID.into()
        }
        .encode()
    )));

    res = ft_program.send(BUYER_ID, ft_io::Action::BalanceOf(BUYER_ID.into()));
    assert!(res.contains(&(
        BUYER_ID,
        ft_io::Event::Balance(ESCROW_AMOUNT_REMAINDER).encode()
    )));

    res = ft_program.send(SELLER_ID, ft_io::Action::BalanceOf(SELLER_ID.into()));
    assert!(res.contains(&(SELLER_ID, ft_io::Event::Balance(ESCROW_AMOUNT).encode())));
}

#[test]
fn not_enougn_tokens() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let _ft_program = init_fungible_tokens(&system, 0);

    // Should fail because the buyer doesn't have enought tokens to deposit
    let res = escrow_program.send(BUYER_ID, escrow_io::Action::Deposit);
    assert!(res.main_failed());
}

#[test]
fn double_pay() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    // Purposely make it possible for the buyer to pay twice
    let ft_program = init_fungible_tokens(&system, ESCROW_AMOUNT * 2);

    let mut res = escrow_program.send(BUYER_ID, escrow_io::Action::Deposit);
    assert!(res.contains(&(
        BUYER_ID,
        escrow_io::Event::Deposit {
            buyer: BUYER_ID.into(),
            amount: ESCROW_AMOUNT,
        }
        .encode()
    )));

    // Should fail because the buyer tries to make the deposit twice
    res = escrow_program.send(BUYER_ID, escrow_io::Action::Deposit);
    assert!(res.main_failed());

    res = ft_program.send(BUYER_ID, ft_io::Action::BalanceOf(BUYER_ID.into()));
    assert!(res.contains(&(BUYER_ID, ft_io::Event::Balance(ESCROW_AMOUNT).encode())));
}

#[test]
fn seller_tries_to_make_deposit() {
    let system = init_system();

    let escrow_program = init_escrow(&system);

    // Should fail because the seller tries to make the deposit
    let res = escrow_program.send(SELLER_ID, escrow_io::Action::Deposit);
    assert!(res.main_failed());
}

#[test]
fn seller_tries_to_confirm_delivery() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system, ESCROW_AMOUNT);

    let mut res = escrow_program.send(BUYER_ID, escrow_io::Action::Deposit);
    assert!(res.contains(&(
        BUYER_ID,
        escrow_io::Event::Deposit {
            buyer: BUYER_ID.into(),
            amount: ESCROW_AMOUNT,
        }
        .encode()
    )));

    // Should fail because seller tries to confirm delivery
    res = escrow_program.send(SELLER_ID, escrow_io::Action::ConfirmDelivery);
    assert!(res.main_failed());

    res = ft_program.send(SELLER_ID, ft_io::Action::BalanceOf(SELLER_ID.into()));
    assert!(res.contains(&(SELLER_ID, ft_io::Event::Balance(0).encode())));
}

#[test]
fn double_confirm() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system, ESCROW_AMOUNT);

    let mut res = escrow_program.send(BUYER_ID, escrow_io::Action::Deposit);
    assert!(res.contains(&(
        BUYER_ID,
        escrow_io::Event::Deposit {
            buyer: BUYER_ID.into(),
            amount: ESCROW_AMOUNT,
        }
        .encode()
    )));

    res = escrow_program.send(BUYER_ID, escrow_io::Action::ConfirmDelivery);
    assert!(res.contains(&(
        BUYER_ID,
        escrow_io::Event::ConfirmDelivery {
            amount: ESCROW_AMOUNT,
            seller: SELLER_ID.into()
        }
        .encode()
    )));

    // Should fail because buyer tries to confirm delivery twice
    res = escrow_program.send(BUYER_ID, escrow_io::Action::ConfirmDelivery);
    assert!(res.main_failed());

    res = ft_program.send(SELLER_ID, ft_io::Action::BalanceOf(SELLER_ID.into()));
    assert!(res.contains(&(SELLER_ID, ft_io::Event::Balance(ESCROW_AMOUNT).encode())));
}

#[test]
fn confirm_before_deposit() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system, ESCROW_AMOUNT);

    // Should fail because buyer tries to confirm delivery before making the deposit
    let mut res = escrow_program.send(BUYER_ID, escrow_io::Action::ConfirmDelivery);
    assert!(res.main_failed());

    res = ft_program.send(SELLER_ID, ft_io::Action::BalanceOf(SELLER_ID.into()));
    assert!(res.contains(&(SELLER_ID, ft_io::Event::Balance(0).encode())));
}
