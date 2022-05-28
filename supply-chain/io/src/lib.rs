#![no_std]

use gstd::{prelude::*, ActorId};
use primitive_types::U256;

#[derive(Encode, Decode, TypeInfo)]
pub struct InitSupplyChain {
    pub producers: BTreeSet<ActorId>,
    pub distributors: BTreeSet<ActorId>,
    pub retailers: BTreeSet<ActorId>,

    pub ft_program_id: ActorId,
    pub nft_program_id: ActorId,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum SupplyChainAction {
    Produce { name: String, notes: String },
    PutUpForSaleByProducer { item_id: U256, price: u128 },
    PurchaseByDistributor { item_id: U256, delivery_time: u64 },
    ShipByProducer(U256),
    ReceiveByDistributor(U256),
    ProcessByDistributor(U256),
    PackageByDistributor(U256),
    PutUpForSaleByDistributor { item_id: U256, price: u128 },
    PurchaseByRetailer { item_id: U256, delivery_time: u64 },
    ShipByDistributor(U256),
    ReceiveByRetailer(U256),
    PutUpForSaleByRetailer { item_id: U256, price: u128 },
    PurchaseByConsumer(U256),
    GetItemInfo(U256),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum SupplyChainEvent {
    Produced(U256),
    ForSaleByProducer(U256),
    PurchasedByDistributor {
        from: ActorId,
        item_id: U256,
        price: u128,
    },
    ShippedByProducer(U256),
    ReceivedByDistributor {
        from: ActorId,
        item_id: U256,
    },
    ProcessedByDistributor(U256),
    PackagedByDistributor(U256),
    ForSaleByDistributor {
        item_id: U256,
        price: u128,
    },
    PurchasedByRetailer {
        from: ActorId,
        item_id: U256,
        price: u128,
    },
    ShippedByDistributor(U256),
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
        info: ItemInfo,
    },
}

#[derive(Encode, Decode, Clone, TypeInfo, Default)]
pub struct ItemInfo {
    pub name: String,
    pub notes: String,
    pub producer: ActorId,
    pub distributor: ActorId,
    pub retailer: ActorId,
    pub state: ItemState,
}

#[derive(Encode, Decode, PartialEq, Clone, Copy, Debug, TypeInfo)]
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

impl Default for ItemState {
    fn default() -> Self {
        Self::Produced
    }
}
