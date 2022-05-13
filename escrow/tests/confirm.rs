pub mod utils;
use utils::*;

#[test]
fn not_buyer_confirm() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_ft(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        ACCOUNT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    check::deposit(&escrow_program, ACCOUNT[0], BUYER[0], AMOUNT[0]);
    // Should fail because not the buyer saved in the account tries to confirm it.
    fail::confirm(&escrow_program, ACCOUNT[0], FOREIGN_USER);
    fail::confirm(&escrow_program, ACCOUNT[0], BUYER[1]);
    fail::confirm(&escrow_program, ACCOUNT[0], SELLER[0]);
    check_balance(&ft_program, SELLER[0], 0);
}

#[test]
fn double_confirm() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_ft(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        ACCOUNT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    check::deposit(&escrow_program, ACCOUNT[0], BUYER[0], AMOUNT[0]);
    check::confirm(&escrow_program, ACCOUNT[0], BUYER[0], SELLER[0], AMOUNT[0]);
    // Should fail because the buyer tries to confirm the account twice.
    fail::confirm(&escrow_program, ACCOUNT[0], BUYER[0]);
    check_balance(&ft_program, SELLER[0], AMOUNT[0]);
}

#[test]
fn confirm_before_deposit() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_ft(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        ACCOUNT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    // Should fail because the buyer tries to confirm the account before making a deposit.
    fail::confirm(&escrow_program, ACCOUNT[0], BUYER[0]);
    check_balance(&ft_program, SELLER[0], 0);
}

#[test]
fn interact_after_confirm() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_ft(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        ACCOUNT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    check::deposit(&escrow_program, ACCOUNT[0], BUYER[0], AMOUNT[0]);
    check::confirm(&escrow_program, ACCOUNT[0], BUYER[0], SELLER[0], AMOUNT[0]);

    // All of this should fail because nobody can interact with an account after confirming it.
    fail::deposit(&escrow_program, ACCOUNT[0], BUYER[0]);
    fail::refund(&escrow_program, ACCOUNT[0], SELLER[0]);
    fail::confirm(&escrow_program, ACCOUNT[0], BUYER[0]);
    fail::cancel(&escrow_program, ACCOUNT[0], SELLER[0]);
}
