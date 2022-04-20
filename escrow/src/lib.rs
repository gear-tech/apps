#![no_std]

//Includes only the minimal gstd components for this smart contract
use gstd::{
    msg::{self},
    exec, 
    prelude::*, 
    ActorId
};
use ft_io::{FTAction, FTEvent};

//// The available states during the escrow
enum State {
    PendingPayment,
    PendingDelivery,
    Finished,
}

//Default status for State enum data type
impl Default for State {
    fn default() -> Self {
        Self::PendingPayment
    }
}

#[derive(Default)]
struct Escrow {
    ft_token_id: ActorId,
    //Current escrow State
    current_state: State,
    //Buyer address
    buyer: ActorId,
    //Seller payable address, it'll receive funds after the sender makes a deposit
    seller: ActorId,
    //Amount to transfer
    amount: u128,
}

pub async fn transfer_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let _transfer_response: FTEvent = msg::send_and_wait_for_reply(
        *token_id,
        FTAction::Transfer {
            from: *from,
            to: *to,
            amount,
        },
        0,
    ).unwrap()
    .await
    .expect("Error in transfer");
}

#[derive(Debug, Decode, Encode)]
pub enum EscrowActions {
    Deposit(),
    ConfirmDeliery(),
}

#[derive(Decode, Encode)]
pub enum EscrowEvent {
    Deposit {
        buyer: ActorId,
        amount: u128,
    },
    Confirm {
        seller: ActorId,
        amount: u128,
    },
}

impl Escrow {
    //Validates the sender is the same than buyer
    fn only_buyer(&mut self) -> bool {
        return msg::source() != self.buyer;
    }

    //The buyer makes a deposit only when a payment is pending
    //and the seller receives the funds
    async fn deposit(&mut self) {
        if matches!(self.current_state, State::PendingPayment) {
            panic!("Already Payed");
        }
        if !self.only_buyer() {
            panic!("Only buyer is able to call this function");
        }

        //Sends the transfer from buyer to smart contract
        transfer_tokens(
            &self.ft_token_id,
            &self.buyer,
            &exec::program_id(),
            self.amount,
        ).await;

        self.current_state = State::PendingDelivery;

        msg::reply(
            EscrowEvent::Deposit {
                buyer: self.buyer,
                amount: self.amount,
            },
            0,
        )
        .unwrap();
    }

    //The buyer confirms the delivery only when the delivery is
    //pending, the balance is transfered to the seller
    //and  the  transaction is finished.
    async fn confirm_delivery(&mut self) {
        if matches!(self.current_state, State::PendingDelivery) {
            panic!("Cannot confirm delivery");
        }
        if !self.only_buyer() {
            panic!("Only buyer is able to call this function");
        }

        //Sends the transfer from buyer to seller
        transfer_tokens(
            &self.ft_token_id,
            &exec::program_id(),
            &self.seller,
            self.amount,
        ).await;

        self.current_state = State::Finished;

        msg::reply(
            EscrowEvent::Confirm {
                seller: self.seller,
                amount: self.amount,
            },
            0,
        )
        .unwrap();
    }
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
pub async unsafe extern "C" fn handle() {
    let action: EscrowActions = msg::load().expect("Could not load Action");
    let escrow = ESCROW.get_or_insert(Default::default());

    match action {
        EscrowActions::Deposit() => {
            escrow.deposit().await;
        }
        EscrowActions::ConfirmDeliery() => {
            escrow.confirm_delivery().await;
        }
    }
}

#[cfg(test)]
mod tests {

    extern crate std;
    use gtest::{Program, System};

    pub const _FT: u64 = 2;
    pub const _FOREIGN_USER: u64 = 2;
    pub const _BUYER: [u64; 2] = [1, 2];
    pub const _SELLER: [u64; 2] = [3, 4];
    pub const _AMOUNT: [u128; 2] = [50, 200];
    pub const _CONTRACT: [u128; 2] = [1, 2];

    #[test]
    fn wrong_buyer_deposit() {
        let system = System::new();
        let program = Program::current(&system);

        system.init_logger();

        assert!(program
        .send(_FOREIGN_USER, super::EscrowActions::Deposit())
        .main_failed());
    }
}
