#![no_std]

use ft_io::{FTAction, FTEvent};
use gstd::{async_main, exec, msg, prelude::*, ActorId};
use nft_example_io::{Action as NFTAction, Event as NFTEvent};
use primitive_types::U256;
use supply_chain_io::*;

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
    // By default, all tokens transfered to a seller,
    let (mut to, mut amount) = (seller, item.price);

    // but if a seller spent more time than it was agreed...
    if elapsed_time > item.delivery_time {
        // ...and extremely late (more than or exactly 2 times in this example),
        if elapsed_time >= item.delivery_time * 2 {
            // then all tokens refunded to a buyer...
            to = msg::source();
        } else {
            // ...or a half of tokens refunded to a buyer and...
            transfer_tokens(
                ft_program_id,
                exec::program_id(),
                msg::source(),
                (item.price + 1) / 2,
            )
            .await;

            // ...another half transfered to a seller.
            amount /= 2;
        }
    }

    transfer_tokens(ft_program_id, exec::program_id(), to, amount).await;
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

    /// Produces one item with a name and notes and replies with its ID.
    /// Transfers created NFT for an item to a producer.
    ///
    /// Requirements:
    /// * `msg::source()` must be a producer in a supply chain.
    ///
    /// Arguments:
    /// * `name`: an item's name.
    /// * `notes`: an item's notes.
    async fn produce_item(&mut self, name: String, notes: String) {
        self.check_producer();

        // After minting NFT for an item,
        // an item gets ID equal to ID of its NFT.
        let item_id = match msg::send_and_wait_for_reply(self.nft_program_id, NFTAction::Mint, 0)
            .unwrap()
            .await
            .expect("Unable to decode NFTEvent") {
                NFTEvent::Transfer { from, to, token_id } if from == ActorId::default() && to == exec::program_id() => token_id,
                smth_else => panic!("NFTEvent must be Transfer {{ from: ActorId::default(), to: exec::program_id(), .. }} not {smth_else:?}")
            };
        transfer_nft(self.nft_program_id, msg::source(), item_id).await;

        self.items.insert(
            item_id,
            Item {
                info: ItemInfo {
                    name,
                    notes,
                    producer: ActorId::default(),
                    distributor: ActorId::default(),
                    retailer: ActorId::default(),
                    state: ItemState::Produced,
                },
                price: 0,
                delivery_time: 0,
                shipping_time: 0,
            },
        );

        msg::reply(SupplyChainEvent::Produced(item_id), 0).unwrap();
    }

    /// Puts an item up for a sale to a distributor for a given price
    /// on behalf of a producer.
    /// Transfers item's NFT to a supply chain.
    ///
    /// Requirements:
    /// * `msg::source()` must be a producer in a supply chain
    /// and a producer of this item.
    /// * Item's `ItemState` must be `Produced`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    /// * `price`: an item's price.
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

    /// Purchares an item from a producer on behalf of a distributor.
    /// Transfers tokens for purchasing an item to a supply chain
    /// until an item is received (by `receive_by_distributor` function).
    ///
    /// Requirements:
    /// * `msg::source()` must be a distributor in a supply chain.
    /// * Item's `ItemState` must be `ForSaleByProducer`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    /// * `delivery_time`: a time in seconds for which a producer must deliver an item.
    /// A countdown starts after the `ship_by_producer` function is executed.
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

    /// Starts shipping a purchased item to a distributor on behalf of a producer.
    /// Starts countdown for delivery time specified in
    /// `purchare_by_distributor` function.
    ///
    /// Requirements:
    /// * `msg::source()` must be a producer in a supply chain
    /// and a producer of this item.
    /// * Item's `ItemState` must be `PurchasedByDistributor`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    fn ship_by_producer(&mut self, item_id: U256) {
        self.check_producer();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::PurchasedByDistributor);
        assert_eq!(item.info.producer, msg::source());

        item.shipping_time = exec::block_timestamp();

        item.info.state = ItemState::ShippedByProducer;
        msg::reply(SupplyChainEvent::ShippedByProducer(item_id), 0).unwrap();
    }

    /// Receives a shipped item from a producer on behalf of a distrubutor.
    /// Depending on a counted delivery time, transfers tokens for purchasing an item
    /// from supply chain to producer or as a penalty for being late refunds some or
    /// all of them to a distributor.
    /// Transfers item's NFT to a distributor.
    ///
    /// Requirements:
    /// * `msg::source()` must be a distributor in a supply chain
    /// and a distributor of this item.
    /// * Item's `ItemState` must be `ShippedByProducer`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
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

    /// Processes a received item from a producer on behalf of distributor.
    ///
    /// Requirements:
    /// * `msg::source()` must be a distributor in a supply chain
    /// and a distributor of this item.
    /// * Item's `ItemState` must be `ReceivedByDistributor`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    fn process_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ReceivedByDistributor);
        assert_eq!(item.info.distributor, msg::source());

        item.info.state = ItemState::ProcessedByDistributor;
        msg::reply(SupplyChainEvent::ProcessedByDistributor(item_id), 0).unwrap();
    }

    /// Packages a processed item on behalf of distributor.
    ///
    /// Requirements:
    /// * `msg::source()` must be a distributor in a supply chain
    /// and a distributor of this item.
    /// * Item's `ItemState` must be `ProcessedByDistributor`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    fn package_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::ProcessedByDistributor);
        assert_eq!(item.info.distributor, msg::source());

        item.info.state = ItemState::PackagedByDistributor;
        msg::reply(SupplyChainEvent::PackagedByDistributor(item_id), 0).unwrap();
    }

    /// Puts a packaged item up for a sale to a retailer
    /// for a given price on behalf of a distributor.
    /// Transfers item's NFT to a supply chain.
    ///
    /// Requirements:
    /// * `msg::source()` must be a distributor in a supply chain
    /// and a distributor of this item.
    /// * Item's `ItemState` must be `PackagedByDistributor`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    /// * `price`: an item's price.
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

    /// Purchares an item from a distributor on behalf of a retailer.
    /// Transfers tokens for purchasing an item to a supply chain
    /// until an item is received (by `receive_by_retailer` function).
    ///
    /// Requirements:
    /// * `msg::source()` must be a retailer in a supply chain.
    /// * Item's `ItemState` must be `ForSaleByDistributor`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    /// * `delivery_time`: a time in seconds for which a distributor must deliver an item.
    /// A countdown starts after the `ship_by_distributor` function is executed.
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

    /// Starts shipping a purchased item to a retailer on behalf of a distributor.
    /// Starts countdown for delivery time specified in `purchare_by_retailer` function.
    ///
    /// Requirements:
    /// * `msg::source()` must be a distributor in a supply chain
    /// and a distributor of this item.
    /// * Item's `ItemState` must be `PurchasedByRetailer`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    fn ship_by_distributor(&mut self, item_id: U256) {
        self.check_distributor();
        let item = get_item(&mut self.items, item_id);
        assert_eq!(item.info.state, ItemState::PurchasedByRetailer);
        assert_eq!(item.info.distributor, msg::source());

        item.shipping_time = exec::block_timestamp();

        item.info.state = ItemState::ShippedByDistributor;
        msg::reply(SupplyChainEvent::ShippedByDistributor(item_id), 0).unwrap();
    }

    /// Receives a shipped item from a distributor on behalf of a retailer.
    /// Depending on a counted delivery time, transfers tokens for purchasing an item
    /// from supply chain to distributor or as a penalty for being late refunds some or
    /// all of them to a retailer.
    /// Transfers item's NFT to a retailer.
    ///
    /// Requirements:
    /// * `msg::source()` must be a retailer in a supply chain
    /// and a retailer of this item.
    /// * Item's `ItemState` must be `ShippedByDistributor`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
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

    /// Puts a received item from distributor up for a sale to a consumer
    /// for a given price on behalf of a retailer.
    /// Transfers item's NFT to a supply chain.
    ///
    /// Requirements:
    /// * `msg::source()` must be a retailer in a supply chain
    /// and a retailer of this item.
    /// * Item's `ItemState` must be `ReceivedByRetailer`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
    /// * `price`: an item's price.
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

    /// Purchares an item from a retailer.
    /// Transfers tokens for purchasing an item to its retailer.
    /// Transfers item's NFT to a consumer.
    ///
    /// Requirements:
    /// * Item's `ItemState` must be `ForSaleByRetailer`.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
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

    /// Gets item info.
    ///
    /// Arguments:
    /// * `item_id`: an item's ID.
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

            ft_program_id: ActorId::default(),
            nft_program_id: ActorId::default(),
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
