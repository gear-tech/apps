#![no_std]

pub mod messages;
pub use messages::*;

pub mod constants;
use constants::ZERO_ID;

use ico_io::*;

use core::{panic};

use gstd::{prelude::*, exec, msg, ActorId, debug}; 

#[derive(Default)]
struct IcoContract {
    ico_state: IcoState,
    start_price: u128,
    current_price: u128, 
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
    async fn get_tokens(&self) {
        let balance = balance(&self.token_id, &self.owner).await;

        if balance < self.tokens_goal {
            panic!("Need to mint at least {} (= tokens goal) tokens", self.tokens_goal)
        }

        transfer_tokens(&self.token_id, &self.owner, &exec::program_id(), self.tokens_goal).await;
        approve(&self.token_id, &exec::program_id(), self.tokens_goal).await;
    }

    async fn start_ico(&mut self, duration: u64) {
        if msg::source() == self.owner && !self.ico_state.ico_started {
            self.get_tokens().await;

            self.ico_state.ico_started = true;
            self.ico_state.duration = duration;
            self.ico_state.start_time = exec::block_timestamp();

            msg::reply(IcoEvent::SaleStarted(self.ico_state.duration), 0).unwrap();
        }
        else {
            panic!(
                "start_contract(): ICO contract's already been started: {}  Owner message: {}",
                self.ico_state.ico_started,
                msg::source() == self.owner
            );
        }
    }

    pub fn buy_tokens(&mut self, tokens_cnt: u128)  {
        let time_now: u64 = exec::block_timestamp();

        self.in_process(time_now, true);

        self.update_price(time_now);

        let (cost, overflow) = tokens_cnt.overflowing_mul(self.current_price);
        if overflow {
            panic!("Overflowing multiplication: {} * {}", tokens_cnt, self.current_price)
        }

        if msg::value() != cost {
            panic!("Wrong amount sent, expect {} get {}", cost, msg::value())
        }

        let (tokens_sum, overflow) = self.tokens_sold.overflowing_add(tokens_cnt);
        if overflow {
            panic!("Overflowing addition: {} + {}", self.tokens_sold, tokens_cnt)
        }

        if tokens_sum > self.tokens_goal {
            panic!("Not enough tokens to sell")
        }

        self.token_holders
            .entry(msg::source())
            .and_modify(|balance| *balance += tokens_cnt)
            .or_insert(tokens_cnt);

        self.tokens_sold += tokens_cnt;
        
        msg::reply(IcoEvent::Bought { buyer: msg::source(), amount: tokens_cnt }, 0).unwrap();
    }

    async fn end_sale(&mut self) {
        let time_now: u64 = exec::block_timestamp();

        if msg::source() != self.owner {
            panic!("end_sale(): Not owner message");
        }

        if self.ico_state.ico_ended {
            panic!("You can end sale only once")
        }

        if !self.ico_state.ico_started {
            panic!("end_sale(): Ico wan't started")
        }

        if !self.in_process(time_now, false){

            for (id, val) in &self.token_holders {
                transfer_tokens(
                    &self.token_id,
                    &exec::program_id(),
                    id,
                    *val,
                )
                .await;
            }

            self.ico_state.ico_ended = true;

            msg::reply(IcoEvent::SaleEnded, 0).unwrap();
        }
        else {
            panic!("Can't end sale, ico is in process")
        }
    }

    fn update_price(&mut self, time_now: u64) {
        let step = self.time_increase_step;
        let amount: u128 = (time_now - self.ico_state.start_time).into();

        if step > amount {
            return
        }

        self.current_price = self.start_price + self.price_increase_step * (amount / step);
    }
    
    fn get_balance(&self) -> u128 {
        self.tokens_goal - self.tokens_sold
    }

    fn in_process(&self, time_now: u64, panic: bool) -> bool { 
        if !panic {
            return self.ico_state.ico_started &&
                    self.ico_state.start_time + self.ico_state.duration >= time_now &&
                    self.get_balance() > 0 &&
                    !self.ico_state.ico_ended
        }

        if !self.ico_state.ico_started {
            panic!("Ico wasn't started")
        }

        if self.ico_state.start_time + self.ico_state.duration < time_now {
            panic!("Duration of the contract has ended")
        }

        if self.get_balance() == 0 {
            panic!("All tokens have been sold")
        }

        if self.ico_state.ico_ended {
            panic!("Ico was ended")
        }

        return true
    }
}


#[gstd::async_main]
async unsafe fn main() {
    if msg::source() == ZERO_ID {
        panic!("Message from zero address");
    }

    let action: IcoAction = msg::load().expect("Unable to decode SaleAction");
    let ico: &mut IcoContract = unsafe { ICO_CONTRACT.get_or_insert(IcoContract::default()) };

    match action {
        IcoAction::StartSale(duration) => {
            if duration == 0 {
                panic!("Can't start ico with duration = {}", duration)
            }
            else {
                ico.start_ico(duration).await
            }
        }
        IcoAction::Buy(value) => {
            if value == 0 {
                panic!("Can't buy {} tokens", value)
            }
            else {
                ico.buy_tokens(value)
            }
        }
        IcoAction::EndSale => {
            ico.end_sale().await
        }
        IcoAction::BalanceOf(address) => {
            if msg::source() != ico.owner {
                panic!("BalanceOf(): Not owner message");
            }

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
    if config.tokens_goal == 0 {
        panic!("Tokens goal is zero: {:?}", config.tokens_goal)
    }

    if config.token_id == ZERO_ID {
        panic!("Token address is zero: {:?}", config.token_id)
    }

    if config.owner == ZERO_ID {
        panic!("Owner address is zero: {:?}", config.owner)
    }

    if config.start_price == 0 {
        panic!("Start price is zero: {}", config.start_price)
    }
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
        current_price: config.start_price,
        price_increase_step: config.price_increase_step,
        time_increase_step: config.time_increase_step,
        ..IcoContract::default()
    };

    ICO_CONTRACT = Some(ico);
}

gstd::metadata! {
    title: "ICO_contract",
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
            ico.update_price(time_now);
            StateReply::CurrentPrice(ico.current_price).encode()
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

    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));
    core::mem::forget(encoded);
    result
}