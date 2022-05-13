pub mod utils;
use utils::*;

#[test]
fn refund_not_paid() {
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
    // Should fail because the seller tries to refund tokens from the unpaid account.
    fail::refund(&escrow_program, ACCOUNT[0], SELLER[0]);
}

#[test]
fn not_seller_refund() {
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
    // Should fail because not the seller saved in the account tries to refund.
    fail::refund(&escrow_program, ACCOUNT[0], FOREIGN_USER);
    fail::refund(&escrow_program, ACCOUNT[0], BUYER[0]);
    fail::refund(&escrow_program, ACCOUNT[0], SELLER[1]);
}
