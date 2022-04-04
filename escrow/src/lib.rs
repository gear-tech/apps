#![no_std]

use gstd::{async_main, exec, msg, prelude::*, ActorId};

#[cfg(test)]
mod tests;

#[derive(PartialEq)]
enum State {
    AwaitingPayment,
    AwaitingDelivery,
    Complete,
}

impl Default for State {
    fn default() -> Self {
        Self::AwaitingPayment
    }
}

#[derive(Default)]
struct Escrow {
    buyer: ActorId,
    seller: ActorId,
    ft_program_id: ActorId,
    state: State,
    amount: u128,
}

impl Escrow {
    async fn deposit(&mut self) {
        if msg::source() != self.buyer {
            panic!("Seller can't make deposit");
        }

        if self.state != State::AwaitingPayment {
            panic!("Already paid");
        }

        msg::send_and_wait_for_reply::<ft_io::Event, _>(
            self.ft_program_id,
            ft_io::Action::Transfer {
                from: self.buyer,
                to: exec::program_id(),
                amount: self.amount,
            },
            0,
        )
        .await
        .expect("Error in deposit");

        self.state = State::AwaitingDelivery;

        msg::reply(
            escrow_io::Event::Deposit {
                buyer: self.buyer,
                amount: self.amount,
            },
            0,
        );
    }

    async fn confirm_delivery(&mut self) {
        if msg::source() != self.buyer {
            panic!("Seller can't confirm delivery")
        }

        if self.state != State::AwaitingDelivery {
            panic!("Escrow completed or not paid");
        }

        msg::send_and_wait_for_reply::<ft_io::Event, _>(
            self.ft_program_id,
            ft_io::Action::Transfer {
                from: exec::program_id(),
                to: self.seller,
                amount: self.amount,
            },
            0,
        )
        .await
        .expect("Error in confirming delivery");

        self.state = State::Complete;

        msg::reply(
            escrow_io::Event::ConfirmDelivery {
                amount: self.amount,
                seller: self.seller,
            },
            0,
        );
    }
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
pub extern "C" fn init() {
    let config: escrow_io::InitConfig = msg::load().expect("Unable to decode InitConfig");
    let escrow = Escrow {
        buyer: config.buyer,
        seller: config.seller,
        ft_program_id: config.ft_program_id,
        amount: config.amount,
        state: State::AwaitingPayment,
    };
    unsafe {
        ESCROW = Some(escrow);
    }
}

#[async_main]
pub async fn main() {
    let action: escrow_io::Action = msg::load().expect("Unable to decode Action");
    let escrow = unsafe { ESCROW.get_or_insert(Default::default()) };
    match action {
        escrow_io::Action::Deposit => escrow.deposit().await,
        escrow_io::Action::ConfirmDelivery => escrow.confirm_delivery().await,
    }
}
