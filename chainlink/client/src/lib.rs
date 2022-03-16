#![no_std]

use client_io::*;
use oracle_io::*;
use gstd::{exec, msg, prelude::*, ActorId};

#[derive(Debug, Default)]
pub struct Client {
    pub oracle: ActorId,
    pub request_id: u128,
    pub requests: BTreeMap<u128, ClientRequest>,
}
static mut CLIENT: Option<Client> = None;

impl Client {
    async fn make_request(&mut self, payment: u128, spec_id: String, data: String, callback_method: String) {
        self.requests.insert(self.request_id, ClientRequest {
            spec_id: spec_id.clone(),
            data_requested: data.clone(),
            data_answer: String::new(),
            fulfilled: false,
        });
        let _oracle_response: OracleEvent = msg::send_and_wait_for_reply(
            self.oracle,
            OracleAction::Request{
                payment,
                spec_id: spec_id.clone(),
                callback_address: exec::program_id(),
                callback_method,
                request_id: self.request_id,
                data: data.clone(),
            },
            0,
        )
        .await
        .expect("Error in making request to oracle");
        self.request_id += 1;
        msg::reply(
            ClientEvent::RequestMade {
                spec_id,
                data,
            },
            0
        );
    }

    fn oracle_answer(&mut self, request_id: u128, message_type: String, data: String) {
        match message_type.as_str() {
            "token_price" => self.token_price(request_id, data),
            _ => panic!("Unknown answer from oracle")
        }
    }
    
    fn token_price(&mut self, request_id: u128, data: String) {
        self.requests.entry(request_id)
            .and_modify(|r| {
                r.data_answer = data.clone();
                r.fulfilled = true;
            });
        msg::reply(
            ClientEvent::RequestFulfilled {
                request_id,
                data_answer: data,
            },
            0
        );
    }
   
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitClient = msg::load().expect("Unable to decode InitConfig");
    let client = Client {
        oracle: config.oracle,
        ..Client::default()
    };
   CLIENT = Some(client);
}

#[gstd::async_main]
async fn main() {
    let action: ClientAction = msg::load().expect("Could not load Action");
    let client: &mut Client = unsafe {CLIENT.get_or_insert(Client::default())};
    match action {
        ClientAction::MakeRequest{payment, spec_id, data, callback_method} => {
            client.make_request(payment, spec_id, data, callback_method).await;
        },
        ClientAction::OracleAnswer {request_id, message_type, data} => client.oracle_answer(request_id, message_type, data),
    }
}