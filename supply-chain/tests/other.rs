pub mod utils;
use utils::*;

#[test]
fn ownership_checking() {
    let system = init_system();

    let ft_program = init_ft_program(&system);
    init_nft_program(&system);
    let supply_chain_program = init_supply_chain_program(&system);

    mint(&ft_program, DISTRIBUTOR[0], ITEM_PRICE_BY_PRODUCER[0]);
    mint(&ft_program, RETAILER[0], ITEM_PRICE_BY_DISTRIBUTOR[0]);
    mint(&ft_program, CONSUMER[0], ITEM_PRICE_BY_RETAILER[0]);

    fail::produce(&supply_chain_program, FOREIGN_USER);
    check::produce(
        &supply_chain_program,
        PRODUCER[0],
        ITEM_NAME[0],
        ITEM_NOTES[0],
        ITEM_ID[0],
    );

    fail::put_up_for_sale_by_producer(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::put_up_for_sale_by_producer(&supply_chain_program, PRODUCER[1], ITEM_ID[0]);
    check::put_up_for_sale_by_producer(
        &supply_chain_program,
        PRODUCER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_PRODUCER[0],
    );

    fail::purchare_by_distributor(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    check::purchare_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
        PRODUCER[0],
        ITEM_PRICE_BY_PRODUCER[0],
    );

    fail::ship_by_producer(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::ship_by_producer(&supply_chain_program, PRODUCER[1], ITEM_ID[0]);
    check::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);

    fail::receive_by_distributor(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::receive_by_distributor(&supply_chain_program, DISTRIBUTOR[1], ITEM_ID[0]);
    check::receive_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        PRODUCER[0],
    );

    fail::process_by_distributor(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::process_by_distributor(&supply_chain_program, DISTRIBUTOR[1], ITEM_ID[0]);
    check::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    fail::package_by_distributor(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::package_by_distributor(&supply_chain_program, DISTRIBUTOR[1], ITEM_ID[0]);
    check::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    fail::put_up_for_sale_by_distributor(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::put_up_for_sale_by_distributor(&supply_chain_program, DISTRIBUTOR[1], ITEM_ID[0]);
    check::put_up_for_sale_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );

    fail::purchare_by_retailer(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    check::purchare_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
        DISTRIBUTOR[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );

    fail::ship_by_distributor(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[1], ITEM_ID[0]);
    check::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    fail::receive_by_retailer(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::receive_by_retailer(&supply_chain_program, RETAILER[1], ITEM_ID[0]);
    check::receive_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DISTRIBUTOR[0],
    );

    fail::put_up_for_sale_by_retailer(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
    fail::put_up_for_sale_by_retailer(&supply_chain_program, RETAILER[1], ITEM_ID[0]);
    check::put_up_for_sale_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_RETAILER[0],
    );

    check::purchare_by_consumer(
        &supply_chain_program,
        CONSUMER[0],
        ITEM_ID[0],
        RETAILER[0],
        ITEM_PRICE_BY_RETAILER[0],
    );
}

#[test]
fn state_checking() {
    let system = init_system();

    let ft_program = init_ft_program(&system);
    init_nft_program(&system);
    let supply_chain_program = init_supply_chain_program(&system);

    mint(&ft_program, DISTRIBUTOR[0], ITEM_PRICE_BY_PRODUCER[0]);
    mint(&ft_program, RETAILER[0], ITEM_PRICE_BY_DISTRIBUTOR[0]);
    mint(&ft_program, CONSUMER[0], ITEM_PRICE_BY_RETAILER[0]);

    check::produce(
        &supply_chain_program,
        PRODUCER[0],
        ITEM_NAME[0],
        ITEM_NOTES[0],
        ITEM_ID[0],
    );

    check::put_up_for_sale_by_producer(
        &supply_chain_program,
        PRODUCER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_PRODUCER[0],
    );
    fail::put_up_for_sale_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);

    check::purchare_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
        PRODUCER[0],
        ITEM_PRICE_BY_PRODUCER[0],
    );
    fail::purchare_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    check::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);
    fail::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);

    check::receive_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        PRODUCER[0],
    );
    fail::receive_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    check::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    fail::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    check::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    fail::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    check::put_up_for_sale_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );
    fail::put_up_for_sale_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    check::purchare_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
        DISTRIBUTOR[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );
    fail::purchare_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[0]);

    check::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    fail::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    check::receive_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DISTRIBUTOR[0],
    );
    fail::receive_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[0]);

    check::put_up_for_sale_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_RETAILER[0],
    );
    fail::put_up_for_sale_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[0]);

    check::purchare_by_consumer(
        &supply_chain_program,
        CONSUMER[0],
        ITEM_ID[0],
        RETAILER[0],
        ITEM_PRICE_BY_RETAILER[0],
    );
    fail::purchare_by_consumer(&supply_chain_program, FOREIGN_USER, ITEM_ID[0]);
}

#[test]
fn interact_with_unexistend_item() {
    let system = init_system();

    let ft_program = init_ft_program(&system);
    init_nft_program(&system);
    let supply_chain_program = init_supply_chain_program(&system);

    mint(&ft_program, DISTRIBUTOR[0], ITEM_PRICE_BY_PRODUCER[0]);
    mint(&ft_program, RETAILER[0], ITEM_PRICE_BY_DISTRIBUTOR[0]);
    mint(&ft_program, CONSUMER[0], ITEM_PRICE_BY_RETAILER[0]);

    check::produce(
        &supply_chain_program,
        PRODUCER[0],
        ITEM_NAME[0],
        ITEM_NOTES[0],
        ITEM_ID[0],
    );

    fail::put_up_for_sale_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[1]);
    check::put_up_for_sale_by_producer(
        &supply_chain_program,
        PRODUCER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_PRODUCER[0],
    );

    fail::purchare_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[1]);
    check::purchare_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
        PRODUCER[0],
        ITEM_PRICE_BY_PRODUCER[0],
    );

    fail::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[1]);
    check::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);

    fail::receive_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[1]);
    check::receive_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        PRODUCER[0],
    );

    fail::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[1]);
    check::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    fail::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[1]);
    check::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    fail::put_up_for_sale_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[1]);
    check::put_up_for_sale_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );

    fail::purchare_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[1]);
    check::purchare_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
        DISTRIBUTOR[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );

    fail::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[1]);
    check::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);

    fail::receive_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[1]);
    check::receive_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DISTRIBUTOR[0],
    );

    fail::put_up_for_sale_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[1]);
    check::put_up_for_sale_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_RETAILER[0],
    );

    fail::purchare_by_consumer(&supply_chain_program, FOREIGN_USER, ITEM_ID[1]);
    check::purchare_by_consumer(
        &supply_chain_program,
        CONSUMER[0],
        ITEM_ID[0],
        RETAILER[0],
        ITEM_PRICE_BY_RETAILER[0],
    );

    fail::get_item_info(&supply_chain_program, ITEM_ID[1]);
    check::get_item_info(
        &supply_chain_program,
        ITEM_ID[0],
        ItemInfo {
            distributor: DISTRIBUTOR[0].into(),
            name: ITEM_NAME[0].into(),
            notes: ITEM_NOTES[0].into(),
            producer: PRODUCER[0].into(),
            retailer: RETAILER[0].into(),
            state: ItemState::PurchasedByConsumer,
        },
    );
}
