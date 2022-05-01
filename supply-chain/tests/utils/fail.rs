use super::*;

pub fn produce(supply_chain_program: &Program, user: u64) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::Produce {
                name: "".into(),
                notes: "".into(),
            },
        )
        .main_failed());
}

pub fn put_up_for_sale_by_producer(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::PutUpForSaleByProducer {
                item_id: item_id.into(),
                price: 0,
            },
        )
        .main_failed());
}

pub fn purchare_by_distributor(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::PurchaseByDistributor {
                item_id: item_id.into(),
                delivery_time: 0,
            },
        )
        .main_failed());
}

pub fn ship_by_producer(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(user, SupplyChainAction::ShipByProducer(item_id.into()))
        .main_failed());
}

pub fn receive_by_distributor(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::ReceiveByDistributor(item_id.into()),
        )
        .main_failed());
}

pub fn process_by_distributor(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::ProcessByDistributor(item_id.into()),
        )
        .main_failed());
}

pub fn package_by_distributor(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::PackageByDistributor(item_id.into()),
        )
        .main_failed());
}

pub fn put_up_for_sale_by_distributor(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::PutUpForSaleByDistributor {
                item_id: item_id.into(),
                price: 0,
            }
        )
        .main_failed());
}

pub fn purchare_by_retailer(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::PurchaseByRetailer {
                item_id: item_id.into(),
                delivery_time: 0,
            }
        )
        .main_failed());
}

pub fn ship_by_distributor(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(user, SupplyChainAction::ShipByDistributor(item_id.into()))
        .main_failed());
}

pub fn receive_by_retailer(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(user, SupplyChainAction::ReceiveByRetailer(item_id.into()))
        .main_failed());
}

pub fn put_up_for_sale_by_retailer(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(
            user,
            SupplyChainAction::PutUpForSaleByRetailer {
                item_id: item_id.into(),
                price: 0,
            }
        )
        .main_failed());
}

pub fn purchare_by_consumer(supply_chain_program: &Program, user: u64, item_id: u128) {
    assert!(supply_chain_program
        .send(user, SupplyChainAction::PurchaseByConsumer(item_id.into()))
        .main_failed());
}

pub fn get_item_info(supply_chain_program: &Program, item_id: u128) {
    assert!(supply_chain_program
        .send(
            FOREIGN_USER,
            SupplyChainAction::GetItemInfo(item_id.into())
        )
        .main_failed());
}
