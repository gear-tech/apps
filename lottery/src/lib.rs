#![no_std]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use lt_io::*;
use scale_info::TypeInfo;
use sp_core::hashing::blake2_256;

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
struct Lottery {
    ///Lottery state
    lottery_state: LotteryState,
    ///Lottery owner
    lottery_owner: ActorId,
    ///Players by index
    players: BTreeMap<u32, Player>,
    ///Players by timestamp
    players_timestamp: BTreeMap<ActorId, u64>,
    ///Winners list
    lottery_history: BTreeMap<u32, ActorId>,
    ///Lottery Id
    lottery_id: u32,
}

impl Lottery {
    fn lottery_state(&mut self) -> bool {
        self.lottery_state.lottery_started
            && (self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration)
                > exec::block_timestamp()
    }

    fn start_lottery(&mut self, duration: u64) {
        if msg::source() == self.lottery_owner && !self.lottery_state() {
            self.lottery_state.lottery_started = true;
            self.lottery_state.lottery_start_time = exec::block_timestamp();
            self.lottery_state.lottery_duration = duration;
            self.lottery_id += 1;
        }
    }

    fn enter(&mut self) {
        if self.lottery_state() && msg::value() > 0 {
            if self.players_timestamp.get(&msg::source()) == None {
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
                debug!("Player {:?} already added", msg::source());
            }
        }
    }

    fn leave_lottery(&mut self, index: u32) {
        if self.lottery_state() {
            if let Some(player) = self.players.get(&index) {
                if player.player_id == msg::source() {
                    msg::send_bytes(player.player_id, b"LeaveLottery", player.balance);
                    self.players.remove(&index);
                    self.players_timestamp.remove(&msg::source());
                }
            }
        }
    }

    fn add_value(&mut self, index: u32) {
        if self.lottery_state() {
            self.players.entry(index).and_modify(|item| {
                if item.player_id == msg::source() {
                    item.balance = item.balance.saturating_add(msg::value());
                }
            });
        }
    }

    fn get_balance(&mut self, index: u32) {
        if self.lottery_state() {
            if let Some(player) = self.players.get(&index) {
                msg::reply(Event::Balance(player.balance), 0);
            }
        }
    }

    fn get_players(&mut self) {
        if self.lottery_state() && !self.players.is_empty() {
            msg::reply(Event::Players(self.players.clone()), 0);
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
            }

            debug!("Winner: {}", index);

            self.lottery_state = LotteryState::default();
            self.players = BTreeMap::new();
            self.players_timestamp = BTreeMap::new();
        } else {
            debug!(
                "pick_winner() failed! {} {}",
                self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration,
                exec::block_timestamp()
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

        Action::AddValue(index) => {
            lottery.add_value(index);
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
        init:

        handle:
            input: Action,
            output: Event,
        state:
            input: State,
            output: StateReply,
}
