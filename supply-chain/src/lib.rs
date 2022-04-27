#![no_std]

use ft_io::{FTAction, FTEvent};
use gstd::{async_main, exec, msg, prelude::*, ActorId};
use nft_example_io::{Action as NFTAction, Event as NFTEvent};
use primitive_types::U256;
use supply_chain_io::*;

const ZERO_ID: ActorId = ActorId::new([0; 32]);

struct Item {
    info: ItemInfo,
    price: u128,
    delivery_time: u64,
    shipping_time: u64,
}

struct SupplyChain {
    items: BTreeMap<U256, Item>,

    producers: BTreeSet<ActorId>,
    distributors: BTreeSet<ActorId>,
    retailers: BTreeSet<ActorId>,

    ft_program_id: ActorId,
    nft_program_id: ActorId,
}

fn get_item(items: &mut BTreeMap<U256, Item>, id: U256) -> &mut Item {
    if let Some(item) = items.get_mut(&id) {
        item
    } else {
        panic!("Item with the {id} ID doesn't exist");
    }
}

async fn transfer_tokens(ft_program_id: ActorId, from: ActorId, to: ActorId, amount: u128) {
    msg::send_and_wait_for_reply::<FTEvent, _>(
        ft_program_id,
        FTAction::Transfer { from, to, amount },
        0,
    )
    .unwrap()
    .await
    .expect("Unable to decode FTEvent");
}

async fn transfer_nft(nft_program_id: ActorId, to: ActorId, token_id: U256) {
    msg::send_and_wait_for_reply::<NFTEvent, _>(
        nft_program_id,
        NFTAction::Transfer { to, token_id },
        0,
    )
    .unwrap()
    .await
    .expect("Unable to decode NFTEvent");
}

async fn receive(ft_program_id: ActorId, seller: ActorId, item: &Item) {
    let elapsed_time = exec::block_timestamp() - item.shipping_time;
    // If a seller spent more time than it was agreed...
    let (from, to, amount) = if elapsed_time > item.delivery_time {
        // ...and extremely late (more than 2 times in this example),
        // then all tokens refunded to a buyer...
        if elapsed_time > item.delivery_time * 2 {
            (exec::program_id(), msg::source(), item.price)
        } else {
            // ...or a half of tokens refunded to a buyer and...
            transfer_tokens(
                exec::program_id(),
                msg::source(),
                seller,
                item.price / 2 + 1,
            )
            .await;
            // another half transfered to a seller...
            (ft_program_id, exec::program_id(), item.price / 2)
        }
    } else {
        // ...or all tokens transfered to a seller.
        (exec::program_id(), seller, item.price)
    };
    transfer_tokens(ft_program_id, from, to, amount).await;
}

impl SupplyChain {
    fn check_producer(&self) {
        if !self.producers.contains(&msg::source()) {
            panic!("msg::source() must be a producer");
        }
    }

