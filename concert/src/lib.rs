#![no_std]

use concert_io::*;
use gstd::{msg, prelude::*, ActorId};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const NFT_COUNT: u128 = 1;

#[derive(Debug, Default)]
pub struct Concert {
    pub owner_id: ActorId,
    pub contract_id: ActorId,

    pub name: TokenId,
    pub creator: ActorId,
    pub number_of_tickets: u128,
    pub date: u128,

    pub buyers: BTreeSet<ActorId>,

    pub id_counter: u128,
    // user to token id to metadata
    pub metadata: BTreeMap<ActorId, BTreeMap<u128, Option<TokenMetadata>>>,
}

static mut CONTRACT: Option<Concert> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConcert = msg::load().expect("Unable to decode InitConfig");
    let concert = Concert {
        owner_id: config.owner_id,
        contract_id: config.mtk_contract,
        ..Default::default()
    };
    CONTRACT = Some(concert);
}

#[gstd::async_main]
async unsafe fn main() {
    let action: ConcertAction = msg::load().expect("Could not load Action");
    let concert: &mut Concert = unsafe { CONTRACT.get_or_insert(Concert::default()) };
    match action {
        ConcertAction::Create {
            creator,
            concert_id,
            no_tickets,
            date,
        } => concert.create_concert(creator, concert_id, no_tickets, date),
        ConcertAction::Hold { concert_id } => concert.hold_concert(concert_id).await,
        ConcertAction::BuyTickets {
            concert_id,
            amount,
            metadata,
        } => concert.buy_tickets(concert_id, amount, metadata).await,
    }
}

impl Concert {
    fn create_concert(
        &mut self,
        creator: ActorId,
        concert_id: u128,
        number_of_tickets: u128,
        date: u128,
    ) {
        self.creator = creator;
        self.id_counter = concert_id;
        self.name = concert_id;
        self.number_of_tickets = number_of_tickets;
        self.date = date;
        msg::reply(
            ConcertEvent::Creation {
                creator,
                concert_id,
                no_tickets: number_of_tickets,
                date,
            },
            0,
        )
        .unwrap();
    }

    async fn buy_tickets(
        &mut self,
        concert_id: u128,
        amount: u128,
        mtd: Vec<Option<TokenMetadata>>,
    ) {
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

        for meta in mtd {
            self.id_counter += 1;
            self.metadata
                .entry(msg::source())
                .or_default()
                .insert(self.id_counter + 1, meta);
        }

        self.buyers.insert(msg::source());

        let _: MTKEvent = msg::send_and_wait_for_reply(
            self.contract_id,
            MTKAction::Mint {
                account: msg::source(),
                id: concert_id,
                amount,
                meta: None,
            },
            0,
        )
        .unwrap()
        .await
        .expect("CONCERT: Error minting concert tokens");

        msg::reply(ConcertEvent::Purchase { concert_id, amount }, 0).unwrap();
    }

    // MINT SEVERAL FOR A USER
    async fn hold_concert(&mut self, concert_id: u128) {
        if msg::source() != self.creator {
            panic!("CONCERT: Only creator can hold a concert");
        }
        // get balances from a contract
        let accounts: Vec<_> = self.buyers.clone().into_iter().collect();
        let tokens: Vec<TokenId> = iter::repeat(self.name).take(accounts.len()).collect();

        let balance_response: MTKEvent = msg::send_and_wait_for_reply(
            self.contract_id,
            MTKAction::BalanceOfBatch {
                accounts,
                ids: tokens,
            },
            0,
        )
        .unwrap()
        .await
        .expect("CONCERT: Error getting balances from the contract");

        let balances: Vec<BalanceOfBatchReply> =
            if let MTKEvent::BalanceOfBatch(balance_response) = balance_response {
                balance_response
            } else {
                Vec::new()
            };
        // we know each user balance now
        for balance in &balances {
            let _: MTKEvent = msg::send_and_wait_for_reply(
                self.contract_id,
                MTKAction::Burn {
                    id: balance.id,
                    amount: balance.amount,
                },
                0,
            )
            .unwrap()
            .await
            .expect("CONCERT: Error burning balances");
        }

        for actor in &self.buyers {
            let mut ids = vec![];
            let mut amounts = vec![];
            let mut meta = vec![];
            let actor_metadata = self.metadata.get(actor);
            if let Some(actor_md) = actor_metadata.cloned() {
                for (token, token_meta) in actor_md {
                    ids.push(token);
                    amounts.push(NFT_COUNT);
                    meta.push(token_meta);
                }

                let _: MTKEvent = msg::send_and_wait_for_reply(
                    self.contract_id,
                    MTKAction::MintBatch {
                        account: *actor,
                        ids,
                        amounts,
                        meta,
                    },
                    0,
                )
                .unwrap()
                .await
                .expect("CONCERT: Error minging tickets");
            }
        }
        msg::reply(ConcertEvent::Hold { concert_id }, 0).unwrap();
    }
}
