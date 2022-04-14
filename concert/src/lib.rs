use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use codec::{Decode, Encode};
use gear_contract_libraries::erc1155::{erc1155_core::*, io::*};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use scale_info::TypeInfo;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const TICKET_AMOUNT: u128 = 100;

#[derive(Debug, Default)]
pub struct Concert {
    pub tokens: ERC1155State,

    // timings to actually store a timing info about the concerts
    pub timings: BTreeMap<u128, u128>,
    pub creators: BTreeMap<u128, ActorId>,
}

impl StateKeeper for Concert {
    fn get(&self) -> &ERC1155State {
        &self.tokens
    }
    fn get_mut(&mut self) -> &mut ERC1155State {
        &mut self.tokens
    }
}

impl BalanceTrait for Concert {}
impl ERC1155TokenAssert for Concert {}
impl ERC1155Core for Concert {}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertAction {
    Create {
        creator: ActorId,
        name: u128,
        date: u128,
    },
    BuyTicket {
        buyer: ActorId,
        concert_id: u128,
        amount: u128,
    },
    Base(Action),
}

pub trait ConcertCore: ERC1155Core {
    // name -- concert ID to use as a token ID for fungible
    // date -- concert date
    // number_of -- number of tickets
    fn create_concert(&mut self, creator: ActorId, name: u128, date: u128);

    // who buys
    // how many
    fn buy_tickets(&mut self, buyer: ActorId, concert_id: u128, amount: u128);

    // wait till the date to transfer to NFT
    fn wait_till();

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() < 2 {
            return None;
        }
        if bytes[0] == 0 {
            let mut bytes = bytes;
            bytes.remove(0);
            return <Self as ConcertCore>::proc(self, bytes);
        }
        let action = ConcertAction::decode(&mut &bytes[..]).ok()?;
        match action {
            ConcertAction::Create{creator, name, date} => {
                <Self as ConcertCore>::create_concert(self, creator, name, date)
            }
            ConcertAction::BuyTicket {buyer, concert_id, amount} => {
                <Self as ConcertCore>::buy_tickets(self, buyer, concert_id, amount)
            }
            ConcertAction::Base(_) => unreachable!(),
        }
        Some(())
    }
}

impl ConcertCore for Concert {
    fn create_concert(&mut self, creator: ActorId, name: u128, date: u128) {
        // creating a concert means we are minting tickets fungible tokens to a creator address
        // and store a concert id to a datetime
        // and store a concert id to a creator to transfer
        ERC1155Core::mint(self, &creator, &name, TICKET_AMOUNT, None);
        self.timings.insert(name, date);
        self.creators.insert(name, creator);
    }

    fn buy_tickets(&mut self, buyer: ActorId, concert_id: u128, amount: u128) {
        // let _seller = self.creators.get(&concert_id);
        if let Some(seller) = self.creators.get(&concert_id) {
            ERC1155Core::transfer_from(self, &seller, &buyer, &concert_id, amount);
            // since we sold, we should specify metadata!
        } else {
            panic!("CONCERT: Concert must be created first.");
        }
    }

    fn wait_till() {}
}