    fn check_distributor(&self) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }
    }

    fn check_retailer(&self) {
        if !self.retailers.contains(&msg::source()) {
            panic!("msg::source() must be a retailer");
        }
    }

    async fn produce_item(&mut self, name: String, notes: String) {
        self.check_producer();

        // After minting a NFT token for an item,
        // an item gets ID equal to ID of its NFT token.
        let item_id = match msg::send_and_wait_for_reply(self.nft_program_id, NFTAction::Mint, 0)
            .unwrap()
            .await
            .expect("Unable to decode NFTEvent") {
                NFTEvent::Transfer { from: ZERO_ID, to, token_id } if to == exec::program_id() => token_id,
                smth_else => panic!("NFTEvent must be Transfer {{ from: ZERO_ID, to: msg::source(), .. }} not {smth_else:?}")
            };
        transfer_nft(self.nft_program_id, msg::source(), item_id).await;

        self.items.insert(
            item_id,
            Item {
                info: ItemInfo {
                    name,
                    notes,
                    producer: msg::source(),
                    distributor: ZERO_ID,
                    retailer: ZERO_ID,
                    state: ItemState::Produced,
                },
                price: 0,
                delivery_time: 0,
                shipping_time: 0,
            },
        );

        msg::reply(SupplyChainEvent::Produced(item_id), 0).unwrap();
    }

    async fn put_up_for_sale_by_producer(&mut self, item_id: U256, price: u128) {
        self.check_producer();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::Produced);
        assert_eq!(item.info.producer, msg::source());

        item.price = price;
        transfer_nft(self.nft_program_id, exec::program_id(), item_id).await;

        item.info.state = ItemState::ForSaleByProducer;
        msg::reply(SupplyChainEvent::ForSaleByProducer(item_id), 0).unwrap();
    }

    async fn purchare_by_distributor(&mut self, item_id: U256, delivery_time: u64) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ForSaleByProducer);

        transfer_tokens(
            self.ft_program_id,
            msg::source(),
            exec::program_id(),
            item.price,
        )
        .await;
        item.delivery_time = delivery_time;
        item.info.distributor = msg::source();

        item.info.state = ItemState::PurchasedByDistributor;
        msg::reply(
            SupplyChainEvent::PurchasedByDistributor {
                from: item.info.producer,
                item_id,
                price: item.price,
            },
            0,
        )
        .unwrap();
    }

    fn ship_by_producer(&mut self, item_id: U256) {
        self.check_producer();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::PurchasedByDistributor);
        assert_eq!(item.info.producer, msg::source());

        item.shipping_time = exec::block_timestamp();

        item.info.state = ItemState::ShippedByProducer;
        msg::reply(SupplyChainEvent::ShippedByProducer(item_id), 0).unwrap();
    }

    async fn receive_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ShippedByProducer);
        assert_eq!(item.info.distributor, msg::source());

        receive(self.ft_program_id, item.info.producer, item).await;
        transfer_nft(self.nft_program_id, msg::source(), item_id).await;

        item.info.state = ItemState::ReceivedByDistributor;
        msg::reply(
            SupplyChainEvent::ReceivedByDistributor {
                from: item.info.producer,
                item_id,
            },
            0,
        )
        .unwrap();
    }

    fn process_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ReceivedByDistributor);
        assert_eq!(item.info.distributor, msg::source());

        item.info.state = ItemState::ProcessedByDistributor;
        msg::reply(SupplyChainEvent::ProcessedByDistributor(item_id), 0).unwrap();
    }

    fn package_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ProcessedByDistributor);
        assert_eq!(item.info.distributor, msg::source());

        item.info.state = ItemState::PackagedByDistributor;
        msg::reply(SupplyChainEvent::PackagedByDistributor(item_id), 0).unwrap();
    }

    async fn put_up_for_sale_by_distributor(&mut self, item_id: U256, price: u128) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::PackagedByDistributor);
        assert_eq!(item.info.distributor, msg::source());

        item.price = price;
        transfer_nft(self.nft_program_id, exec::program_id(), item_id).await;

        item.info.state = ItemState::ForSaleByDistributor;
        msg::reply(SupplyChainEvent::ForSaleByDistributor { item_id, price }, 0).unwrap();
    }

    async fn purchare_by_retailer(&mut self, item_id: U256, delivery_time: u64) {
        self.check_retailer();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ForSaleByDistributor);

        transfer_tokens(
            self.ft_program_id,
            msg::source(),
            exec::program_id(),
            item.price,
        )
        .await;
        item.delivery_time = delivery_time;
        item.info.retailer = msg::source();

        item.info.state = ItemState::PurchasedByRetailer;
        msg::reply(
            SupplyChainEvent::PurchasedByRetailer {
                from: item.info.distributor,
                item_id,
                price: item.price,
            },
            0,
        )
        .unwrap();
    }

    fn ship_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::PurchasedByRetailer);
        assert_eq!(item.info.distributor, msg::source());

        item.shipping_time = exec::block_timestamp();

        item.info.state = ItemState::ShippedByDistributor;
        msg::reply(SupplyChainEvent::ShippedByDistributor(item_id), 0).unwrap();
    }

    async fn receive_by_retailer(&mut self, item_id: U256) {
        self.check_retailer();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ShippedByDistributor);
        assert_eq!(item.info.retailer, msg::source());

        receive(self.ft_program_id, item.info.distributor, item).await;
        transfer_nft(self.nft_program_id, msg::source(), item_id).await;

        item.info.state = ItemState::ReceivedByRetailer;
        msg::reply(
            SupplyChainEvent::ReceivedByRetailer {
                from: item.info.distributor,
                item_id,
            },
            0,
        )
        .unwrap();
    }

    async fn put_up_for_sale_by_retailer(&mut self, item_id: U256, price: u128) {
        self.check_retailer();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ReceivedByRetailer);
        assert_eq!(item.info.retailer, msg::source());

        item.price = price;
        transfer_nft(self.nft_program_id, exec::program_id(), item_id).await;

        item.info.state = ItemState::ForSaleByRetailer;
        msg::reply(SupplyChainEvent::ForSaleByRetailer { item_id, price }, 0).unwrap();
    }

    async fn purchare_by_consumer(&mut self, item_id: U256) {
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ForSaleByRetailer);

        transfer_tokens(
            self.ft_program_id,
            msg::source(),
            item.info.retailer,
            item.price,
        )
        .await;
        transfer_nft(self.nft_program_id, msg::source(), item_id).await;

        item.info.state = ItemState::PurchasedByConsumer;
        msg::reply(
            SupplyChainEvent::PurchasedByConsumer {
                from: item.info.retailer,
                item_id,
                price: item.price,
            },
            0,
        )
        .unwrap();
    }

    fn get_item_info(&mut self, item_id: U256) {
        let item = get_item(&mut self.items, item_id);

        msg::reply(
            SupplyChainEvent::ItemInfo {
                item_id,
                info: item.info.clone(),
            },
            0,
        )
        .unwrap();
    }
}

