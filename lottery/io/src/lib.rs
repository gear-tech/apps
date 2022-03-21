#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Default, Encode, Decode, TypeInfo, Clone, Copy)]
pub struct LotteryState {
    pub lottery_started: bool,
    pub lottery_start_time: u64,
    pub lottery_duration: u64,
}

#[derive(Debug, Default, Encode, Decode, TypeInfo, Clone)]
pub struct Player {
    pub player_id: ActorId,
    pub balance: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Enter,
    StartLottery(u64),
    LotteryState,
    PickWinner,
    BalanceOf(u32),
    GetPlayers,
    LeaveLottery(u32),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    LotteryState(LotteryState),
    Winner(u32),
    Balance(u128),
    Players(BTreeMap<u32, Player>),
    PlayerAdded(u32),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum State {
    GetWinners,
    GetPlayers,
    BalanceOf(u32),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    Winners(BTreeMap<u32, ActorId>),
    Players(BTreeMap<u32, Player>),
    Balance(u128),
}
