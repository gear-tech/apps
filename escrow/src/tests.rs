use gstd::{Encode, String};
use gtest::{Program, System};

const ZERO_ID: u64 = 0;
const FT_ID: u64 = 2;
const ANY_USER_ID: u64 = 1337;
const BUYER_ID_1: u64 = 12;
const BUYER_ID_2: u64 = 34;
const SELLER_ID_1: u64 = 56;
const SELLER_ID_2: u64 = 78;
const ESCROW_AMOUNT_1: u128 = 12345;
const ESCROW_AMOUNT_2: u128 = 54321;
const CONTRACT_ID_1: u128 = 0;
const CONTRACT_ID_2: u128 = 1;

fn init_system() -> System {
    let system = System::new();
    system.init_logger();

    system
}

fn init_fungible_tokens(sys: &System) -> Program {
    let ft_program = Program::from_file(
        &sys,
        "../target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft_program.send(
        ANY_USER_ID,
        ft_io::InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );
    assert!(res.log().is_empty());

    ft_program
}

fn init_escrow(sys: &System) -> Program {
    let escrow_program = Program::current(&sys);

    let res = escrow_program.send(
        ANY_USER_ID,
        escrow_io::InitConfig {
            ft_program_id: FT_ID.into(),
        },
    );
    assert!(res.log().is_empty());

    escrow_program
}

fn create(escrow_program: &Program, amount: u128, seller: u64, buyer: u64, contract_id: u128) {
    assert!(escrow_program
        .send(
            ANY_USER_ID,
            escrow_io::Action::Create {
                amount,
                seller: seller.into(),
                buyer: buyer.into(),
            },
        )
        .contains(&(
            ANY_USER_ID,
            escrow_io::Event::Created { contract_id }.encode()
        )));
}

fn deposit(escrow_program: &Program, amount: u128, buyer: u64, contract_id: u128) {
    assert!(escrow_program
        .send(buyer, escrow_io::Action::Deposit { contract_id })
        .contains(&(
            buyer,
            escrow_io::Event::Deposited {
                buyer: buyer.into(),
                amount,
            }
            .encode()
        )));
}

fn deposit_fail(escrow_program: &Program, contract_id: u128, user: u64) {
    assert!(escrow_program
        .send(user, escrow_io::Action::Deposit { contract_id },)
        .main_failed());
}

fn confirm(escrow_program: &Program, amount: u128, buyer: u64, seller: u64, contract_id: u128) {
    assert!(escrow_program
        .send(buyer, escrow_io::Action::Confirm { contract_id },)
        .contains(&(
            buyer,
            escrow_io::Event::Confirmed {
                amount,
                seller: seller.into()
            }
            .encode()
        )));
}

fn confirm_fail(escrow_program: &Program, user: u64, contract_id: u128) {
    assert!(escrow_program
        .send(user, escrow_io::Action::Confirm { contract_id })
        .main_failed());
}

fn check_balance(ft_program: &Program, amount: u128, user: u64) {
    assert!(ft_program
        .send(user, ft_io::Action::BalanceOf(user.into()))
        .contains(&(user, ft_io::Event::Balance(amount).encode())));
}

fn mint(ft_program: &Program, user: u64, amount: u128) {
    assert!(ft_program
        .send(user, ft_io::Action::Mint(amount))
        .contains(&(
            user,
            ft_io::Event::Transfer {
                from: ZERO_ID.into(),
                to: user.into(),
                amount,
            }
            .encode()
        )));
}

