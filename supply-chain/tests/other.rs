// mod utils;
// use utils::{
//     check, init_ft_program, init_nft_program, init_supply_chain_program, init_system, mint,
//     ItemInfo, ItemState, CONSUMER, DISTRIBUTOR, ITEM_ID, PRODUCER, RETAILER, check_nft_owner
// };

// #[test]
// fn two_items_through_all_chain() {
//     const ITEM_PRICE_BY_PRODUCER: [u128; 2] = [1234, 4321];
//     const ITEM_PRICE_BY_DISTRIBUTOR: [u128; 2] = [12345, 54321];
//     const ITEM_PRICE_BY_RETAILER: [u128; 2] = [123456, 654321];
//     const CONSUMER_BALANCE: [u128; 2] = [1234567, 7654321];
//     const DELIVERY_TIME: u64 = 1000;

//     let system = init_system();

//     let ft_program = init_ft_program(&system);
//     let nft_program = init_nft_program(&system);
//     let supply_chain_program = init_supply_chain_program(&system);

//     mint(&ft_program, PRODUCER[0], ITEM_PRICE_BY_PRODUCER);
//     mint(&ft_program, PRODUCER[0], ITEM_PRICE_BY_PRODUCER);
//     mint(&ft_program, DISTRIBUTOR[0], ITEM_PRICE_BY_DISTRIBUTOR);
//     mint(&ft_program, DISTRIBUTOR[0], ITEM_PRICE_BY_DISTRIBUTOR);
//     mint(&ft_program, RETAILER[0], ITEM_PRICE_BY_RETAILER);
//     mint(&ft_program, RETAILER[0], ITEM_PRICE_BY_RETAILER);
//     mint(&ft_program, CONSUMER[0], CONSUMER_BALANCE);
//     mint(&ft_program, CONSUMER[0], CONSUMER_BALANCE);

//     check::produce(
//         &supply_chain_program,
//         PRODUCER[0],
//         "Banana",
//         "Tasty",
//         ITEM_ID[0],
//     );
//     check::produce(
//         &supply_chain_program,
//         PRODUCER[1],
//         "Watermelon",
//         "Fresh",
//         ITEM_ID[1],
//     );
//     check_nft_owner(&nft_program, ITEM_ID[0], PRODUCER[0]);
//     check_nft_owner(&nft_program, ITEM_ID[1], PRODUCER[1]);

//     check::put_up_for_sale_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0], ITEM_PRICE_BY_PRODUCER);
//     check::put_up_for_sale_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0], ITEM_PRICE_BY_PRODUCER);



//     check::purchare_by_distributor(
//         &supply_chain_program,
//         DISTRIBUTOR[0],
//         ITEM_ID[0],
//         1000,
//         PRODUCER[0],
//         1000,
//     );
//     check::ship_by_producer(&supply_chain_program, PRODUCER[0], ITEM_ID[0]);
//     check::receive_by_distributor(
//         &supply_chain_program,
//         DISTRIBUTOR[0],
//         ITEM_ID[0],
//         PRODUCER[0],
//     );
//     check::process_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
//     check::package_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
//     check::put_up_for_sale_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0], 10000);
//     check::purchare_by_retailer(
//         &supply_chain_program,
//         RETAILER[0],
//         ITEM_ID[0],
//         1000,
//         DISTRIBUTOR[0],
//         10000,
//     );
//     check::ship_by_distributor(&supply_chain_program, DISTRIBUTOR[0], ITEM_ID[0]);
//     check::receive_by_retailer(
//         &supply_chain_program,
//         RETAILER[0],
//         ITEM_ID[0],
//         DISTRIBUTOR[0],
//     );
//     check::put_up_for_sale_by_retailer(&supply_chain_program, RETAILER[0], ITEM_ID[0], 100000);
//     check::purchare_by_consumer(
//         &supply_chain_program,
//         CONSUMER[0],
//         ITEM_ID[0],
//         RETAILER[0],
//         100000,
//     );
//     check::item_info(
//         &supply_chain_program,
//         ITEM_ID[0],
//         ItemInfo {
//             name: "Banana".into(),
//             notes: "Tasty".into(),
//             state: ItemState::PurchasedByConsumer,
//             distributor: DISTRIBUTOR[0].into(),
//             producer: PRODUCER[0].into(),
//             retailer: RETAILER[0].into(),
//         },
//     );
// }
