#![no_std]

use gstd::{ActorId, BTreeSet, Decode, Encode, String};
use primitive_types::U256;

#[derive(Encode, Decode)]
pub struct InitSupplyChain {
    pub producers: BTreeSet<ActorId>,
    pub distributors: BTreeSet<ActorId>,
    pub retailers: BTreeSet<ActorId>,
}

#[derive(Encode, Decode)]
pub enum SupplyChainAction {
    Produce { name: String, notes: String },
    PutUpForSaleByProducer { item_id: U256, price: u128 },
    PurchaseByDistributor { item_id: U256, delivery_time: u64 },
    ShipByProducer { item_id: U256 },
    ReceiveByDistributor { item_id: U256 },
    ProcessByDistributor { item_id: U256 },
    PackageByDistributor { item_id: U256 },
    PutUpForSaleByDistributor { item_id: U256, price: u128 },
    PurchaseByRetailer { item_id: U256, delivery_time: u64 },
    ShipByDistributor { item_id: U256 },
    ReceiveByRetailer { item_id: U256 },
    PutUpForSaleByRetailer { item_id: U256, price: u128 },
    PurchaseByConsumer { item_id: U256 },
    GetItemInfo { item_id: U256 },
}

#[derive(Encode, Decode)]
pub enum SupplyChainEvent {
    Produced {
        item_id: U256,
    },
    ForSaleByProducer {
        item_id: U256,
        price: u128,
    },
    PurchasedByDistributor {
        from: ActorId,
        item_id: U256,
        price: u128,
    },
    ShippedByProducer {
        item_id: U256,
        shipping_time: u64,
    },
    ReceivedByDistributor {
        from: ActorId,
        item_id: U256,
    },
    ProcessedByDistributor {
        item_id: U256,
    },
    PackagedByDistributor {
        item_id: U256,
    },
    ForSaleByDistributor {
        item_id: U256,
        price: u128,
    },
    PurchasedByRetailer {
        from: ActorId,
        item_id: U256,
        price: u128,
    },
    ShippedByDistributor {
        item_id: U256,
        shipping_time: u64,
    },
    ReceivedByRetailer {
        item_id: U256,
        from: ActorId,
    },
    ForSaleByRetailer {
        item_id: U256,
        price: u128,
    },
    PurchasedByConsumer {
        from: ActorId,
        item_id: U256,
        price: u128,
    },
    ItemInfo {
        item_id: U256,
        item: ItemInfo,
    },
}

#[derive(Encode, Decode, Clone)]
pub struct ItemInfo {
    pub name: String,
    pub notes: String,
    pub producer: ActorId,
    pub distributor: ActorId,
    pub retailer: ActorId,
    pub state: ItemState,
}

#[derive(Encode, Decode, PartialEq, Clone, Copy, Debug)]
pub enum ItemState {
    Produced,
    ForSaleByProducer,
    PurchasedByDistributor,
    ShippedByProducer,
    ReceivedByDistributor,
    ProcessedByDistributor,
    PackagedByDistributor,
    ForSaleByDistributor,
    PurchasedByRetailer,
    ShippedByDistributor,
    ReceivedByRetailer,
    ForSaleByRetailer,
    PurchasedByConsumer,
}
