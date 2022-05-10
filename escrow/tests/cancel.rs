pub mod utils;
use utils::*;

#[test]
fn cancel_paid() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        CONTRACT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    check::deposit(&escrow_program, CONTRACT[0], BUYER[0], AMOUNT[0]);
    // Should fail because a buyer/seller tries to cancel a paid contract.
    fail::cancel(&escrow_program, CONTRACT[0], BUYER[0]);
    fail::cancel(&escrow_program, CONTRACT[0], SELLER[0]);
}

#[test]
fn foreign_user_cancel() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        CONTRACT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    // Should fail because not a buyer/seller saved in a contract tries to cancel.
    fail::cancel(&escrow_program, CONTRACT[0], FOREIGN_USER);
}

#[test]
fn interact_after_cancel() {
    let system = init_system();
    let escrow_program = init_escrow(&system);
    let ft_program = init_fungible_tokens(&system);

    mint(&ft_program, BUYER[0], AMOUNT[0]);
    check::create(
        &escrow_program,
        CONTRACT[0],
        SELLER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );
    check::cancel(
        &escrow_program,
        CONTRACT[0],
        BUYER[0],
        BUYER[0],
        SELLER[0],
        AMOUNT[0],
    );

    // All of this should fail because nobody can interact with a contract after cancel.
    fail::deposit(&escrow_program, CONTRACT[0], BUYER[0]);
    fail::refund(&escrow_program, CONTRACT[0], SELLER[0]);
    fail::confirm(&escrow_program, CONTRACT[0], BUYER[0]);
    fail::cancel(&escrow_program, CONTRACT[0], SELLER[0]);
}
