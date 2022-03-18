use crate::{Member, Proposal};
use codec::{Decode, Encode};
use gstd::{ActorId, prelude::*};
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Role {
    Admin,
    Member,
    None,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    UserStatus(ActorId),
    WaitList,
    AllProposals,
    IsMember(ActorId),
    IsInWaitlist(ActorId),
    AmountOfTokens(ActorId),
    ProposalId,
    ProposalInfo(u128),
    MemberInfo(ActorId),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    UserStatus(Role),
    WaitList(BTreeMap<ActorId, u128>),
    AllProposals(BTreeMap<u128, Proposal>),
    IsMember(bool),
    IsInWaitlist(bool),
    AmountOfTokens(u128),
    ProposalId(u128),
    ProposalInfo{
        proposal_id: u128,
        proposal: Proposal
    },
    MemberInfo(Member),
}
