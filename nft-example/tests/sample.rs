use gtest::{Program, System};
use primitive_types::U256;
use nft_example_io::*;

fn init_with_mint<'a>(sys: &'a System) -> Program<'a> {
    sys.init_logger();

    let nft = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/nft_example.wasm",
    );

    nft.send(InitConfig {
        name: String::from("Hello"),
        symbol: String::from("WRLD"),
        base_uri:  String::from(""),
    });
    sys.assert_log_empty();

    nft.send(Action::Mint);
    sys.assert_log(
        205,
        Event::Transfer{
            from: 0.into(),
            to: 5.into(),
            token_id: U256::zero(),
        },
    );

    nft
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}

// #[test]
// fn burn() {
//     /let/ sys = _System_::new();
//     /let/ erc20 = init_with_mint(&sys);

//     erc20.send(_Action_::Burn(_BurnInput_ {
//         account: 5.into(),
//         amount: 10,
//     }));
//     sys.assert_run_success();
//     sys.assert_log(
//         205,
//         _Event_::Transfer({
//             _TransferReply_ {
//                 from: 5.into(),
//                 to: 0.into(),
//                 amount: 10,
//             }
//         }),
//     );

//     sys.set_user(2);

//     erc20.send(_Action_::Burn(_BurnInput_ {
//         account: 5.into(),
//         amount: 10,
//     }));
//     sys.assert_run_failed();
//     sys.assert_log_bytes(205, []);
// }

// #[test]
// fn transfer() {
//     /let/ sys = _System_::new();
//     /let/ erc20 = init_with_mint(&sys);

//     sys.set_user(5);

//     erc20.send(_Action_::Transfer({
//         _TransferInput_ {
//             to: 6.into(),
//             amount: 30,
//         }
//     }));
//     sys.assert_log(
//         205,
//         _Event_::Transfer({
//             _TransferReply_ {
//                 from: 5.into(),
//                 to: 6.into(),
//                 amount: 30,
//             }
//         }),
//     );
//     sys.assert_run_success();
// }

// #[test]
// fn approve_and_transfer_from() {
//     /let/ sys = _System_::new();
//     /let/ erc20 = init_with_mint(&sys);

//     sys.set_user(5);

//     erc20.send(_Action_::Approve(_ApproveInput_ {
//         spender: 6.into(),
//         amount: 60,
//     }));
//     sys.assert_run_success();
//     sys.assert_log(
//         205,
//         _Event_::Approval(_ApproveReply_ {
//             owner: 5.into(),
//             spender: 6.into(),
//             amount: 60,
//         }),
//     );

//     sys.set_user(6);

//     erc20.send(_Action_::TransferFrom(_TransferFromInput_ {
//         owner: 5.into(),
//         to: 7.into(),
//         amount: 50,
//     }));
//     sys.assert_run_success();
//     sys.assert_log(
//         205,
//         _Event_::TransferFrom(_TransferFromReply_ {
//             owner: 5.into(),
//             sender: 6.into(),
//             recipient: 7.into(),
//             amount: 50,
//             new_limit: 10,
//         }),
//     );
// }