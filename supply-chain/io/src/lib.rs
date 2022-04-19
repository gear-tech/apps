#![no_std]

use gstd::{ActorId, BTreeSet, Decode, Encode, String};

#[derive(Encode, Decode)]
pub struct InitSupplyChain {
    pub producers: BTreeSet<ActorId>,
    pub distributors: BTreeSet<ActorId>,
    pub retailers: BTreeSet<ActorId>,
}

#[derive(Encode, Decode)]
pub enum SupplyChainAction {
    Produce { name: String, notes: String },
    PutUpForSaleByProducer { item_id: u128, price: u128 },
    PurchaseByDistributor { item_id: u128 },
    ShipByProducer { item_id: u128 },
    ReceiveByDistributor { item_id: u128 },
    ProcessByDistributor { item_id: u128 },
    PackageByDistributor { item_id: u128 },
    PutUpForSaleByDistributor { item_id: u128, price: u128 },
    PurchaseByRetailer { item_id: u128 },
    ShipByDistributor { item_id: u128 },
    ReceiveByRetailer { item_id: u128 },
    PutUpForSaleByRetailer { item_id: u128, price: u128 },
    PurchaseByConsumer { item_id: u128 },
    GetItemInfo { item_id: u128 },
}

#[derive(Encode, Decode)]
pub enum SupplyChainEvent {
    Produced {
        item_id: u128,
        name: String,
        notes: String,
    },
    ForSaleByProducer {
        item_id: u128,
        price: u128,
    },
    PurchasedByDistributor {
        item_id: u128,
        by: ActorId,
        price: u128,
    },
    ShippedByProducer {
        item_id: u128,
    },
    ReceivedByDistributor {
        item_id: u128,
        from: ActorId,
        price: u128,
    },
    ProcessedByDistributor {
        item_id: u128,
    },
    PackagedByDistributor {
        item_id: u128,
    },
    ForSaleByDistributor {
        item_id: u128,
        price: u128,
    },
    PurchasedByRetailer {
        item_id: u128,
        by: ActorId,
        price: u128,
    },
    ShippedByDistributor {
        item_id: u128,
    },
    ReceivedByRetailer {
        item_id: u128,
        from: ActorId,
        price: u128,
    },
    ForSaleByRetailer {
        item_id: u128,
        price: u128,
    },
    PurchasedByConsumer {
        item_id: u128,
        by: ActorId,
        from: ActorId,
        price: u128,
    },
    ItemInfo {
        item_id: u128,
        item: ItemInfo,
    },
}

#[derive(Encode, Decode, Clone, Copy)]
pub struct ItemInfo {
    pub producer: ActorId,
    pub distributor: ActorId,
    pub retailer: ActorId,
    pub state: ItemState,
}

#[derive(Encode, Decode, PartialEq, Clone, Copy)]
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
