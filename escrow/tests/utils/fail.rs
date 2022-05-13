use super::*;

pub fn create(escrow_program: &Program, from: u64, buyer: u64, seller: u64, amount: u128) {
    assert!(escrow_program
        .send(
            from,
            EscrowAction::Create {
                buyer: buyer.into(),
                seller: seller.into(),
                amount,
            },
        )
        .main_failed());
}

pub fn deposit(escrow_program: &Program, account_id: u128, from: u64) {
    assert!(escrow_program
        .send(from, EscrowAction::Deposit(account_id.into()))
        .main_failed());
}

pub fn confirm(escrow_program: &Program, account_id: u128, from: u64) {
    assert!(escrow_program
        .send(from, EscrowAction::Confirm(account_id.into()))
        .main_failed());
}

pub fn refund(escrow_program: &Program, account_id: u128, from: u64) {
    assert!(escrow_program
        .send(from, EscrowAction::Refund(account_id.into()))
        .main_failed());
}

pub fn cancel(escrow_program: &Program, account_id: u128, from: u64) {
    assert!(escrow_program
        .send(from, EscrowAction::Cancel(account_id.into()))
        .main_failed());
}
