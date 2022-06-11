#![no_std]

pub mod messages;
pub use messages::*;

pub mod asserts;
pub use asserts::*;

use ico_io::*;

use gstd::{prelude::*, exec, msg, ActorId, debug}; 

#[derive(Default)]
struct IcoContract {
    ico_state: IcoState,
    start_price: u128,
    price_increase_step: u128,
    time_increase_step: u128,
    tokens_sold: u128,
    tokens_goal: u128,
    owner: ActorId, 
    token_id: ActorId,
    token_holders: BTreeMap<ActorId, u128>,
}

static mut ICO_CONTRACT: Option<IcoContract> = None;

impl IcoContract {

    /// Starts ICO contract
    /// 
    /// Requirements:
    /// * Only owner can start ICO
    /// * At least `tokens_goal` tokens need to be minted
    /// * ICO can be started only once
    /// * Duration must be greater than zero
    /// 
    /// Arguments:
    /// * `duration`: Time in milliseconds
    /// 
    async fn start_ico(&mut self, duration: u64) {
        assert_ne!(duration, 0, "start_ico(): Can't start ICO with zero duration");
        assert_owner_message(&self.owner, "start_ico(): Not owner start ICO");
        if self.ico_state.ico_started { panic!("start_ico(): Second ICO start"); }
        
        transfer_tokens(&self.token_id, &self.owner, &exec::program_id(), self.tokens_goal).await;

        self.ico_state.ico_started = true;
        self.ico_state.duration = duration;
        self.ico_state.start_time = exec::block_timestamp();

        msg::reply(IcoEvent::SaleStarted(self.ico_state.duration), 0).unwrap();
    }

    /// Purchase of tokens
    /// 
    /// Requirements:
    /// * `tokens_cnt` must be greater than zero
    /// * ICO must be in progress (already started and not finished yet)
    /// * `msg::value` must be greater than or equal to `price * tokens_cnt`
    /// * At least `tokens_cnt` tokens available for sale
    /// 
    /// Arguments:
    /// * `tokens_cnt`: amount of tokens to purchase
    /// 
    pub fn buy_tokens(&mut self, tokens_cnt: u128)  {
        let time_now: u64 = exec::block_timestamp();

        assert!(tokens_cnt != 0, "buy_tokens(): Can't buy zero tokens");
        assert!(self.ico_state.start_time + self.ico_state.duration >= time_now, "buy_tokens(): Duration of the ICO has ended");
        assert!(self.get_balance() != 0, "buy_tokens(): All tokens have been sold");
        self.in_process("buy_tokens()");

        let current_price = self.get_current_price(time_now);
        let cost = tokens_cnt.checked_mul(current_price)
            .unwrap_or_else(|| panic!("buy_tokens(): Overflowing multiplication: {} * {}", tokens_cnt, current_price));

        let mut change = 0;
        let amount_sent = msg::value();

        assert!(tokens_cnt <= self.get_balance(), "buy_tokens(): Not enough tokens to sell");
        assert!(amount_sent >= cost, "buy_tokens(): Wrong amount sent, expect {} get {}", cost, amount_sent);

        if amount_sent > cost {
            change = amount_sent - cost;
            msg::send(msg::source(), "", change).unwrap();
        }

        self.token_holders
            .entry(msg::source())
            .and_modify(|balance| *balance += tokens_cnt)
            .or_insert(tokens_cnt);

        self.tokens_sold += tokens_cnt; 
        
        msg::reply(IcoEvent::Bought { buyer: msg::source(), amount: tokens_cnt, change }, 0).unwrap();
    }

