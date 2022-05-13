pub mod utils;
use utils::*;

#[test]
fn cancel_paid() {
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
    // Should fail because the buyer/seller tries to close the paid account.
    fail::cancel(&escrow_program, ACCOUNT[0], BUYER[0]);
    fail::cancel(&escrow_program, ACCOUNT[0], SELLER[0]);
}

#[test]
fn foreign_user_cancel() {
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
    // Should fail because not the buyer/seller saved in the account tries to close it.
    fail::cancel(&escrow_program, ACCOUNT[0], FOREIGN_USER);
}

#[test]
fn interact_after_cancel() {
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
    check::cancel(
        &escrow_program,
        ACCOUNT[0],
        BUYER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );

    // All of this should fail because nobody can interact with a closed account.
    fail::deposit(&escrow_program, ACCOUNT[0], BUYER[0]);
    fail::refund(&escrow_program, ACCOUNT[0], SELLER[0]);
    fail::confirm(&escrow_program, ACCOUNT[0], BUYER[0]);
    fail::cancel(&escrow_program, ACCOUNT[0], SELLER[0]);
}
