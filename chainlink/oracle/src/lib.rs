#![no_std]

use oracle_io::*;
use client_io::*;
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::H256;
pub mod ft_messages;
use ft_messages::transfer_tokens;

#[derive(Debug, Default)]
pub struct Oracle {
    pub owner: ActorId,
    pub link_token: ActorId,
    pub external_adapter: ActorId,
    pub requests: BTreeMap<AccountAndRequestId, OracleRequest>,
}

static mut ORACLE: Option<Oracle> = None;
const EXPIRY_TIME: u64 = 5 * 60 * 1_000_000_000;
pub type AccountAndRequestId = String;
impl Oracle {

    /// * `spec_id`: refers to a specific job for that node to run. Each job is unique and returns different types of data.
    /// * `data`: The CBOR payload of the request
    async fn request(&mut self, payment: u128, spec_id: String, callback_address: ActorId, callback_method: String, request_id: u128, data: String) {
        let account_and_request_id =
            format!("{}{}", H256::from_slice(msg::source().as_ref()),request_id);
        if self.requests.contains_key(&account_and_request_id) {
            panic!("Existing account and nonce in requests");
        }
        transfer_tokens(&self.link_token, &msg::source(), &exec::program_id(), payment).await;
        self.requests.insert(account_and_request_id, OracleRequest {
            caller: msg::source(),
            spec_id: spec_id.clone(),
            callback_address,
            callback_method,
            data: data.clone(),
            payment,
            expiration: exec::block_timestamp() + EXPIRY_TIME,
        });
        msg::reply(
            OracleEvent::Request {
                spec_id,
                caller: msg::source(),
                data
            },
            0
        );
    }

        async fn fullfill_request(&mut self, account: ActorId, request_id: u128, data: String) {
            self.check_external_adapter();
            let account_and_request_id =
                format!("{}{}", H256::from_slice(account.as_ref()), request_id);
            let request =  self.requests.remove(&account_and_request_id).expect("That request doesn't exist");
            let _client_response: ClientEvent = msg::send_and_wait_for_reply(
                request.callback_address,
                ClientAction::OracleAnswer{
                    request_id,
                    message_type: request.callback_method,
                    data,
                },
                0,
            )
            .await
            .expect("Error in sending answer to client");
            msg::reply(
                OracleEvent::RequestFulfilled {
                    account,
                    request_id,
                },
                0
            );
        }

        async fn cancel_request(&mut self, account: &ActorId, request_id: u128) {
            let account_and_request_id =
                format!("{}{}", H256::from_slice(account.as_ref()), request_id);
            let request =  self.requests.remove(&account_and_request_id).expect("That request doesn't exist");
            if request.expiration > exec::block_timestamp() {
                panic!("Request is not expired");
            }
            transfer_tokens(&self.link_token, &exec::program_id(), &msg::source(), request.payment).await;
            msg::reply(
                OracleEvent::RequestCancelled {
                    account: request.caller,
                    request_id,
                },
                0
            );
        }

        fn check_external_adapter(&self) {
            if self.external_adapter != msg::source() {
                panic!("Not an authorized node to fulfill requests");
            }
        }
    }

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitOracle = msg::load().expect("Unable to decode InitConfig");
    let oracle = Oracle {
        owner: config.owner,
        link_token: config.link_token,
        external_adapter: config.external_adapter,
        ..Oracle::default()
    };
   ORACLE = Some(oracle);
}

#[gstd::async_main]
async fn main() {
    let action: OracleAction = msg::load().expect("Could not load Action");
    let oracle: &mut Oracle = unsafe {ORACLE.get_or_insert(Oracle::default())};
    match action {
        OracleAction::Request{payment, spec_id, callback_address, callback_method, request_id, data} => {
            oracle.request(payment, spec_id, callback_address, callback_method, request_id, data).await;
        },
        OracleAction::FullfillRequest {
           account, request_id, data
        } => oracle.fullfill_request(account, request_id, data).await,
        OracleAction::CancelRequest {account, request_id} => oracle.cancel_request(&account, request_id).await
    }
}