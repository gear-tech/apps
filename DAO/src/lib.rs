#![no_std]
#![feature(const_btree_new)]
use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId, String};
use primitive_types::H256;
use scale_info::TypeInfo;

pub mod payloads;
pub use payloads::{
    CancelledProposal, FundingProposal, FundingProposalInput, InitConfig, MembershipProposal,
    MembershipProposalInput, ProcessProposalInput, ProcessedProposal, SubmitVoteInput, Vote,
    VoteOnProposal, Withdrawal,
};
pub mod erc20_functions;
pub use erc20_functions::{balance, transfer_tokens};
const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    AddToWhiteList(H256),
    SubmitMembershipProposal(MembershipProposalInput),
    SubmitFundingProposal(FundingProposalInput),
    ProcessProposal(ProcessProposalInput),
    SubmitVote(SubmitVoteInput),
    RageQuit(u128),
    Abort(u128),
    CancelProposal(u128),
    UpdateDelegateKey(H256),
    SetAdmin(H256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    MemberAddedToWhitelist(H256),
    SubmitMembershipProposal(MembershipProposal),
    SubmitFundingProposal(FundingProposal),
    SubmitVote(VoteOnProposal),
    ProcessProposal(ProcessedProposal),
    RageQuit(Withdrawal),
    Abort(CancelledProposal),
    Cancel(CancelledProposal),
    AdminUpdated(H256),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    IsMember(H256),
    IsInWhitelist(H256),
    ProposalId,
    ProposalInfo(u128),
    MemberInfo(H256),
}

#[derive(Debug, Encode, TypeInfo)]
pub enum StateReply {
    IsMember(bool),
    IsInWhitelist(bool),
    ProposalId(u128),
    ProposalInfo(Proposal),
    MemberInfo(Member),
}

#[derive(Debug)]
struct Dao {
    admin: ActorId,
    approved_token_program_id: ActorId,
    period_duration: u64,
    voting_period_length: u64,
    grace_period_length: u64,
    dilution_bound: u128,
    abort_window: u64,
    total_shares: u128,
    members: BTreeMap<ActorId, Member>,
    member_by_delegate_key: BTreeMap<ActorId, ActorId>,
    proposal_id: u128,
    proposals: BTreeMap<u128, Proposal>,
    whitelist: Vec<ActorId>,
}

