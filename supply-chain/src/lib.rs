#![no_std]

use gstd::{async_main, msg, ActorId, BTreeMap, BTreeSet, String};
use supply_chain_io::{InitSupplyChain, ItemInfo, ItemState, SupplyChainAction, SupplyChainEvent};

mod escrow;
use escrow::{confirm, create, deposit};
struct Item {
    info: ItemInfo,
    price: u128,
    contract_id: u128,
}

struct SupplyChain {
    item_id_nonce: u128,
    items: BTreeMap<u128, Item>,

    owner: ActorId,
    producers: BTreeSet<ActorId>,
    distributors: BTreeSet<ActorId>,
    retailers: BTreeSet<ActorId>,

    escrow_program_id: ActorId,
}

fn get_item(items: &mut BTreeMap<u128, Item>, id: u128) -> &mut Item {
    if let Some(item) = items.get_mut(&id) {
        item
    } else {
        panic!("Item with the {id} ID does not exist");
    }
}

impl SupplyChain {
    fn produce_item(&mut self, name: String, notes: String) {
        if !self.producers.contains(&msg::source()) {
            panic!("msg::source() must be a producer");
        }

        let item_id = self.item_id_nonce;
        self.item_id_nonce += 1;

        self.items.insert(
            self.item_id_nonce,
            Item {
                info: ItemInfo {
                    producer: msg::source(),
                    distributor: 0.into(),
                    retailer: 0.into(),
                    state: ItemState::Produced,
                },
                price: 0,
                contract_id: 0,
            },
        );

        msg::reply(
            SupplyChainEvent::Produced {
                item_id,
                name,
                notes,
            },
            0,
        )
        .unwrap();
    }

    fn put_up_for_sale_by_producer(&mut self, item_id: u128, price: u128) {
        if !self.producers.contains(&msg::source()) {
            panic!("msg::source() must be a producer");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::Produced {
            panic!("ItemState must be ItemState::Produced");
        }

        item.price = price;
        item.info.state = ItemState::ForSaleByProducer;

        msg::reply(SupplyChainEvent::ForSaleByProducer { item_id, price }, 0)
            .expect("Failed to reply with SupplyChainEvent::ForSaleByProducer");
    }

    async fn purchare_by_distributor(&mut self, item_id: u128) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::ForSaleByProducer {
            panic!("ItemState must be ItemState::ForSaleByProducer");
        }

        let contract_id = create(self.escrow_program_id, item.info.producer, item.price).await;
        let (by, price) = deposit(self.escrow_program_id, contract_id).await;

        item.contract_id = contract_id;
        item.info.state = ItemState::PurchasedByDistributor;

        msg::reply(
            SupplyChainEvent::PurchasedByDistributor { item_id, by, price },
            0,
        )
        .unwrap();
    }

    fn ship_by_producer(&mut self, item_id: u128) {
        if !self.producers.contains(&msg::source()) {
            panic!("msg::source() must be a producer");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::PurchasedByDistributor {
            panic!("ItemState must be ItemState::PurchasedByDistributor");
        }

        item.info.state = ItemState::ShippedByProducer;

        msg::reply(SupplyChainEvent::ShippedByProducer { item_id }, 0).unwrap();
    }

    async fn receive_by_distributor(&mut self, item_id: u128) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::ShippedByDistributor {
            panic!("ItemState must be ItemState::ShippedByDistributor");
        }

        let (from, price) = confirm(self.escrow_program_id, item.contract_id).await;

        item.info.state = ItemState::ReceivedByDistributor;

        msg::reply(
            SupplyChainEvent::ReceivedByDistributor {
                item_id,
                from,
                price,
            },
            0,
        )
        .unwrap();
    }

    fn process_by_distributor(&mut self, item_id: u128) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::ReceivedByDistributor {
            panic!("ItemState must be ItemState::ReceivedByDistributor");
        }

        item.info.state = ItemState::ProcessedByDistributor;

