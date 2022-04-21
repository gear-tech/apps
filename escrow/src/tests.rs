#[cfg(test)]
extern crate std;
#[cfg(test)]

use gtest::{Program, System};
use std::{println,env};


pub const _FT: u64 = 2;
pub const _FOREIGN_USER: u64 = 2;
pub const _BUYER: [u64; 2] = [1, 2];
pub const _SELLER: [u64; 2] = [3, 4];
pub const _AMOUNT: [u128; 2] = [50, 200];
pub const _CONTRACT: [u128; 2] = [1, 2];


#[test]
fn wrong_buyer_deposit() {
    let system = System::new();
    system.init_logger();

    let current_dir = env::current_dir().expect("Unable to get current dir");
    let path_file = current_dir.join(".binpath");
    println!("current_dir: {}", current_dir.into_os_string().into_string().unwrap());
    println!("path file: {}", path_file.into_os_string().into_string().unwrap());

    let _program = Program::current(&system);
    assert!(_program
        .send(
            _FOREIGN_USER,
            super::InitEscrow {
                ft_token_id: _FT.into(),
            },
        )
        .log()
        .is_empty());

    assert!(_program
    .send(_FOREIGN_USER, super::EscrowActions::Deposit())
    .main_failed());
}