#[test]
fn two_simple_escrows() {
    const ESCROW_AMOUNT_REMAINDER: u128 = 20000;

    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(
        &ft_program,
        BUYER_ID_1,
        ESCROW_AMOUNT_1 + ESCROW_AMOUNT_REMAINDER,
    );
    mint(
        &ft_program,
        BUYER_ID_2,
        ESCROW_AMOUNT_2 + ESCROW_AMOUNT_REMAINDER,
    );

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );
    create(
        &escrow_program,
        ESCROW_AMOUNT_2,
        SELLER_ID_2,
        BUYER_ID_2,
        CONTRACT_ID_2,
    );

    deposit(&escrow_program, ESCROW_AMOUNT_1, BUYER_ID_1, CONTRACT_ID_1);
    deposit(&escrow_program, ESCROW_AMOUNT_2, BUYER_ID_2, CONTRACT_ID_2);

    confirm(
        &escrow_program,
        ESCROW_AMOUNT_1,
        BUYER_ID_1,
        SELLER_ID_1,
        CONTRACT_ID_1,
    );
    confirm(
        &escrow_program,
        ESCROW_AMOUNT_2,
        BUYER_ID_2,
        SELLER_ID_2,
        CONTRACT_ID_2,
    );

    check_balance(&ft_program, ESCROW_AMOUNT_REMAINDER, BUYER_ID_1);
    check_balance(&ft_program, ESCROW_AMOUNT_1, SELLER_ID_1);

    check_balance(&ft_program, ESCROW_AMOUNT_REMAINDER, BUYER_ID_2);
    check_balance(&ft_program, ESCROW_AMOUNT_2, SELLER_ID_2);
}

#[test]
fn not_enougn_tokens() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let _ft_program = init_fungible_tokens(&system);

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );

    // Should fail because the buyer doesn't have enought tokens to deposit
    deposit_fail(&escrow_program, CONTRACT_ID_1, BUYER_ID_1);
}

#[test]
fn double_pay() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    // Purposely make it possible for the buyer to pay twice
    mint(&ft_program, BUYER_ID_1, ESCROW_AMOUNT_1 * 2);

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );

    deposit(&escrow_program, ESCROW_AMOUNT_1, BUYER_ID_1, CONTRACT_ID_1);

    // Should fail because the buyer tries to make the deposit twice
    deposit_fail(&escrow_program, CONTRACT_ID_1, BUYER_ID_1);

    check_balance(&ft_program, ESCROW_AMOUNT_1, BUYER_ID_1);
}

#[test]
fn seller_tries_to_make_deposit() {
    let system = init_system();

    let escrow_program = init_escrow(&system);

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );

    // Should fail because the seller tries to make the deposit
    deposit_fail(&escrow_program, CONTRACT_ID_1, SELLER_ID_1);
}

#[test]
fn seller_tries_to_confirm_delivery() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(&ft_program, BUYER_ID_1, ESCROW_AMOUNT_1);

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );

    deposit(&escrow_program, ESCROW_AMOUNT_1, BUYER_ID_1, CONTRACT_ID_1);

    // Should fail because seller tries to confirm delivery
    confirm_fail(&escrow_program, SELLER_ID_1, CONTRACT_ID_1);

    check_balance(&ft_program, 0, SELLER_ID_1);
}

#[test]
fn double_confirm() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(&ft_program, BUYER_ID_1, ESCROW_AMOUNT_1);

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );

    deposit(&escrow_program, ESCROW_AMOUNT_1, BUYER_ID_1, CONTRACT_ID_1);

    confirm(
        &escrow_program,
        ESCROW_AMOUNT_1,
        BUYER_ID_1,
        SELLER_ID_1,
        CONTRACT_ID_1,
    );

    // Should fail because buyer tries to confirm delivery twice
    confirm_fail(&escrow_program, BUYER_ID_1, CONTRACT_ID_1);

    check_balance(&ft_program, ESCROW_AMOUNT_1, SELLER_ID_1);
}

#[test]
fn confirm_before_deposit() {
    let system = init_system();

    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(&ft_program, BUYER_ID_1, ESCROW_AMOUNT_1);

    create(
        &escrow_program,
        ESCROW_AMOUNT_1,
        SELLER_ID_1,
        BUYER_ID_1,
        CONTRACT_ID_1,
    );

    // Should fail because buyer tries to confirm delivery before making the deposit
    confirm_fail(&escrow_program, BUYER_ID_1, CONTRACT_ID_1);

    check_balance(&ft_program, 0, SELLER_ID_1);
}
