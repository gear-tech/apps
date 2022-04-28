#![no_std]

use codec::{Decode, Encode};
use ft_io::*;
use gstd::{exec, msg, prelude::*, ActorId};
use scale_info::TypeInfo;
use staking_io::*;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
struct Staking {
    staking_token_address: ActorId,
    reward_token_address: Option<ActorId>,
    tokens_per_stake: u128,
    total_staked: u128,
    distribution_time: u64,
    produced_time: u64,
    reward_produced: u128,
    reward_total: u128,
    all_produced: u128,
    stakers: BTreeMap<ActorId, Staker>,
}

static mut STAKING: Option<Staking> = None;

impl Staking {
    /// Transfers `amount` tokens from `sender` account to `recipient` account.
    /// Arguments:
    /// * `from`: sender account
    /// * `to`: recipient account
    /// * `amount`: amount of tokens
    async fn transfer_tokens(
        &mut self,
        token_address: &ActorId,
        from: &ActorId,
        to: &ActorId,
        amount_tokens: u128,
    ) {
        let _transfer_response: FTEvent = msg::send_and_wait_for_reply(
            *token_address,
            FTAction::Transfer {
                from: *from,
                to: *to,
                amount: amount_tokens,
            },
            0,
        )
        .expect("Error in sending message")
        .await
        .expect("Error in transfer");
    }

    /// Calculates the reward produced so far
    fn produced(&mut self) -> u128 {
        self.all_produced
            + self.reward_total * (exec::block_timestamp() - self.produced_time) as u128
                / self.distribution_time as u128
    }

    /// Updates the reward produced so far and calculates tokens per stake
    fn update_reward(&mut self) {
        let reward_produced_at_now: u128 = self.produced();

        if reward_produced_at_now > self.reward_produced {
            let produced_new: u128 = reward_produced_at_now - self.reward_produced;
            if self.total_staked > 0 {
                self.tokens_per_stake += produced_new * (1 << 20) / self.total_staked;
            }
            self.reward_produced += produced_new;
        }
    }

    /// Calculates the reward of the staker that is currently avaiable
    fn calc_reward(&mut self) -> u128 {
        if let Some(staker) = self.stakers.get(&msg::source()) {
            return (staker.balance * self.tokens_per_stake) / (1 << 20) + staker.reward_allowed
                - staker.reward_debt
                - staker.distributed;
        }

        panic!("calc_reward(): Staker {:?} not found", msg::source());
    }

    /// Stakes the tokens
    async fn stake(&mut self, amount: u128) {
        if amount > 0 {
            self.update_reward();

            self.stakers
                .entry(msg::source())
                .and_modify(|stake| {
                    stake.reward_debt += (amount * self.tokens_per_stake) / (1 << 20);
                    stake.balance += amount;
                })
                .or_insert(Staker {
                    reward_debt: (amount * self.tokens_per_stake) / (1 << 20),
                    balance: amount,
                    ..Default::default()
                });

            let token_address: ActorId = self.staking_token_address;

            self.transfer_tokens(&token_address, &msg::source(), &exec::program_id(), amount)
                .await;

            msg::reply(StakingEvent::StakeAccepted(amount), 0).unwrap();
        } else {
            panic!("enter(): amount: {}", amount);
        }
    }

    ///Sends reward to the staker
    async fn send_reward(&mut self) {
        self.update_reward();
        let reward: u128 = self.calc_reward();

        if reward > 0 {
            self.stakers
                .entry(msg::source())
                .and_modify(|stake| stake.distributed += reward);

            let mut token_address: ActorId = self.staking_token_address;

            if self.reward_token_address.is_some() {
                token_address = self.reward_token_address.unwrap();
            }

            self.transfer_tokens(&token_address, &exec::program_id(), &msg::source(), reward)
                .await;

            msg::reply(StakingEvent::Reward(reward), 0).unwrap();
        }
    }

    async fn withdraw(&mut self, amount: u128) {
        if amount == 0 {
            panic!("withdraw(): amount is null");
        }

        if let Some(staker) = self.stakers.get_mut(&msg::source()) {
            if staker.balance < amount {
                panic!("withdraw(): staker.balance < amount");
            }

            staker.reward_allowed += (amount * self.tokens_per_stake) / (1 << 20);
            staker.balance -= amount;

            self.update_reward();

            self.total_staked -= amount;

            let token_address: ActorId = self.staking_token_address;

            self.transfer_tokens(&token_address, &exec::program_id(), &msg::source(), amount)
                .await;

            msg::reply(StakingEvent::Withdrawn(amount), 0).unwrap();
        } else {
            panic!("withdraw(): Staker {:?} not found", msg::source());
        }
    }
}

#[gstd::async_main]
async unsafe fn main() {
    if msg::source() == ZERO_ID {
        panic!("Message from zero address");
    }

    let staking: &mut Staking = unsafe { STAKING.get_or_insert(Staking::default()) };

    let action: StakingAction = msg::load().expect("Could not load Action");

    match action {
        StakingAction::Stake(amount) => {
            staking.stake(amount).await;
        }

        StakingAction::Withdraw(amount) => {
            staking.withdraw(amount).await;
        }

        StakingAction::GetReward => {
            staking.send_reward().await;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitStaking = msg::load().expect("Unable to decode InitConfig");

    let staking = Staking {
        staking_token_address: config.staking_token_address,
        reward_token_address: config.reward_token_address,
        distribution_time: config.distribution_time,
        produced_time: exec::block_timestamp(),
        ..Default::default()
    };

    STAKING = Some(staking);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: StakingState = msg::load().expect("failed to decode input argument");
    let staking: &mut Staking = STAKING.get_or_insert(Staking::default());

    let encoded = match query {
        StakingState::GetStakers => StakingStateReply::Stakers(staking.stakers.clone()).encode(),

        StakingState::GetStaker(address) => {
            if let Some(staker) = staking.stakers.get(&address) {
                StakingStateReply::Staker(*staker).encode()
            } else {
                panic!("meta_state(): Staker {:?} not found", address);
            }
        }
    };

    gstd::util::to_leak_ptr(encoded)
}

gstd::metadata! {
    title: "Staking",
    handle:
        input: StakingAction,
        output: StakingEvent,
    state:
        input: StakingState,
        output: StakingStateReply,
}
