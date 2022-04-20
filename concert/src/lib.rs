#![no_std]

use codec::Decode;
use concert_io::*;
use gear_contract_libraries::erc1155::{erc1155_core::*, io::*, state::*};
use gstd::{msg, prelude::*, ActorId};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const NFT_COUNT: u128 = 1;

#[derive(Debug, Default)]
pub struct Concert {
    pub tokens: ERC1155State,

    pub name: u128,
    pub creator: ActorId,
    pub number_of_tickets: u128,

    pub id_counter: u128,
    // user to token id to metadata
    pub metadata: BTreeMap<ActorId, BTreeMap<u128, Option<TokenMetadata>>>,
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
impl ERC1155TokenState for Concert {}
impl ERC1155TokenAssert for Concert {}
impl ERC1155Core for Concert {}

pub trait ConcertCore: ERC1155Core {
    // name -- concert ID to use as a token ID for fungible
    // number_of -- number of tickets
    fn create_concert(&mut self, creator: ActorId, concert_id: u128, no_tickets: u128);

    // who buys
    // how many
    fn buy_tickets(&mut self, concert_id: u128, amount: u128, metadata: Vec<Option<TokenMetadata>>);

    fn hold_concert(&mut self, concert_id: u128);

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() < 2 {
            return None;
        }
        // debug!("bytes[0] = {:?}", bytes[0]);
        if bytes[0] == 0 {
            let mut bytes = bytes;
            bytes.remove(0);
            return <Self as ERC1155Core>::proc(self, bytes);
        }
        let action = ConcertAction::decode(&mut &bytes[..]).ok()?;
        // debug!("action = {:?}", action);
        match action {
            ConcertAction::Create {
                creator,
                concert_id,
                no_tickets,
            } => <Self as ConcertCore>::create_concert(self, creator, concert_id, no_tickets),
            ConcertAction::BuyTicket {
                concert_id,
                amount,
                metadata,
            } => <Self as ConcertCore>::buy_tickets(self, concert_id, amount, metadata),
            ConcertAction::Hold { concert_id } => {
                <Self as ConcertCore>::hold_concert(self, concert_id);
            }
            ConcertAction::Base(_) => unreachable!(),
        }
        Some(())
    }
}

static mut CONTRACT: Option<Concert> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConcertERC1155 = msg::load().expect("Unable to decode InitConfig");
    let mut concert = Concert::default();
    concert.tokens.name = config.name;
    concert.tokens.symbol = config.symbol;
    concert.tokens.base_uri = config.base_uri;
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Vec<u8> = msg::load().expect("Could not load msg");
    let concert = CONTRACT.get_or_insert(Concert::default());
    ConcertCore::proc(concert, action);
}

// #[no_mangle]
// pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
//     let query: Vec<u8> = msg::load().expect("failed to decode input argument");
//     let concert = CONTRACT.get_or_insert(Concert::default());
//     // let encoded = ERC1155Core::proc_state(concert, query).expect("error");
//     let encoded: Option<Vec<u8>>= None;
//     gstd::util::to_leak_ptr(encoded)
// }

impl ConcertCore for Concert {
    fn create_concert(&mut self, creator: ActorId, concert_id: u128, number_of_tickets: u128) {
        self.creator = creator;
        self.id_counter = concert_id;
        self.name = concert_id;
        self.number_of_tickets = number_of_tickets;
        msg::reply(
            ConcertEvent::Creation {
                creator,
                concert_id,
                no_tickets: number_of_tickets,
            },
            0,
        )
        .unwrap();
    }

    fn buy_tickets(&mut self, concert_id: u128, amount: u128, mtd: Vec<Option<TokenMetadata>>) {
        if msg::source() == ZERO_ID {
            panic!("CONCERT: Message from zero address");
        }

        if amount < 1 {
            panic!("CONCERT: Can not buy less than 1 ticket");
        }

        if self.number_of_tickets < amount {
            panic!("CONCERT: Not enought tickets");
        }

        if mtd.len() != amount as usize {
            panic!("CONCERT: Metadata not provided for all the tickets");
        }

        mtd.into_iter().enumerate().for_each(|(_i, meta)| {
            self.id_counter += 1;
            self.metadata
                .entry(msg::source())
                .or_default()
                .insert(self.id_counter + 1, meta);
        });
        // should add comission here
        ERC1155Core::mint(self, &msg::source(), &concert_id, amount, None);

        msg::reply(ConcertEvent::Purchase { concert_id, amount }, 0).unwrap();
    }

    // MINT SEVERAL FOR A USER
    fn hold_concert(&mut self, concert_id: u128) {
        if msg::source() != self.creator {
            panic!("CONCERT: Only creator can hold a concert");
        }
        let _balances = self.get().balances.get(&concert_id).cloned();
        if let Some(balances) = _balances {
            // _amount for now:)
            for (actor, _amount) in balances.iter() {
                // no need to burn, just set balance
                // just a burn analog
                let mut _balance = self
                    .get_mut()
                    .balances
                    .entry(concert_id)
                    .or_default()
                    .insert(*actor, 0);
                let mut ids: Vec<TokenId> = vec![];
                let mut amounts: Vec<TokenId> = vec![];
                let mut meta: Vec<Option<TokenMetadata>> = vec![];
                let _actor_md = self.metadata.get(actor);
                if let Some(actor_md) = _actor_md.cloned() {
                    for (token, token_meta) in actor_md {
                        ids.push(token);
                        amounts.push(NFT_COUNT);
                        meta.push(token_meta);
                    }
                    ERC1155Core::mint_batch(self, actor, &ids, &amounts, meta);
                }
            }
        }
        msg::reply(ConcertEvent::Hold { concert_id }, 0).unwrap();
    }
}
