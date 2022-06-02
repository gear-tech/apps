#![no_std]
use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
use scale_info::TypeInfo;
use staking_io::*;
pub mod ft_messages;
pub use ft_messages::*;

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
struct Staking {
    staking_token_address: ActorId,
    reward_token_address: ActorId,
    tokens_per_stake: u128,
    total_staked: u128,
    distribution_time: u64,
    produced_time: u64,
    reward_total: u128,
    all_produced: u128,
    reward_produced: u128,
    stakers: BTreeMap<ActorId, Staker>,
}

static mut STAKING: Option<Staking> = None;
const DECIMALS_COUNT: u128 = u128::pow(10, 20);

impl Staking {
    /// Calculates the reward produced so far
    fn produced(&mut self) -> u128 {
        self.all_produced
            + self.reward_total
            + (exec::block_timestamp() - self.produced_time) as u128
                / self.distribution_time as u128
    }

    /// Updates the reward produced so far and calculates tokens per stake
    fn update_reward(&mut self) {
        let reward_produced_at_now = self.produced();

        if reward_produced_at_now > self.reward_produced {
            let produced_new = reward_produced_at_now - self.reward_produced;

            if self.total_staked > 0 {
                self.tokens_per_stake = self
                    .tokens_per_stake
                    .saturating_add((produced_new * DECIMALS_COUNT) / self.total_staked);
            }

            self.reward_produced = self.reward_produced.saturating_add(produced_new);
        }
    }

    /// Calculates the maximum possible reward
    /// The reward that the depositor would have received if he had initially paid this amount
    /// Arguments:
    /// `amount`: the number of tokens
    fn get_max_reward(&self, amount: u128) -> u128 {
        (amount * self.tokens_per_stake) / DECIMALS_COUNT
    }

    /// Calculates the reward of the staker that is currently avaiable
    fn calc_reward(&mut self) -> u128 {
        if let Some(staker) = self.stakers.get(&msg::source()) {
            return self.get_max_reward(staker.balance) + staker.reward_allowed
                - staker.reward_debt
                - staker.distributed;
        }

        panic!("calc_reward(): Staker {:?} not found", msg::source());
    }

    /// Sets the reward to be distributed within distribution time
    /// param 'reward' The value of the distributed reward
    fn set_reward_total(&mut self, reward: u128) {
        if reward == 0 {
            panic!("set_reward_total(): reward is null");
        }

        self.update_reward();
        self.all_produced = self.reward_produced;
        self.produced_time = exec::block_timestamp();
        self.reward_total = reward;
    }

    /// Stakes the tokens
    /// Arguments:
    /// `amount`: the number of tokens for the stake
    async fn stake(&mut self, amount: u128) {
        if amount == 0 {
            panic!("enter(): amount is null");
        }

        let token_address = self.staking_token_address;

        transfer_tokens(&token_address, &msg::source(), &exec::program_id(), amount).await;

        msg::reply(StakingEvent::StakeAccepted(amount), 0).unwrap();

        self.update_reward();
        let amount_per_token = self.get_max_reward(amount);

        self.stakers
            .entry(msg::source())
            .and_modify(|stake| {
                stake.reward_debt = stake.reward_debt.saturating_add(amount_per_token);
                stake.balance = stake.balance.saturating_add(amount);
            })
            .or_insert_with(|| Staker {
                reward_debt: amount_per_token,
                balance: amount,
                ..Default::default()
            });

        self.total_staked = self.total_staked.saturating_add(amount);
    }

    ///Sends reward to the staker
    async fn send_reward(&mut self) {
        self.update_reward();
        let reward = self.calc_reward();

        if reward == 0 {
            panic!("send_reward(): reward is null");
        }

        let token_address = self.reward_token_address;

        transfer_tokens(&token_address, &exec::program_id(), &msg::source(), reward).await;

        msg::reply(StakingEvent::Reward(reward), 0).unwrap();

        self.stakers
            .entry(msg::source())
            .and_modify(|stake| stake.distributed = stake.distributed.saturating_add(reward));
    }

    /// Withdraws the staked the tokens
    /// Arguments:
    /// `amount`: the number of withdrawn tokens
    async fn withdraw(&mut self, amount: u128) {
        if amount == 0 {
            panic!("withdraw(): amount is null");
        }

        let amount_per_token = self.get_max_reward(amount);

        let token_address = self.staking_token_address;

        if let Some(staker) = self.stakers.get_mut(&msg::source()) {
            if staker.balance < amount {
                panic!("withdraw(): staker.balance < amount");
            }

            transfer_tokens(&token_address, &exec::program_id(), &msg::source(), amount).await;

            msg::reply(StakingEvent::Withdrawn(amount), 0).unwrap();

            staker.reward_allowed = staker.reward_allowed.saturating_add(amount_per_token);
            staker.balance = staker.balance.saturating_sub(amount);

            self.update_reward();

            self.total_staked = self.total_staked.saturating_sub(amount);
        } else {
            panic!("withdraw(): Staker {:?} not found", msg::source());
        }
    }
}

#[gstd::async_main]
async unsafe fn main() {
    let staking = unsafe { STAKING.get_or_insert(Staking::default()) };

    let action: StakingAction = msg::load().expect("Could not load Action");

    match action {
        StakingAction::Stake(amount) => {
            staking.stake(amount).await;
        }

        StakingAction::Withdraw(amount) => {
            staking.withdraw(amount).await;
        }

        StakingAction::SetRewardTotal(reward_total) => {
            staking.set_reward_total(reward_total);
            msg::reply(StakingEvent::RewardTotal(reward_total), 0).unwrap();
        }

        StakingAction::GetReward => {
            staking.send_reward().await;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitStaking = msg::load().expect("Unable to decode InitConfig");

    let mut staking = Staking {
        staking_token_address: config.staking_token_address,
        reward_token_address: config.reward_token_address,
        distribution_time: config.distribution_time,
        ..Default::default()
    };

    staking.set_reward_total(config.reward_total);
    STAKING = Some(staking);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: StakingState = msg::load().expect("failed to decode input argument");
    let staking = STAKING.get_or_insert(Staking::default());

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
    init:
        input : InitStaking,
    handle:
        input: StakingAction,
        output: StakingEvent,
    state:
        input: StakingState,
        output: StakingStateReply,
}
