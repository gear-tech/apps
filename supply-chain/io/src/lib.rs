#![no_std]

use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub type ItemId = U256;

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
    PutUpForSaleByProducer { item_id: ItemId, price: u128 },
    PurchaseByDistributor { item_id: ItemId, delivery_time: u64 },
    ApproveByProducer { item_id: ItemId, approve: bool },
    ShipByProducer(ItemId),
    ReceiveByDistributor(ItemId),
    ProcessByDistributor(ItemId),
    PackageByDistributor(ItemId),
    PutUpForSaleByDistributor { item_id: ItemId, price: u128 },
    PurchaseByRetailer { item_id: ItemId, delivery_time: u64 },
    ApproveByDistributor { item_id: ItemId, approve: bool },
    ShipByDistributor(ItemId),
    ReceiveByRetailer(ItemId),
    PutUpForSaleByRetailer { item_id: ItemId, price: u128 },
    PurchaseByConsumer(ItemId),
    GetItemInfo(ItemId),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum SupplyChainEvent {
    Produced(ItemId),
    Success,
    ItemInfo(ItemInfo),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum SupplyChainState {
    GetItemInfo(ItemId),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum SupplyChainStateReply {
    ItemInfo(ItemInfo),
}

#[derive(Encode, Decode, Clone, TypeInfo, Default)]
pub struct ItemInfo {
    pub name: String,
    pub notes: String,

    pub producer: ActorId,
    pub distributor: ActorId,
    pub retailer: ActorId,

    pub state: ItemState,
    pub price: u128,
    pub delivery_time: u64,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Copy, Debug, TypeInfo, Default)]
pub enum ItemState {
    #[default]
    Produced,
    ForSaleByProducer,
    PurchasedByDistributor,
    ApprovedByProducer,
    ShippedByProducer,
    ReceivedByDistributor,
    ProcessedByDistributor,
    PackagedByDistributor,
    ForSaleByDistributor,
    PurchasedByRetailer,
    ApprovedByDistributor,
    ShippedByDistributor,
    ReceivedByRetailer,
    ForSaleByRetailer,
    PurchasedByConsumer,
}
