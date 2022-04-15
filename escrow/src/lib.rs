#![no_std]

//Includes only the minimal gstd components for this smart contract
use gstd::{
    msg::{self, CodecMessageFuture},
    exec, 
    prelude::*, 
    ActorId
};

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
    program_id: ActorId,
    //Current escrow State
    current_state: State,
    //Buyer address
    buyer: ActorId,
    //Seller payable address, it'll receive funds after the sender makes a deposit
    seller: ActorId,
    //Amount to transfer
    amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum FTAction {
    Mint(u128),
    Burn(u128),
    Transfer {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        to: ActorId,
        amount: u128,
    },
    TotalSupply,
    BalanceOf(ActorId),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum FTEvent {
    Transfer {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    TotalSupply(u128),
    Balance(u128),
}


pub fn transfer_tokens(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) -> CodecMessageFuture<FTEvent> {
    msg::send_and_wait_for_reply(
        *token_id,
        FTAction::Transfer {
            from: *from,
            to: *to,
            amount,
        },
        0,
    ).unwrap()
}

impl Escrow {
    //Validates the sender is the same than buyer
    fn only_buyer(&mut self) -> bool {
        return msg::source() != self.buyer;
    }

    //The buyer makes a deposit only when a payment is pending
    //and the seller receives the funds
    fn deposit(&mut self) {
        if matches!(self.current_state, State::PendingPayment) {
            panic!("Already Payed");
        }
        if !self.only_buyer() {
            panic!("Only buyer is able to call this function");
        }

        //Sends the transfer from buyer to smart contract
        transfer_tokens(
            &self.program_id,
            &self.buyer,
            &exec::program_id(),
            self.amount,
        );

        self.current_state = State::PendingDelivery;
    }

    //The buyer confirms the delivery only when the delivery is
    //pending, the balance is transfered to the seller
    //and  the  transaction is finished.
    fn confirm_delivery(&mut self) {
        if matches!(self.current_state, State::PendingDelivery) {
            panic!("Cannot confirm delivery");
        }
        if !self.only_buyer() {
            panic!("Only buyer is able to call this function");
        }

        //Sends the transfer from buyer to seller
        transfer_tokens(
            &self.program_id,
            &exec::program_id(),
            &self.seller,
            self.amount,
        );

        self.current_state = State::Finished;
    }
}

#[derive(Debug, Decode, Encode)]
pub enum EscrowActions {
    Deposit(),
    ConfirmDeliery(),
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: EscrowActions = msg::load().expect("Could not load Action");
    let ft = ESCROW.get_or_insert(Default::default());

    match action {
        EscrowActions::Deposit() => {
            ft.deposit();
        }
        EscrowActions::ConfirmDeliery() => {
            ft.confirm_delivery();
        }
    }
}


