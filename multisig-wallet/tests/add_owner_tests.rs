use gtest::{Program, RunResult, System};
use multisig_wallet_io::*;
const USERS: &'static [u64] = &[3, 4, 5, 6];

// fn common_init(sys: &System, users: &[u64], required: u64,) -> Program {
//     sys.init_logger();
//
//     let wallet = Program::from_file(
//         &sys,
//         "../target/wasm32-unknown-unknown/release/multisig_wallet.wasm",
//     );
//
//     wallet.send(
//         USERS[0],
//         MWInitConfig {
//             owners: users
//                 .iter()
//                 .copied()
//                 .map(|x| x.into())
//                 .collect(),
//             required
//         },
//     );
//
//     wallet
// }

// #[test]
// fn add_owner() {
//     let sys = System::new();
//     let res = common_init(&sys, &USERS[0..3], 3);
//
//     assert!(res.log().is_empty())
// }
