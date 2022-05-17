use codec::Encode;
use gtest::{Program, RunResult, System};
use rmrk_io::*;
pub const USERS: &'static [u64] = &[3, 4, 5, 6];
pub const ZERO_ID: u64 = 0;

pub fn init_rmrk(sys: &System) {
    sys.init_logger();
    let rmrk = Program::current(&sys);
    let res = rmrk.send(
        USERS[0],
        InitRMRK {
            name: "RMRKToken".to_string(),
            symbol: "RMRKSymbol".to_string(),
        },
    );
    assert!(res.log().is_empty());
}

pub fn mint_to_root_owner(rmrk: &Program, user: u64, to: u64, token_id: TokenId) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::MintToRootOwner {
            to: to.into(),
            token_id,
        },
    )
}

pub fn mint_to_nft(
    rmrk: &Program,
    user: u64,
    to: u64,
    token_id: TokenId,
    destination_id: TokenId,
) -> RunResult {
    rmrk.send(
        user,
        RMRKAction::MintToNft {
            to: to.into(),
            token_id: token_id.into(),
            destination_id: destination_id.into(),
        },
    )
}

pub fn approve_utils(rmrk: &Program, user: u64, to: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::Approve{ to: to.into(), token_id })
}

pub fn burn_utils(rmrk: &Program, user: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::Burn{ token_id })
}

pub fn nft_parent_utils(rmrk: &Program, user: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::NFTParent{ token_id })
}

pub fn root_owner_utils(rmrk: &Program, user: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::RootOwner{ token_id })
}