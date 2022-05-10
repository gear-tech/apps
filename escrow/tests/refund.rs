pub mod utils;
use utils::*;

#[test]
fn refund_not_paid() {
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
    // Should fail because a seller tries to refund an unpaid contract.
    fail::refund(&escrow_program, CONTRACT[0], SELLER[0]);
}

#[test]
fn not_seller_refund() {
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
    // Should fail because not a seller saved in a contract tries to refund.
    fail::refund(&escrow_program, CONTRACT[0], FOREIGN_USER);
    fail::refund(&escrow_program, CONTRACT[0], BUYER[0]);
    fail::refund(&escrow_program, CONTRACT[0], SELLER[1]);
}
