#![no_std]
use gstd::{ActorId, String};
use scale_info::TypeInfo;
use codec::{Decode, Encode};

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitClient {
    pub oracle: ActorId,
}

#[derive(Debug, Default, Decode, Encode, TypeInfo)]
pub struct ClientRequest {
    pub spec_id: String,
    pub data_requested: String,
    pub data_answer: String,
    pub fulfilled: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum ClientAction {
    MakeRequest{
        payment: u128,
        spec_id: String,
        data: String,
        callback_method: String,
    },
    OracleAnswer {
        message_type: String,
        request_id: u128,
        data: String,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ClientEvent {
    RequestMade{
        spec_id: String,
        data: String,
    },
    RequestFulfilled{
        request_id: u128,
        data_answer: String,
    },
}