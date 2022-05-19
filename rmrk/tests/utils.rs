use codec::Encode;
use gtest::{Program, RunResult, System};
use rmrk_io::*;
pub const USERS: &'static [u64] = &[5, 6, 7, 8];
pub const ZERO_ID: u64 = 0;
pub const PARENT_NFT_CONTRACT: u64 = 2;
pub const CHILD_NFT_CONTRACT: u64 = 1;
pub const PARENT_TOKEN_ID: u64 = 0;
pub const CHILD_TOKEN_ID: u64 = 0;
pub const FROM_TOKEN_ID: u64 = 0;
pub const TO_TOKEN_ID: u64 = 1;
pub const NEW_PARENT_NFT_CONTRACT: u64 = 3;

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
    rmrk.send(
        user,
        RMRKAction::Approve {
            to: to.into(),
            token_id,
        },
    )
}

pub fn burn_utils(rmrk: &Program, user: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::Burn { token_id })
}

pub fn nft_parent_utils(rmrk: &Program, user: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::NFTParent { token_id })
}

pub fn root_owner_utils(rmrk: &Program, user: u64, token_id: TokenId) -> RunResult {
    rmrk.send(user, RMRKAction::RootOwner { token_id })
}

pub fn transfer_utils(rmrk: &Program, from: u64, to: u64, token_id: TokenId) -> RunResult {
    rmrk.send(
        from,
        RMRKAction::Transfer {
            to: to.into(),
            token_id,
        },
    )
}

pub fn transfer_to_nft_utils(
    rmrk: &Program,
    from: u64,
    to: u64,
    token_id: TokenId,
    destination_id: TokenId,
) -> RunResult {
    rmrk.send(
        from,
        RMRKAction::TransferToNft {
            to: to.into(),
            token_id,
            destination_id,
        },
    )
}

pub fn owner(rmrk: &Program, token_id: TokenId) -> RunResult {
    rmrk.send(10, RMRKAction::Owner { token_id })
}

pub fn get_root_owner(rmrk: &Program, token_id: TokenId) -> RunResult {
    rmrk.send(10, RMRKAction::RootOwner { token_id })
}