impl Default for SupplyChain {
    fn default() -> Self {
        Self {
            items: BTreeMap::new(),

            producers: BTreeSet::new(),
            distributors: BTreeSet::new(),
            retailers: BTreeSet::new(),

            ft_program_id: ZERO_ID,
            nft_program_id: ZERO_ID,
        }
    }
}

static mut SUPPLY_CHAIN: Option<SupplyChain> = None;

#[no_mangle]
pub extern "C" fn init() {
    let InitSupplyChain {
        producers,
        distributors,
        retailers,
        ft_program_id,
        nft_program_id,
    } = msg::load().expect("Unable to decode InitSupplyChain");
    let supply_chain = SupplyChain {
        producers,
        distributors,
        retailers,
        ft_program_id,
        nft_program_id,
        ..Default::default()
    };
    unsafe {
        SUPPLY_CHAIN = Some(supply_chain);
    }
}

#[async_main]
pub async fn main() {
    let action = msg::load().expect("Unable to decode SupplyChainAction");
    let supply_chain = unsafe { SUPPLY_CHAIN.get_or_insert(Default::default()) };
    match action {
        SupplyChainAction::Produce { name, notes } => supply_chain.produce_item(name, notes).await,
        SupplyChainAction::PutUpForSaleByProducer { item_id, price } => {
            supply_chain
                .put_up_for_sale_by_producer(item_id, price)
                .await;
        }
        SupplyChainAction::PurchaseByDistributor {
            item_id,
            delivery_time,
        } => {
            supply_chain
                .purchare_by_distributor(item_id, delivery_time)
                .await;
        }
        SupplyChainAction::ShipByProducer(item_id) => supply_chain.ship_by_producer(item_id),
        SupplyChainAction::ReceiveByDistributor(item_id) => {
            supply_chain.receive_by_distributor(item_id).await;
        }
        SupplyChainAction::ProcessByDistributor(item_id) => {
            supply_chain.process_by_distributor(item_id);
        }
        SupplyChainAction::PackageByDistributor(item_id) => {
            supply_chain.package_by_distributor(item_id);
        }
        SupplyChainAction::PutUpForSaleByDistributor { item_id, price } => {
            supply_chain
                .put_up_for_sale_by_distributor(item_id, price)
                .await;
        }
        SupplyChainAction::PurchaseByRetailer {
            item_id,
            delivery_time,
        } => {
            supply_chain
                .purchare_by_retailer(item_id, delivery_time)
                .await;
        }
        SupplyChainAction::ShipByDistributor(item_id) => {
            supply_chain.ship_by_distributor(item_id);
        }
        SupplyChainAction::ReceiveByRetailer(item_id) => {
            supply_chain.receive_by_retailer(item_id).await;
        }
        SupplyChainAction::PutUpForSaleByRetailer { item_id, price } => {
            supply_chain
                .put_up_for_sale_by_retailer(item_id, price)
                .await;
        }
        SupplyChainAction::PurchaseByConsumer(item_id) => {
            supply_chain.purchare_by_consumer(item_id).await;
        }
        SupplyChainAction::GetItemInfo(item_id) => supply_chain.get_item_info(item_id),
    }
}
