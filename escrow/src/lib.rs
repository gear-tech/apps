#![no_std]

use gstd::{async_main, exec, msg, prelude::*, ActorId};

#[cfg(test)]
mod tests;

#[derive(PartialEq)]
enum State {
    AwaitingPayment,
    AwaitingDelivery,
    Completed,
}

#[derive(Default)]
struct Escrow {
    ft_program_id: ActorId,
    contracts: BTreeMap<u128, Contract>,
    id_nonce: u128,
}

fn get(contracts: &mut BTreeMap<u128, Contract>, contract_id: u128) -> &mut Contract {
    if let Some(contract) = contracts.get_mut(&contract_id) {
        contract
    } else {
        panic!("A contract with the {contract_id} ID does not exist");
    }
}

impl Escrow {
    fn create(&mut self, buyer: ActorId, seller: ActorId, amount: u128) {
        let contract_id = self.id_nonce;
        self.id_nonce += 1;

        self.contracts.insert(
            contract_id,
            Contract {
                buyer,
                seller,
                amount,
                state: State::AwaitingPayment,
            },
        );

        msg::reply(escrow_io::Event::Created { contract_id }, 0);
    }

    async fn deposit(&mut self, contract_id: u128) {
        let contract = get(&mut self.contracts, contract_id);

        if msg::source() != contract.buyer {
            panic!("Only a buyer saved in contract can make a deposit");
        }

        if contract.state != State::AwaitingPayment {
            panic!("Escrow already paid or completed");
        }

        msg::send_and_wait_for_reply::<ft_io::Event, _>(
            self.ft_program_id,
            ft_io::Action::Transfer {
                from: contract.buyer,
                to: exec::program_id(),
                amount: contract.amount,
            },
            0,
        )
        .await
        .expect("Error when making the deposit");

        contract.state = State::AwaitingDelivery;

        msg::reply(
            escrow_io::Event::Deposited {
                buyer: contract.buyer,
                amount: contract.amount,
            },
            0,
        );
    }

    async fn confirm(&mut self, contract_id: u128) {
        let contract = get(&mut self.contracts, contract_id);

        if msg::source() != contract.buyer {
            panic!("Only a buyer saved in contract can confirm an escrow")
        }

        if contract.state != State::AwaitingDelivery {
            panic!("Escrow completed or not paid");
        }

        msg::send_and_wait_for_reply::<ft_io::Event, _>(
            self.ft_program_id,
            ft_io::Action::Transfer {
                from: exec::program_id(),
                to: contract.seller,
                amount: contract.amount,
            },
            0,
        )
        .await
        .expect("Error when confirming the escrow");

        contract.state = State::Completed;

        msg::reply(
            escrow_io::Event::Confirmed {
                amount: contract.amount,
                seller: contract.seller,
            },
            0,
        );
    }

    async fn refund(&mut self, contract_id: u128) {
        let contract = get(&mut self.contracts, contract_id);

        if msg::source() != contract.seller {
            panic!("Only a seller saved in contract can refund an escrow")
        }

        if contract.state != State::AwaitingDelivery {
            panic!("Escrow completed or not paid");
        }

        msg::send_and_wait_for_reply::<ft_io::Event, _>(
            self.ft_program_id,
            ft_io::Action::Transfer {
                from: exec::program_id(),
                to: contract.buyer,
                amount: contract.amount,
            },
            0,
        )
        .await
        .expect("Error when refunding the escrow");

        msg::reply(
            escrow_io::Event::Refunded {
                amount: contract.amount,
                buyer: contract.buyer,
            },
            0,
        );
    }

    async fn cancel(&mut self, contract_id: u128) {
        let contract = get(&mut self.contracts, contract_id);

        if contract.state != State::AwaitingPayment {
            panic!("Escrow can't be cancelled if it's completed or paid");
        }

        contract.state = State::Completed;

        msg::reply(
            escrow_io::Event::Cancelled {
                buyer: contract.buyer,
                seller: contract.seller,
                amount: contract.amount,
            },
            0,
        );
    }
}

struct Contract {
    buyer: ActorId,
    seller: ActorId,
    state: State,
    amount: u128,
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
pub extern "C" fn init() {
    let config: escrow_io::InitConfig = msg::load().expect("Unable to decode InitConfig");
    let escrow = Escrow {
        ft_program_id: config.ft_program_id,
        ..Default::default()
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
        escrow_io::Action::Create {
            buyer,
            seller,
            amount,
        } => escrow.create(buyer, seller, amount),
        escrow_io::Action::Deposit { contract_id } => escrow.deposit(contract_id).await,
        escrow_io::Action::Confirm { contract_id } => escrow.confirm(contract_id).await,
        escrow_io::Action::Refund { contract_id } => escrow.refund(contract_id).await,
        escrow_io::Action::Cancel { contract_id } => escrow.cancel(contract_id).await,
    }
}
