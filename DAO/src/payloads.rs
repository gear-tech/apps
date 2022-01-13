use codec::{Decode, Encode};
use gstd::String;
use primitive_types::H256;
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub admin: H256,
    pub approved_token_program_id: H256,
    pub period_duration: u64,
    pub voting_period_length: u64,
    pub grace_period_length: u64,
    pub dilution_bound: u128,
    pub abort_window: u64,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct MembershipProposalInput {
    pub applicant: H256,
    pub token_tribute: u128,
    pub shares_requested: u128,
    pub quorum: u128,
    pub details: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct FundingProposalInput {
    pub applicant: H256,
    pub amount: u128,
    pub quorum: u128,
    pub details: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ProcessProposalInput {
    pub proposal_id: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct SubmitVoteInput {
    pub proposal_id: u128,
    pub vote: Vote,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct VoteOnProposal {
    pub account: H256,
    pub proposal_id: u128,
    pub vote: Vote,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ProcessedProposal {
    pub applicant: H256,
    pub proposal_id: u128,
    pub did_pass: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct MembershipProposal {
    pub proposer: H256,
    pub applicant: H256,
    pub proposal_id: u128,
    pub token_tribute: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct FundingProposal {
    pub proposer: H256,
    pub applicant: H256,
    pub proposal_id: u128,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct Withdrawal {
    pub member: H256,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct CancelledProposal {
    pub member: H256,
    pub proposal_id: u128,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, Clone, TypeInfo)]
pub enum Vote {
    Yes,
    No,
}