#[derive(Debug, Clone, Decode, Encode, TypeInfo)]
pub struct Proposal {
    pub proposer: ActorId,
    pub applicant: ActorId,
    pub shares_requested: u128,
    pub yes_votes: u128,
    pub no_votes: u128,
    pub quorum: u128,
    pub is_membership_proposal: bool,
    pub amount: u128,
    pub processed: bool,
    pub did_pass: bool,
    pub cancelled: bool,
    pub aborted: bool,
    pub token_tribute: u128,
    pub details: String,
    pub starting_period: u64,
    pub max_total_shares_at_yes_vote: u128,
    pub votes_by_member: BTreeMap<H256, Vote>,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct Member {
    pub delegate_key: ActorId,
    pub shares: u128,
    pub highest_index_yes_vote: u128,
}

static mut DAO: Dao = Dao {
    admin: ActorId::new(H256::zero().to_fixed_bytes()),
    approved_token_program_id: ActorId::new(H256::zero().to_fixed_bytes()),
    voting_period_length: 0,
    period_duration: 0,
    grace_period_length: 0,
    abort_window: 0,
    dilution_bound: 0,
    total_shares: 0,
    members: BTreeMap::new(),
    member_by_delegate_key: BTreeMap::new(),
    proposal_id: 0,
    proposals: BTreeMap::new(),
    whitelist: Vec::new(),
};

impl Dao {
    // Adds members to whitelist
    // Requirements:
    // * Only admin can add actors to whitelist
    // * Member ID cant be zero
    // * Member can not be added to whitelist more than once
    // Arguments:
    // * `member`: valid actor ID
    fn add_to_whitelist(&mut self, member: &ActorId) {
        if self.admin != msg::source() {
            panic!("msg::source() must be DAO admin");
        }
        if member == &ZERO_ID {
            panic!("Member ID can not be zero");
        }
        if self.whitelist.contains(member) {
            panic!("Member has already been added to the whitelist");
        }
        self.whitelist.push(*member);
        msg::reply(
            Event::MemberAddedToWhitelist(H256::from_slice(member.as_ref())),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    // The proposal of joining the DAO.
    // Requirements:
    // * The proposal can be submitted only by the existing members or their delegate addresses
    // * The applicant ID must be either a DAO member or is on  whitelist
    // Arguments:
    // * `applicant`: an actor, who wishes to become a DAO member
    // * `token_tribute`: the number of tokens the applicant offered for shares in DAO
    // * `shares_requested`: the amount of shares the applicant is requesting for his token tribute
    // * `quorum`: a certain threshold of YES votes in order for the proposal to pass
    // * `details`: the proposal description
    async fn submit_membership_proposal(
        &mut self,
        applicant: &ActorId,
        token_tribute: u128,
        shares_requested: u128,
        quorum: u128,
        details: String,
    ) {
        // check that `msg::source()` is either a DAO member or a delegate key
        match self.member_by_delegate_key.get(&msg::source()) {
            Some(member) => {
                if !self.is_member(member) {
                    panic!("account is not a DAO member");
                }
            }
            None => {
                panic!("account is not a delegate");
            }
        }
        // check that applicant is either in whitelist or a DAO member
        if !self.whitelist.contains(applicant) && !self.members.contains_key(applicant) {
            panic!("Applicant must be either in whitelist or be a DAO member");
        }

        // transfer applicant tokens to DAO contract
        transfer_tokens(
            &self.approved_token_program_id,
            applicant,
            &exec::program_id(),
            token_tribute,
        )
        .await;

        let mut starting_period = exec::block_timestamp();
        // compute startingPeriod for proposal
        // there should be a minimum time interval between proposals (period_duration) so that members have time to ragequit
        if self.proposal_id > 0 {
            let previous_starting_period = self
                .proposals
                .get(&(&self.proposal_id - 1))
                .unwrap()
                .starting_period;
            if starting_period < previous_starting_period + self.period_duration {
                starting_period = previous_starting_period + self.period_duration;
            }
        }

        let proposal = Proposal {
            proposer: msg::source(),
            applicant: *applicant,
            shares_requested,
            yes_votes: 0,
            no_votes: 0,
            quorum,
            is_membership_proposal: true,
            amount: 0,
            processed: false,
            did_pass: false,
            cancelled: false,
            aborted: false,
            token_tribute,
            details,
            starting_period,
            max_total_shares_at_yes_vote: 0,
            votes_by_member: BTreeMap::new(),
        };
        self.proposals.insert(self.proposal_id, proposal);
        let new_proposal = MembershipProposal {
            proposer: H256::from_slice(msg::source().as_ref()),
            applicant: H256::from_slice(applicant.as_ref()),
            proposal_id: self.proposal_id,
            token_tribute,
        };
        msg::reply(
            Event::SubmitMembershipProposal(new_proposal),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
        self.proposal_id = self.proposal_id.saturating_add(1);
    }

    // The proposal of funding
    // Requirements:
    // * The proposal can be submitted only by the existing members or their delegate addresses
    // * The receiver ID can't be the zero
    // * The DAO must have enough funds to finance the proposal
    // Arguments:
    // * `receiver`: an actor that will be funded
    // * `amount`: the number of ERC20 tokens that will be sent to the receiver
    // * `quorum`: a certain threshold of YES votes in order for the proposal to pass
    // * `details`: the proposal description
    async fn submit_funding_proposal(
        &mut self,
        applicant: &ActorId,
        amount: u128,
        quorum: u128,
        details: String,
    ) {
        // check that `msg::source()` is either a DAO member or a delegate key
        match self.member_by_delegate_key.get(&msg::source()) {
            Some(member) => {
                if !self.is_member(member) {
                    panic!("account is not a DAO member");
                }
            }
            None => {
                panic!("account is not a delegate");
            }
        }

        if applicant == &ZERO_ID {
            panic!("Proposal for the zero address");
        }

        // check that DAO has sufficient funds
        let balance = balance(&self.approved_token_program_id, &exec::program_id()).await;
        if balance < amount {
            panic!("Not enough funds in DAO");
        }

        let mut starting_period = exec::block_timestamp();
        // compute startingPeriod for proposal
        // there should be a minimum time interval between proposals (period_duration) so that members have time to ragequit
        if self.proposal_id > 0 {
            let previous_starting_period = self
                .proposals
                .get(&(&self.proposal_id - 1))
                .unwrap()
                .starting_period;
            if starting_period < previous_starting_period + self.period_duration {
                starting_period = previous_starting_period + self.period_duration;
            }
        }

        let proposal = Proposal {
            proposer: msg::source(),
            applicant: *applicant,
            shares_requested: 0,
            yes_votes: 0,
            no_votes: 0,
            quorum,
            is_membership_proposal: false,
            amount,
            processed: false,
            did_pass: false,
            cancelled: false,
            aborted: false,
            token_tribute: 0,
            details,
            starting_period,
            max_total_shares_at_yes_vote: 0,
            votes_by_member: BTreeMap::new(),
        };
        self.proposals.insert(self.proposal_id, proposal);
        let new_proposal = FundingProposal {
            proposer: H256::from_slice(msg::source().as_ref()),
            applicant: H256::from_slice(applicant.as_ref()),
            proposal_id: self.proposal_id,
            amount,
        };
        msg::reply(
            Event::SubmitFundingProposal(new_proposal),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
        self.proposal_id = self.proposal_id.saturating_add(1);
    }

    // The member (or the delegate address of the member) submit his vote (YES or NO) on the proposal
    // Requirements:
    // * The proposal can be submitted only by the existing members or their delegate addresses
    // * The member can vote on the proposal only once
    // * Proposal must exist, the voting period must has started and not expired
    // Arguments:
    // * `proposal_id`: the proposal ID
    // * `vote`: the member  a member vote (YES or NO)
    fn submit_vote(&mut self, proposal_id: u128, vote: Vote) {
        // check that `msg::source()` is either a DAO member or a delegate key
        match self.member_by_delegate_key.get(&msg::source()) {
            Some(member) => {
                if !self.is_member(member) {
                    panic!("account is not a DAO member");
                }
            }
            None => {
                panic!("account is not a delegate");
            }
        }

        let account = H256::from_slice(msg::source().as_ref());
        // checks that proposal exists, the voting period has started, not expired and that member did not vote on the proposal
        match self.proposals.get(&proposal_id) {
            Some(proposal) => {
                if exec::block_timestamp() > proposal.starting_period + self.voting_period_length {
                    panic!("proposal voting period has expired");
                }
                if exec::block_timestamp() < proposal.starting_period {
                    panic!("voting period has not started");
                }
                if proposal.votes_by_member.contains_key(&account) {
                    panic!("account has already voted on that proposal");
                }
            }
            None => {
                panic!("proposal does not exist");
            }
        }

        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        let member = self.members.get_mut(&msg::source()).unwrap();

        match vote {
            Vote::Yes => {
                proposal.yes_votes = proposal.yes_votes.saturating_add(member.shares);
                if self.total_shares > proposal.max_total_shares_at_yes_vote {
                    proposal.max_total_shares_at_yes_vote = self.total_shares;
                }
                // it is necessary to save the highest id of the proposal - must be processed for member to ragequit
                if member.highest_index_yes_vote < proposal_id {
                    member.highest_index_yes_vote = proposal_id;
                }
            }
            Vote::No => {
                proposal.no_votes = proposal.no_votes.saturating_sub(member.shares);
            }
        }
        proposal
            .votes_by_member
            .insert(H256::from_slice(msg::source().as_ref()), vote.clone());
        let vote_on_proposal = VoteOnProposal {
            account,
            proposal_id,
            vote,
        };
        msg::reply(
            Event::SubmitVote(vote_on_proposal),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    // The proposal processing after the proposal completes during the grace period.
    // If the proposal is accepted, the tribute tokens are deposited into the contract and new shares are minted and issued to the applicant.
    // If the proposal is rejected, the tribute tokens are returned to the applicant.
    // Requirements:
    // * The previous proposal must be processed
    // * The proposal must exist, be ready for processing
    // * The proposal must not be cancelled, aborted or already be processed
    // Arguments:
    // * `proposal_id`: the proposal ID
    async fn process_proposal(&mut self, proposal_id: u128) {
        match self.proposals.get(&proposal_id) {
            Some(proposal) => {
                if proposal_id > 0 && !self.proposals.get(&(&proposal_id - 1)).unwrap().processed {
                    panic!("Previous proposal must be processed");
                }
                if proposal.processed || proposal.cancelled || proposal.aborted {
                    panic!("Proposal has already been processed, cancelled or aborted");
                }
                if exec::block_timestamp()
                    < proposal.starting_period
                        + self.voting_period_length
                        + self.grace_period_length
                {
                    panic!("Proposal is not ready to be processed");
                }
            }
            None => {
                panic!("proposal does not exist");
            }
        }

        let mut proposal = self.proposals.get(&proposal_id).unwrap().clone();
        proposal.processed = true;
        proposal.did_pass = proposal.yes_votes > proposal.no_votes
            && proposal.yes_votes * 1000 / self.total_shares > proposal.quorum
            && proposal.max_total_shares_at_yes_vote < self.dilution_bound * self.total_shares;

        // if membership proposal has passed
        if proposal.did_pass && proposal.is_membership_proposal {
            self.members.entry(proposal.applicant).or_insert(Member {
                delegate_key: proposal.applicant,
                shares: 0,
                highest_index_yes_vote: 0,
            });
            let applicant = self.members.get_mut(&proposal.applicant).unwrap();
            applicant.shares = applicant.shares.saturating_add(proposal.shares_requested);
            self.member_by_delegate_key
                .entry(proposal.applicant)
                .or_insert(proposal.applicant);
            self.total_shares = self.total_shares.saturating_add(proposal.shares_requested);
        } else {
            transfer_tokens(
                &self.approved_token_program_id,
                &exec::program_id(),
                &proposal.applicant,
                proposal.token_tribute,
            )
            .await;
        }

        // if funding propoposal has passed
        if proposal.did_pass && !proposal.is_membership_proposal {
            transfer_tokens(
                &self.approved_token_program_id,
                &exec::program_id(),
                &proposal.applicant,
                proposal.amount,
            )
            .await;
        }

        let processed_proposal = ProcessedProposal {
            applicant: H256::from_slice(proposal.applicant.as_ref()),
            proposal_id,
            did_pass: proposal.did_pass,
        };
        msg::reply(
            Event::ProcessProposal(processed_proposal),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
        self.proposals.insert(proposal_id, proposal);
    }

    // Withdraws the capital of the member
    // Requirements:
    // * `msg::source()` must be DAO member
    // * The member must have sufficient amount
    // * The latest proposal the member voted YES must be processed
    // * Admin can ragequit only after transferring his role to another actor
    // Arguments:
    // * `amount`: The amount of ERC20 tokens the member would like to withdraw
    async fn ragequit(&mut self, amount: u128) {
        if self.admin == msg::source() {
            panic!("admin can not ragequit");
        }
        if !self.members.contains_key(&msg::source()) {
            panic!("account is not a DAO member");
        }
        let member = self.members.get_mut(&msg::source()).unwrap();
        if amount > member.shares {
            panic!("unsufficient shares");
        }

        let proposal_id = member.highest_index_yes_vote;
        if !self.proposals.get(&proposal_id).unwrap().processed {
            panic!("cant ragequit until highest index proposal member voted YES on is processed");
        }
        member.shares = member.shares.saturating_sub(amount);
        self.total_shares = self.total_shares.saturating_sub(amount);
        let funds = self.redeemable_funds(amount).await;
        transfer_tokens(
            &self.approved_token_program_id,
            &exec::program_id(),
            &msg::source(),
            funds,
        )
        .await;
        let withdrawal_data = Withdrawal {
            member: H256::from_slice(msg::source().as_ref()),
            amount: funds,
        };
        msg::reply(
            Event::RageQuit(withdrawal_data),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    // Cancels the proposal after the end of the voting period if there are no YES votes.
    // Requirements:
    // * `msg::source()` must be the proposer
    // * It can be cancelled if the number of YES votes is less than number of NO votes or the required quorum is not achieved
    // * The voting period must be over
    // * The proposal must not be cancelled or aborted
    // Arguments:
    // * `proposal_id`: the proposal ID
    async fn cancel_proposal(&mut self, proposal_id: u128) {
        if !self.proposals.contains_key(&proposal_id) {
            panic!("proposal does not exist");
        }
        if self.proposals.get(&proposal_id).unwrap().proposer != msg::source() {
            panic!("caller must be proposer");
        }
        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        if proposal.yes_votes > proposal.no_votes
            && proposal.yes_votes * 1000 / self.total_shares > proposal.quorum
        {
            panic!(
                "Proposal can not be cancelled since YES votes > NO votes and quorum is achieved"
            );
        }
        if exec::block_timestamp() < proposal.starting_period + self.voting_period_length {
            panic!("The voting period is not over yet");
        }
        if proposal.cancelled || proposal.aborted {
            panic!("Proposal has already been cancelled or aborted");
        }
        let amount = proposal.token_tribute;
        proposal.token_tribute = 0;
        proposal.cancelled = true;

        transfer_tokens(
            &self.approved_token_program_id,
            &exec::program_id(),
            &msg::source(),
            amount,
        )
        .await;

        let cancelled_proposal = CancelledProposal {
            member: H256::from_slice(msg::source().as_ref()),
            proposal_id,
            amount,
        };

        msg::reply(
            Event::Cancel(cancelled_proposal),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    // Aborts the membership proposal. It can be used in case when applicant is disagree with the requested shares or the details the proposer  indicated by the proposer
    // Requirements:
    // * `msg::source()` must be the applicant
    // * The proposal must be membership proposal
    // * The proposal can be aborted during only the abort window
    // * The proposal must not be aborted yet
    // Arguments:
    // * `proposal_id`: the proposal ID
    async fn abort(&mut self, proposal_id: u128) {
        if !self.proposals.contains_key(&proposal_id) {
            panic!("proposal does not exist");
        }
        if self.proposals.get(&proposal_id).unwrap().applicant != msg::source() {
            panic!("caller must be applicant");
        }
        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        if exec::block_timestamp() > proposal.starting_period + self.abort_window {
            panic!("The abort window is over");
        }
        if proposal.aborted {
            panic!("Proposal has already been aborted");
        }

        let amount = proposal.token_tribute;
        proposal.token_tribute = 0;
        proposal.aborted = true;

        transfer_tokens(
            &self.approved_token_program_id,
            &exec::program_id(),
            &msg::source(),
            amount,
        )
        .await;

        let aborted_proposal = CancelledProposal {
            member: H256::from_slice(msg::source().as_ref()),
            proposal_id,
            amount,
        };

        msg::reply(
            Event::Abort(aborted_proposal),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    // Assigns the admin position to new actor
    // Requirements:
    // * Only admin can assign new admin
    // Arguments:
    // * `new_admin`: valid actor ID
    fn set_admin(&mut self, new_admin: &ActorId) {
        if self.admin != msg::source() {
            panic!("only admin can assign new admin");
        }
        if new_admin == &ZERO_ID {
            panic!("new admin ID cant be zero");
        }
        self.admin = *new_admin;
        msg::reply(
            Event::AdminUpdated(H256::from_slice(new_admin.as_ref())),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    // Sets the delegate key that is responsible for submitting proposals and voting
    // The deleagate key defaults to member address unless updated
    // Requirements:
    // * `msg::source()` must be DAO member
    // * The delegate key must not be zero address
    // * A delegate key can be assigned only to one member
    // Arguments:
    // * `new_delegate_key`: the valid actor ID
    fn update_delegate_key(&mut self, new_delegate_key: &ActorId) {
        if !self.is_member(&msg::source()) {
            panic!("account is not a DAO member");
        }
        if self.member_by_delegate_key.contains_key(new_delegate_key) {
            panic!("cannot overwrite existing delegate keys");
        }
        if new_delegate_key == &ZERO_ID {
            panic!("newDelegateKey cannot be 0");
        }
        let member = self.members.get_mut(&msg::source()).unwrap();
        self.member_by_delegate_key
            .insert(*new_delegate_key, msg::source());
        member.delegate_key = *new_delegate_key;
    }

    // calculates the funds that the member can redeem based on his shares
    async fn redeemable_funds(&self, share: u128) -> u128 {
        let balance = balance(&self.approved_token_program_id, &exec::program_id()).await;
        (share * balance) / self.total_shares
    }

    //checks that account is DAO member
    fn is_member(&self, account: &ActorId) -> bool {
        match self.members.get(account) {
            Some(member) => {
                if member.shares == 0 {
                    return false;
                }
            }
            None => {
                return false;
            }
        }
        true
    }
}

gstd::metadata! {
    title: "DAO",
    init:
        input : InitConfig,
    handle:
        input : Action,
        output : Event,
    state:
        input: State,
        output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    DAO.admin = ActorId::new(config.admin.to_fixed_bytes());
    DAO.approved_token_program_id = ActorId::new(config.approved_token_program_id.to_fixed_bytes());
    DAO.voting_period_length = config.voting_period_length;
    DAO.period_duration = config.period_duration;
    DAO.grace_period_length = config.grace_period_length;
    DAO.abort_window = config.abort_window;
    DAO.dilution_bound = config.dilution_bound;
}

#[gstd::async_main]
async fn main() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::AddToWhiteList(input) => {
            DAO.add_to_whitelist(&ActorId::new(input.to_fixed_bytes()))
        }
        Action::SubmitMembershipProposal(input) => {
            let applicant = ActorId::new(input.applicant.to_fixed_bytes());
            DAO.submit_membership_proposal(
                &applicant,
                input.token_tribute,
                input.shares_requested,
                input.quorum,
                input.details,
            )
            .await;
        }
        Action::SubmitFundingProposal(input) => {
            let applicant = ActorId::new(input.applicant.to_fixed_bytes());
            DAO.submit_funding_proposal(&applicant, input.amount, input.quorum, input.details)
                .await;
        }
        Action::ProcessProposal(input) => {
            DAO.process_proposal(input.proposal_id).await;
        }
        Action::SubmitVote(input) => {
            DAO.submit_vote(input.proposal_id, input.vote);
        }
        Action::RageQuit(input) => {
            DAO.ragequit(input).await;
        }
        Action::Abort(input) => DAO.abort(input).await,
        Action::CancelProposal(input) => DAO.cancel_proposal(input).await,
        Action::UpdateDelegateKey(input) => {
            DAO.update_delegate_key(&ActorId::new(input.to_fixed_bytes()))
        }
        Action::SetAdmin(input) => DAO.set_admin(&ActorId::new(input.to_fixed_bytes())),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: State = msg::load().expect("failed to decode input argument");
    let encoded = match state {
        State::IsMember(input) => {
            StateReply::IsMember(DAO.is_member(&ActorId::new(input.to_fixed_bytes()))).encode()
        }
        State::IsInWhitelist(input) => StateReply::IsInWhitelist(
            DAO.whitelist
                .contains(&ActorId::new(input.to_fixed_bytes())),
        )
        .encode(),
        State::ProposalId => StateReply::ProposalId(DAO.proposal_id).encode(),
        State::ProposalInfo(input) => {
            StateReply::ProposalInfo(DAO.proposals.get(&input).unwrap().clone()).encode()
        }
        State::MemberInfo(input) => {
            let actor = ActorId::new(input.to_fixed_bytes());
            StateReply::MemberInfo(DAO.members.get(&actor).unwrap().clone()).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));
    core::mem::forget(encoded);
    result
}
