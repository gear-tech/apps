#![no_std]

#[cfg(test)]
mod simple_tests;

#[cfg(test)]
mod panic_tests;

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use lt_io::*;
use scale_info::TypeInfo;
use sp_core::hashing::blake2_256;

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
struct Lottery {
    lottery_state: LotteryState,
    lottery_owner: ActorId,
    players: BTreeMap<u32, Player>,
    players_timestamp: BTreeMap<ActorId, u64>,
    lottery_history: BTreeMap<u32, ActorId>,
    lottery_id: u32,
}

impl Lottery {
    fn lottery_is_on(&mut self) -> bool {
        self.lottery_state.lottery_started
            && (self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration)
                > exec::block_timestamp()
    }

    fn start_lottery(&mut self, duration: u64) {
        if msg::source() == self.lottery_owner && !self.lottery_is_on() {
            self.lottery_state.lottery_started = true;
            self.lottery_state.lottery_start_time = exec::block_timestamp();
            self.lottery_state.lottery_duration = duration;
            self.lottery_id += 1;
        } else {
            panic!(
                "start_lottery(): Lottery on: {}  Owner message: {}",
                self.lottery_is_on(),
                msg::source() == self.lottery_owner
            );
        }
    }

    fn enter(&mut self) {
        if self.lottery_is_on() && msg::value() > 0 {
            if !self.players_timestamp.contains_key(&msg::source()) {
                let player = Player {
                    player_id: msg::source(),
                    balance: msg::value(),
                };

                let player_index = self.players.len() as u32;
                self.players.insert(player_index, player);
                self.players_timestamp
                    .insert(msg::source(), exec::block_timestamp());
                msg::reply(Event::PlayerAdded(player_index), 0);
            } else {
                panic!("enter(): Player {:?} already added", msg::source());
            }
        } else {
            panic!(
                "enter(): Lottery on: {}  Value: {}",
                self.lottery_is_on(),
                msg::value()
            );
        }
    }

    fn leave_lottery(&mut self, index: u32) {
        if self.lottery_is_on() {
            if let Some(player) = self.players.get(&index) {
                if player.player_id == msg::source() {
                    msg::send_bytes(player.player_id, b"LeaveLottery", player.balance);
                    self.players.remove(&index);
                    self.players_timestamp.remove(&msg::source());
                } else {
                    panic!(
                        "leave_lottery(): ActorId's does not match: player: {:?}  msg::source(): {:?}",
                        player.player_id,
                        msg::source()
                    );
                }
            } else {
                panic!("leave_lottery(): Player {} not found", index);
            }
        } else {
            panic!("leave_lottery(): Lottery on: {}", self.lottery_is_on());
        }
    }

    fn get_balance(&mut self, index: u32) {
        if self.lottery_is_on() {
            if let Some(player) = self.players.get(&index) {
                msg::reply(Event::Balance(player.balance), 0);
            } else {
                panic!("get_balance(): Player {} not found", index);
            }
        } else {
            panic!("get_balance(): Lottery on: {}", self.lottery_is_on());
        }
    }

    fn get_players(&mut self) {
        if self.lottery_is_on() && !self.players.is_empty() {
            msg::reply(Event::Players(self.players.clone()), 0);
        } else {
            panic!(
                "get_players(): Lottery on: {}  players.is_empty(): {}",
                self.lottery_is_on(),
                self.players.is_empty()
            );
        }
    }

    fn get_random_number(&mut self) -> u32 {
        let timestamp: u64 = exec::block_timestamp();
        let code_hash: sp_core::H256 = blake2_256(&timestamp.to_be_bytes()).into();
        let u_buf = code_hash.to_fixed_bytes();
        let mut number: u32 = 0;

        for u in u_buf {
            number += u as u32;
        }

        number
    }

    fn pick_winner(&mut self) {
        if msg::source() == self.lottery_owner
            && self.lottery_state.lottery_started
            && self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration
                <= exec::block_timestamp()
            && !self.players.is_empty()
        {
            let index: u32 = self.get_random_number() % (self.players.len() as u32);

            if let Some(win_player) = self.players.get(&index) {
                msg::send_bytes(win_player.player_id, b"Winner", exec::value_available());
                self.lottery_history
                    .insert(self.lottery_id, win_player.player_id);
                msg::reply(Event::Winner(index), 0);
            } else {
                panic!("pick_winner(): Player {} not found", index);
            }

            debug!("Winner: {}", index);

            self.lottery_state = LotteryState::default();
            self.players = BTreeMap::new();
            self.players_timestamp = BTreeMap::new();
        } else {
            panic!(
                "pick_winner(): Owner message: {}  lottery_duration: {}  players.is_empty(): {}",
                msg::source() == self.lottery_owner,
                self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration
                    > exec::block_timestamp(),
                self.players.is_empty()
            );
        }
    }
}

static mut LOTTERY: Option<Lottery> = None;

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    let lottery: &mut Lottery = LOTTERY.get_or_insert(Lottery::default());

    match action {
        Action::Enter => {
            lottery.enter();
        }

        Action::StartLottery(duration) => {
            lottery.start_lottery(duration);
        }

        Action::LotteryState => {
            msg::reply(Event::LotteryState(lottery.lottery_state), 0);
            debug!("LotteryState: {:?}", lottery.lottery_state);
        }

        Action::PickWinner => {
            lottery.pick_winner();
        }

        Action::BalanceOf(index) => {
            lottery.get_balance(index);
        }

        Action::GetPlayers => {
            lottery.get_players();
        }

        Action::LeaveLottery(index) => {
            lottery.leave_lottery(index);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let lottery = Lottery {
        lottery_owner: msg::source(),
        ..Default::default()
    };

    LOTTERY = Some(lottery);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
    let lottery: &mut Lottery = LOTTERY.get_or_insert(Lottery::default());

    let encoded = match query {
        State::GetPlayers => StateReply::Players(lottery.players.clone()).encode(),
        State::GetWinners => StateReply::Winners(lottery.lottery_history.clone()).encode(),

        State::BalanceOf(index) => {
            if let Some(player) = lottery.players.get(&index) {
                StateReply::Balance(player.balance).encode()
            } else {
                StateReply::Balance(0).encode()
            }
        }
    };

    gstd::util::to_leak_ptr(encoded)
}

gstd::metadata! {
    title: "Lottery",
    handle:
        input: Action,
        output: Event,
    state:
        input: State,
        output: StateReply,
}
