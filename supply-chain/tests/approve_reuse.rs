pub mod utils;
use utils::*;

#[test]
fn approve_reuse() {

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

    check::purchare_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
    );
    check::approve_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0], false);
    check::purchare_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        DELIVERY_TIME[1],
    );
    check::approve_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0], true);

    check::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);
    check::receive_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    check::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    check::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    check::put_up_for_sale_by_distributor(
        &supply_chain_program,
        DISTRIBUTOR[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_DISTRIBUTOR[0],
    );

    check::purchare_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DELIVERY_TIME[0],
    );
    check::approve_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0], false);
    check::purchare_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        DELIVERY_TIME[1],
    );
    check::approve_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0], true);

    check::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
    check::receive_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[0]);
    check::put_up_for_sale_by_retailer(
        &supply_chain_program,
        RETAILER[0],
        ITEM_ID[0],
        ITEM_PRICE_BY_RETAILER[0],
    );
    check::purchare_by_consumer(&supply_chain_program, CONSUMER[0], ITEM_ID[0]);
}