        msg::reply(SupplyChainEvent::ProcessedByDistributor { item_id }, 0).unwrap();
    }

    fn package_by_distributor(&mut self, item_id: u128) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::ProcessedByDistributor {
            panic!("ItemState must be ItemState::ProcessedByDistributor");
        }

        item.info.state = ItemState::PackagedByDistributor;

        msg::reply(SupplyChainEvent::PackagedByDistributor { item_id }, 0).unwrap();
    }

    fn put_up_for_sale_by_distributor(&mut self, item_id: u128, price: u128) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }

        let item = get_item(&mut self.items, item_id);

        if item.info.state != ItemState::PackagedByDistributor {
            panic!("ItemState must be ItemState::PackagedByDistributor");
        }

        item.price = price;
        item.info.state = ItemState::ForSaleByDistributor;

        msg::reply(SupplyChainEvent::ForSaleByDistributor { item_id, price }, 0).unwrap();
    }

    async fn purchare_by_retailer(&mut self, item_id: u128) {
        if !self.retailers.contains(&msg::source()) {
            panic!("msg::source() must be a retailer");
        }

        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::ForSaleByDistributor {
            panic!("ItemState must be ItemState::ForSaleByDistributor");
        }

        let contract_id = create(self.escrow_program_id, item.info.distributor, item.price).await;
        let (by, price) = deposit(self.escrow_program_id, contract_id).await;

        item.contract_id = contract_id;
        item.info.state = ItemState::PurchasedByRetailer;

        msg::reply(
            SupplyChainEvent::PurchasedByRetailer { item_id, by, price },
            0,
        )
        .unwrap();
    }

    fn ship_by_distributor(&mut self, item_id: u128) {
        if !self.distributors.contains(&msg::source()) {
            panic!("msg::source() must be a distributor");
        }

        let item = get_item(&mut self.items, item_id);

        if item.info.state != ItemState::PurchasedByRetailer {
            panic!("ItemState must be ItemState::PurchasedByRetailer");
        }

        item.info.state = ItemState::ShippedByDistributor;

        msg::reply(SupplyChainEvent::ShippedByDistributor { item_id }, 0).unwrap();
    }

    async fn receive_by_retailer(&mut self, item_id: u128) {
        if !self.retailers.contains(&msg::source()) {
            panic!("msg::source() must be a retailer");
        }

        let item = get_item(&mut self.items, item_id);

        if item.info.state != ItemState::ShippedByDistributor {
            panic!("ItemState must be ItemState::ShippedByDistributor");
        }

        let (from, price) = confirm(self.escrow_program_id, item.contract_id).await;

        item.info.state = ItemState::ReceivedByRetailer;

        msg::reply(
            SupplyChainEvent::ReceivedByRetailer {
                item_id,
                from,
                price,
            },
            0,
        )
        .unwrap();
    }

    fn put_up_for_sale_by_retailer(&mut self, item_id: u128, price: u128) {
        if !self.retailers.contains(&msg::source()) {
            panic!("msg::source() must be a retailer");
        }

        let item = get_item(&mut self.items, item_id);

        if item.info.state != ItemState::ReceivedByRetailer {
            panic!("ItemState must be ItemState::ReceivedByRetailer");
        }

        item.price = price;
        item.info.state = ItemState::ForSaleByRetailer;

        msg::reply(SupplyChainEvent::ForSaleByRetailer { item_id, price }, 0).unwrap();
    }

    async fn purchare_by_consumer(&mut self, item_id: u128) {
        let item = get_item(&mut self.items, item_id);
        if item.info.state != ItemState::ForSaleByRetailer {
            panic!("ItemState must be ItemState::ForSaleByRetailer");
        }

        let contract_id = create(self.escrow_program_id, item.info.distributor, item.price).await;
        let (by, _price) = deposit(self.escrow_program_id, contract_id).await;
        let (from, price) = confirm(self.escrow_program_id, contract_id).await;

        item.contract_id = contract_id;
        item.info.state = ItemState::PurchasedByConsumer;

        msg::reply(
            SupplyChainEvent::PurchasedByConsumer {
                item_id,
                by,
                from,
                price,
            },
            0,
        )
        .unwrap();
    }

    fn get_item_info(&mut self, item_id: u128) {
        let item = get_item(&mut self.items, item_id);

        msg::reply(
            SupplyChainEvent::ItemInfo {
                item_id,
                item: item.info,
            },
            0,
        )
        .unwrap();
    }
}

impl Default for SupplyChain {
    fn default() -> Self {
        Self {
            item_id_nonce: 0,
            items: BTreeMap::new(),

            owner: msg::source(),
            producers: BTreeSet::new(),
            distributors: BTreeSet::new(),
            retailers: BTreeSet::new(),

            escrow_program_id: 0.into(),
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
    } = msg::load().expect("Unable to decode InitSupplyChain");
    let supply_chain = SupplyChain {
        producers,
        distributors,
        retailers,
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
        SupplyChainAction::Produce { name, notes } => supply_chain.produce_item(name, notes),
        SupplyChainAction::PutUpForSaleByProducer { item_id, price } => {
            supply_chain.put_up_for_sale_by_producer(item_id, price)
        }
        SupplyChainAction::PurchaseByDistributor { item_id } => {
            supply_chain.purchare_by_distributor(item_id).await
        }
        SupplyChainAction::ShipByProducer { item_id } => supply_chain.ship_by_producer(item_id),
        SupplyChainAction::ReceiveByDistributor { item_id } => {
            supply_chain.receive_by_distributor(item_id).await
        }
        SupplyChainAction::ProcessByDistributor { item_id } => {
            supply_chain.process_by_distributor(item_id)
        }
        SupplyChainAction::PackageByDistributor { item_id } => {
            supply_chain.package_by_distributor(item_id)
        }
        SupplyChainAction::PutUpForSaleByDistributor { item_id, price } => {
            supply_chain.put_up_for_sale_by_distributor(item_id, price)
        }
        SupplyChainAction::PurchaseByRetailer { item_id } => {
            supply_chain.purchare_by_retailer(item_id).await
        }
        SupplyChainAction::ShipByDistributor { item_id } => {
            supply_chain.ship_by_distributor(item_id)
        }
        SupplyChainAction::ReceiveByRetailer { item_id } => {
            supply_chain.receive_by_retailer(item_id).await
        }
        SupplyChainAction::PutUpForSaleByRetailer { item_id, price } => {
            supply_chain.put_up_for_sale_by_retailer(item_id, price)
        }
        SupplyChainAction::PurchaseByConsumer { item_id } => {
            supply_chain.purchare_by_consumer(item_id).await
        }
        SupplyChainAction::GetItemInfo { item_id } => supply_chain.get_item_info(item_id),
    }
}
