use super::*;

pub fn produce(
    supply_chain_program: &Program,
    producer: u64,
    name: &str,
    notes: &str,

    item_id: u128,
) {
    assert!(supply_chain_program
        .send(
            producer,
            SupplyChainAction::Produce {
                name: name.into(),
                notes: notes.into(),
            },
        )
        .contains(&(
            producer,
            SupplyChainEvent::Produced(item_id.into()).encode(),
        )));
}

pub fn put_up_for_sale_by_producer(
    supply_chain_program: &Program,
    producer: u64,
    item_id: u128,
    price: u128,
) {
    assert!(supply_chain_program
        .send(
            producer,
            SupplyChainAction::PutUpForSaleByProducer {
                item_id: item_id.into(),
                price,
            },
        )
        .contains(&(
            producer,
            SupplyChainEvent::ForSaleByProducer(item_id.into()).encode(),
        )));
}

pub fn purchare_by_distributor(
    supply_chain_program: &Program,
    distributor: u64,
    item_id: u128,
    delivery_time: u64,

    from: u64,
    price: u128,
) {
    assert!(supply_chain_program
        .send(
            distributor,
            SupplyChainAction::PurchaseByDistributor {
                item_id: item_id.into(),
                delivery_time,
            },
        )
        .contains(&(
            distributor,
            SupplyChainEvent::PurchasedByDistributor {
                from: from.into(),
                item_id: item_id.into(),
                price,
            }
            .encode(),
        )));
}

pub fn ship_by_producer(supply_chain_program: &Program, producer: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(producer, SupplyChainAction::ShipByProducer(item_id.into()))
        .contains(&(
            producer,
            SupplyChainEvent::ShippedByProducer(item_id.into()).encode(),
        )));
}

pub fn receive_by_distributor(
    supply_chain_program: &Program,
    distributor: u64,
    item_id: u128,

    from: u64,
) {
    assert!(supply_chain_program
        .send(
            distributor,
            SupplyChainAction::ReceiveByDistributor(item_id.into()),
        )
        .contains(&(
            distributor,
            SupplyChainEvent::ReceivedByDistributor {
                from: from.into(),
                item_id: item_id.into(),
            }
            .encode(),
        )));
}

pub fn process_by_distributor(supply_chain_program: &Program, distributor: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            distributor,
            SupplyChainAction::ProcessByDistributor(item_id.into()),
        )
        .contains(&(
            distributor,
            SupplyChainEvent::ProcessedByDistributor(item_id.into()).encode(),
        )));
}

pub fn package_by_distributor(supply_chain_program: &Program, distributor: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            distributor,
            SupplyChainAction::PackageByDistributor(item_id.into()),
        )
        .contains(&(
            distributor,
            SupplyChainEvent::PackagedByDistributor(item_id.into()).encode(),
        )));
}

pub fn put_up_for_sale_by_distributor(
    supply_chain_program: &Program,
    distributor: u64,
    item_id: u128,

    price: u128,
) {
    assert!(supply_chain_program
        .send(
            distributor,
            SupplyChainAction::PutUpForSaleByDistributor {
                item_id: item_id.into(),
                price,
            }
        )
        .contains(&(
            distributor,
            SupplyChainEvent::ForSaleByDistributor {
                item_id: item_id.into(),
                price,
            }
            .encode()
        )));
}

pub fn purchare_by_retailer(
    supply_chain_program: &Program,
    retailer: u64,
    item_id: u128,
    delivery_time: u64,

    from: u64,
    price: u128,
) {
    assert!(supply_chain_program
        .send(
            retailer,
            SupplyChainAction::PurchaseByRetailer {
                item_id: item_id.into(),
                delivery_time,
            }
        )
        .contains(&(
            retailer,
            SupplyChainEvent::PurchasedByRetailer {
                from: from.into(),
                item_id: item_id.into(),
                price,
            }
            .encode(),
        )));
}

pub fn ship_by_distributor(supply_chain_program: &Program, distributor: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            distributor,
            SupplyChainAction::ShipByDistributor(item_id.into()),
        )
        .contains(&(
            distributor,
            SupplyChainEvent::ShippedByDistributor(item_id.into()).encode(),
        )));
}

pub fn receive_by_retailer(
    supply_chain_program: &Program,
    retailer: u64,
    item_id: u128,

    from: u64,
) {
    assert!(supply_chain_program
        .send(
            retailer,
            SupplyChainAction::ReceiveByRetailer(item_id.into()),
        )
        .contains(&(
            retailer,
            SupplyChainEvent::ReceivedByRetailer {
                from: from.into(),
                item_id: item_id.into(),
            }
            .encode(),
        )));
}

pub fn put_up_for_sale_by_retailer(
    supply_chain_program: &Program,
    retailer: u64,
    item_id: u128,

    price: u128,
) {
    assert!(supply_chain_program
        .send(
            retailer,
            SupplyChainAction::PutUpForSaleByRetailer {
                item_id: item_id.into(),
                price,
            }
        )
        .contains(&(
            retailer,
            SupplyChainEvent::ForSaleByRetailer {
                item_id: item_id.into(),
                price,
            }
            .encode(),
        )));
}

pub fn purchare_by_consumer(
    supply_chain_program: &Program,
    consumer: u64,
    item_id: u128,

    from: u64,
    price: u128,
) {
    assert!(supply_chain_program
        .send(
            consumer,
            SupplyChainAction::PurchaseByConsumer(item_id.into()),
        )
        .contains(&(
            consumer,
            SupplyChainEvent::PurchasedByConsumer {
                from: from.into(),
                item_id: item_id.into(),
                price,
            }
            .encode(),
        )));
}

pub fn item_info(supply_chain_program: &Program, item_id: u128, info: ItemInfo) {
    assert!(supply_chain_program
        .send(FOREIGN_USER, SupplyChainAction::GetItemInfo(item_id.into()))
        .contains(&(
            FOREIGN_USER,
            SupplyChainEvent::ItemInfo {
                item_id: item_id.into(),
                info
            }
            .encode(),
        )));
}
