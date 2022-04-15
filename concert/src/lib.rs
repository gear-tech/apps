use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use codec::{Decode, Encode};
use gear_contract_libraries::erc1155::{erc1155_core::*, io::*};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use scale_info::TypeInfo;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const TICKET_AMOUNT: u128 = 100;
const NFT_COUNT: u128 = 1;

#[derive(Debug, Default)]
pub struct Concert {
    pub tokens: ERC1155State,

    pub timing u128;
    pub name u128;
    pub creator ActorId;
    pub number_of_tickets: u128;

    pub id_counter u128;
    // user to token id to metadata
    pub metadata: BTreeMap<ActorId, BTreeMap<u128, Option<TokenMetadata>> >
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
        metadata: Vec<Option<TokenMetadata>>,
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
    fn buy_tickets(&mut self, buyer: ActorId, concert_id: u128, amount: u128, metadata: Vec<Option<TokenMetadata>>);

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
            ConcertAction::BuyTicket {buyer, concert_id, amount, metadata} => {
                <Self as ConcertCore>::buy_tickets(self, buyer, concert_id, amount, metadata)
            }
            ConcertAction::Base(_) => unreachable!(),
        }
        Some(())
    }
}

impl ConcertCore for Concert {
    fn create_concert(&mut self, creator: ActorId, name: u128, date: u128, numbet_of_tickets: u128) {
        self.timing = date;
        self.creator = creator;
        self.id_counter = name;
        self.name = name;
        self.number_of_tickets = number_of_tickets
    }

    fn buy_tickets(&mut self, concert_id: u128, amount: u128, metadata: Vec<Option<TokenMetadata>>) {
        if msg::source() == ZERO_ID {
            panic!("CONCERT: Message from zero address");
        }

        if amount < 1 {
            panic!("CONCERT: Can not buy less than 1 ticket");
        }

        if self.number_of_tickets < amount {
            panic!("CONCERT: Not enought tickets");
        }

        if len(metadata) != amount {
            panic!("CONCERT: Metadata not provided for all the tickets");
        }

        for i in 0..amount {
            self.id_counter++;
            let meta_id = self.id_counter;
            self.metadata.entry(msg::source()).or_default().insert(&meta_id, metadata[i]);
        }

        // should add comission here
        ERC1155Core::mint(self, &msg::source(), &concert_id, amount, None);
    }

    fn wait_till() {
        // when the time comes mint
        // addresses of holders
        // amounts of holders
        for (actor, amount) in &balances {
            ERC1155Core::burn(self, &self.name, amount);
            let ids: Vec<TokenId> = vec![];
            let amounts: Vec<TokenId> = vec![];
            let meta: Vec<Option<TokenMetadata>> = vec![];
            let actor_md = self.metadata.get(&actor);
            for (token, token_meta) in &actor_md {
                ids.push(token);
                amounts.push(NFT_COUNT);
                meta.push(token_meta);
            }
            ERC1155Core::mint_batch(self, &actor, &ids, &amounts, meta);
        }
    }
}