    /// Ends ICO contract
    /// 
    /// Requirements:
    /// * Only owner can end ICO
    /// * ICO can be ended more only once
    /// * All tokens must be sold or the ICO duration must end
    /// 
    async fn end_sale(&mut self) {
        let time_now: u64 = exec::block_timestamp();

        assert_owner_message(&self.owner, "end_sale()");
        self.in_process("end_sale()");

        if self.ico_state.start_time + self.ico_state.duration >= time_now && 
            self.get_balance() != 0 
        {
            panic!("Can't end ICO: tokens left = {}, duration ended = {}", 
                self.get_balance(),
                self.ico_state.start_time + self.ico_state.duration < time_now,
            ) 
        }

        
        for (id, val) in &self.token_holders {
            transfer_tokens(
                &self.token_id,
                &exec::program_id(),
                id,
                *val,
            )
            .await;
        }

        let rest_balance = self.get_balance();
        if rest_balance > 0 { 
            transfer_tokens(
                &self.token_id,
                &exec::program_id(),
                &self.owner,
                rest_balance,
            )
            .await;

            self.token_holders
                .entry(self.owner)
                .and_modify(|balance| *balance += rest_balance)
                .or_insert(rest_balance);
        }

        self.ico_state.ico_ended = true;
        msg::reply(IcoEvent::SaleEnded, 0).unwrap();
    }

    fn get_current_price(&self, time_now: u64) -> u128 {
        let amount: u128 = (time_now - self.ico_state.start_time).into();

        self.start_price + self.price_increase_step * (amount / self.time_increase_step)
    }
    
    fn get_balance(&self) -> u128 {
        self.tokens_goal - self.tokens_sold
    }

    fn in_process(&self, message: &str) { 
        assert!(self.ico_state.ico_started, "{}: ICO wasn't started", message);
        assert!(!self.ico_state.ico_ended, "{}: ICO was ended", message);
    }
}

#[gstd::async_main]
async unsafe fn main() {
    assert_not_zero_address(&msg::source(), "Main message source");

    let action: IcoAction = msg::load().expect("Unable to decode SaleAction");
    let ico: &mut IcoContract = unsafe { ICO_CONTRACT.get_or_insert(IcoContract::default()) };

    match action {
        IcoAction::StartSale(duration) => {
            ico.start_ico(duration).await
        }
        IcoAction::Buy(value) => {
            ico.buy_tokens(value)
        }
        IcoAction::EndSale => {
            ico.end_sale().await
        }
        IcoAction::BalanceOf(address) => {
            assert_owner_message(&ico.owner, "BalanceOf()");

            if let Some(val) = ico.token_holders.get(&address) {
                msg::reply(IcoEvent::BalanceOf { address: address, balance: *val }, 0).unwrap();
            }
            else {
                msg::reply(IcoEvent::BalanceOf { address: address, balance: 0 }, 0).unwrap();
            }
        }
    }
}

fn check_input(config: &IcoInit) {
    assert!(config.tokens_goal != 0, "Init tokens goal is zero");
    assert_not_zero_address(&config.token_id, "Init token address");
    assert_not_zero_address(&config.owner, "Init owner address");
    assert!(config.start_price != 0, "Init start price is zero");
}


#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: IcoInit = msg::load().expect("Unable to decode ICOInit");

    check_input(&config);

    let ico = IcoContract {
        tokens_goal: config.tokens_goal,
        token_id: config.token_id,
        owner: config.owner,
        start_price: config.start_price,
        price_increase_step: config.price_increase_step,
        time_increase_step: config.time_increase_step,
        ..IcoContract::default()
    };

    ICO_CONTRACT = Some(ico);
}

gstd::metadata! {
    title: "crowdsale_ico",
    init:
        input: IcoInit,
    handle:
        input: IcoAction,
        output: IcoEvent,
    state:
        input: State,
        output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let time_now: u64 = exec::block_timestamp();

    let state: State = msg::load().expect("failed to decode State");
    let ico: &mut IcoContract = ICO_CONTRACT.get_or_insert(IcoContract::default());

    let encoded = match state {
        State::CurrentPrice => {
            StateReply::CurrentPrice(ico.get_current_price(time_now)).encode()
        } 
        State::TokensLeft => {
            StateReply::TokensLeft(ico.get_balance()).encode()
        }
        State::Balance(address) => {
            if let Some(val) = ico.token_holders.get(&address) {
                StateReply::Balance(*val).encode()
            }
            else {
                StateReply::Balance(0).encode()
            }
        }
    };

    gstd::util::to_leak_ptr(encoded)
}