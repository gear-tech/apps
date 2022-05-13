use super::*;

pub fn create(
    escrow_program: &Program,
    account_id: u128,
    from: u64,
    buyer: u64,
    seller: u64,
    amount: u128,
) {
    assert!(escrow_program
        .send(
            from,
            EscrowAction::Create {
                buyer: buyer.into(),
                seller: seller.into(),
                amount,
            },
        )
        .contains(&(from, EscrowEvent::Created(account_id.into()).encode())));
}

pub fn deposit(escrow_program: &Program, account_id: u128, buyer: u64, amount: u128) {
    assert!(escrow_program
        .send(buyer, EscrowAction::Deposit(account_id.into()))
        .contains(&(
            buyer,
            EscrowEvent::Deposited {
                buyer: buyer.into(),
                amount,
            }
            .encode()
        )));
}

pub fn confirm(escrow_program: &Program, account_id: u128, buyer: u64, seller: u64, amount: u128) {
    assert!(escrow_program
        .send(buyer, EscrowAction::Confirm(account_id.into()))
        .contains(&(
            buyer,
            EscrowEvent::Confirmed {
                seller: seller.into(),
                amount,
            }
            .encode()
        )));
}

pub fn refund(escrow_program: &Program, account_id: u128, buyer: u64, seller: u64, amount: u128) {
    assert!(escrow_program
        .send(seller, EscrowAction::Refund(account_id.into()))
        .contains(&(
            seller,
            EscrowEvent::Refunded {
                buyer: buyer.into(),
                amount
            }
            .encode()
        )));
}

pub fn cancel(
    escrow_program: &Program,
    account_id: u128,
    from: u64,
    buyer: u64,
    seller: u64,
    amount: u128,
) {
    assert!(escrow_program
        .send(from, EscrowAction::Cancel(account_id.into()))
        .contains(&(
            from,
            EscrowEvent::Cancelled {
                buyer: buyer.into(),
                seller: seller.into(),
                amount
            }
            .encode()
        )));
}
