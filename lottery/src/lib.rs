#![no_std]

#[cfg(test)]
mod simple_tests;

#[cfg(test)]
mod panic_tests;

#[cfg(test)]
mod token_tests;

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use lt_io::*;
use scale_info::TypeInfo;
use sp_core::hashing::blake2_256;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
struct Lottery {
    lottery_state: LotteryState,
    lottery_owner: ActorId,
    token_address: Option<ActorId>,
    players: BTreeMap<u32, Player>,
    lottery_history: BTreeMap<u32, ActorId>,
    lottery_id: u32,
    lottery_balance: u128,
}

impl Lottery {
    // checks that lottery has started and lottery time has not expired
    fn lottery_is_on(&mut self) -> bool {
        self.lottery_state.lottery_started
            && (self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration)
                > exec::block_timestamp()
    }

    /// Launches a lottery
    /// Requirements:
    /// * Only owner can launch lottery
    /// * Lottery must not have been launched earlier
    /// Arguments:
    /// * `duration`: lottery duration in milliseconds
    /// * `token_address`: address of Fungible Token contract
    fn start_lottery(&mut self, duration: u64, token_address: Option<ActorId>) {
        if msg::source() == self.lottery_owner && !self.lottery_is_on() {
            self.lottery_state.lottery_started = true;
            self.lottery_state.lottery_start_time = exec::block_timestamp();
            self.lottery_state.lottery_duration = duration;
            self.token_address = token_address;
            self.lottery_id += 1;
            self.lottery_balance = 0;
        } else {
            panic!(
                "start_lottery(): Lottery on: {}  Owner message: {}",
                self.lottery_is_on(),
                msg::source() == self.lottery_owner
            );
        }
    }

    // checks that the player is already participating in the lottery
    fn player_exist(&mut self) -> bool {
        self.players
            .values()
            .any(|player| player.player_id == msg::source())
    }

    /// Transfers `amount` tokens from `sender` account to `recipient` account.
    /// Arguments:
    /// * `from`: sender account
    /// * `to`: recipient account
    /// * `amount`: amount of tokens
    async fn transfer_tokens(&mut self, from: &ActorId, to: &ActorId, amount_tokens: u128) {
        let _transfer_response: FtEvent = msg::send_and_wait_for_reply(
            self.token_address.unwrap(),
            FtAction::Transfer {
                from: *from,
                to: *to,
                amount: amount_tokens,
            },
            0,
        )
        .await
        .expect("Error in transfer");
    }

    /// Called by a player in order to participate in lottery
    /// Requirements:
    /// * Lottery must be on
    /// * Contribution must be greater than zero
    /// * The player cannot enter the lottery more than once
    /// Arguments:
    /// * `amount`: value or tokens to participate in the lottery
    async fn enter(&mut self, amount: u128) {
        if self.lottery_is_on() && !self.player_exist() && amount > 0 {
            let player = Player {
                player_id: msg::source(),
                balance: amount,
            };

            if self.token_address.is_some() {
                self.transfer_tokens(&msg::source(), &exec::program_id(), amount)
                    .await;

                self.lottery_balance += amount;
                debug!("Add in Fungible Token: {}", amount);
            }

            let player_index = self.players.len() as u32;
            self.players.insert(player_index, player);
            msg::reply(Event::PlayerAdded(player_index), 0);
        } else {
            panic!(
                "enter(): Lottery on: {}  player exist: {} amount: {}",
                self.lottery_is_on(),
                self.player_exist(),
                amount
            );
        }
    }

    /// Removes player from lottery
    /// Requirements:
    /// * Lottery has started and lottery time has not expired
    /// * Player must be on the player list
    /// * Player can only remove himself
    /// Arguments:
    /// * `index`: lottery player index
    async fn leave_lottery(&mut self, index: u32) {
        if self.lottery_is_on() {
            if let Some(player) = self.players.get(&index) {
                if player.player_id == msg::source() {
                    if self.token_address.is_some() {
                        let balance = player.balance;
                        self.transfer_tokens(&exec::program_id(), &msg::source(), balance)
                            .await;
                    } else {
                        msg::send_bytes(player.player_id, b"LeaveLottery", player.balance);
                    }

                    self.players.remove(&index);
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

    /// Gets the player's balance
    /// Requirements:
    /// * Lottery has started and lottery time has not expired
    /// * Player must be on the player list
    /// Arguments:
    /// * `index`: lottery player index
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

    /// Gets a list of players
    /// Requirements:
    /// * Lottery has started and lottery time has not expired
    /// * List of players must not be empty
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

    // Random number generation from block timestamp
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

    /// Lottery winner calculation
    /// Requirements:
    /// * Only owner can pick the winner
    /// * Lottery has started and lottery time is expired
    /// * List of players must not be empty
    async fn pick_winner(&mut self) {
        if msg::source() == self.lottery_owner
            && self.lottery_state.lottery_started
            && self.lottery_state.lottery_start_time + self.lottery_state.lottery_duration
                <= exec::block_timestamp()
            && !self.players.is_empty()
        {
            let index = (self.get_random_number() % (self.players.len() as u32)) as usize;
            let win_player_index = *self.players.keys().nth(index).expect("Player not found");
            let player = self.players[&win_player_index];

            if self.token_address.is_some() {
                debug!("Transfer tokens to the winner");
                self.transfer_tokens(&exec::program_id(), &player.player_id, self.lottery_balance)
                    .await;

                self.lottery_balance = 0;
            } else {
                msg::send_bytes(player.player_id, b"Winner", exec::value_available());
            }

            self.lottery_history
                .insert(self.lottery_id, player.player_id);
            msg::reply(Event::Winner(win_player_index), 0);

            debug!(
                "Winner: {} token_address(): {:?}",
                index, self.token_address
            );

            self.token_address = None;
            self.lottery_state = LotteryState::default();
            self.players = BTreeMap::new();
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

#[gstd::async_main]
async unsafe fn main() {
    if msg::source() == ZERO_ID {
        panic!("Message from zero address");
    }

    let action: Action = msg::load().expect("Could not load Action");
    let lottery: &mut Lottery = unsafe { LOTTERY.get_or_insert(Lottery::default()) };

    match action {
        Action::Enter(amount) => {
            lottery.enter(amount).await;
        }

        Action::StartLottery {
            duration,
            token_address,
        } => {
            lottery.start_lottery(duration, token_address);
        }

        Action::LotteryState => {
            msg::reply(Event::LotteryState(lottery.lottery_state), 0);
            debug!("LotteryState: {:?}", lottery.lottery_state);
        }

        Action::PickWinner => {
            lottery.pick_winner().await;
        }

        Action::BalanceOf(index) => {
            lottery.get_balance(index);
        }

        Action::GetPlayers => {
            lottery.get_players();
        }

        Action::LeaveLottery(index) => {
            lottery.leave_lottery(index).await;
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
