use crate::multitoken::io::*;
use gstd::{prelude::*, ActorId};

#[derive(Debug, Default)]
pub struct MTKState {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub balances: BTreeMap<TokenId, BTreeMap<ActorId, u128>>,
    pub operator_approvals: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
    pub token_metadata: BTreeMap<TokenId, TokenMetadata>,
}

pub trait StateKeeper {
    fn get(&self) -> &MTKState;
    fn get_mut(&mut self) -> &mut MTKState;
}

pub trait BalanceTrait: StateKeeper {
    fn get_balance(&self, account: &ActorId, id: &TokenId) -> u128 {
        *self
            .get()
            .balances
            .get(id)
            .and_then(|m| m.get(account))
            .unwrap_or(&0)
    }

    fn set_balance(&mut self, account: &ActorId, id: &TokenId, amount: u128) {
        let mut _balance = self
            .get_mut()
            .balances
            .entry(*id)
            .or_default()
            .insert(*account, amount);
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum MTKQuery {
    Name,
    Symbol,
    Uri,
    BalanceOf(ActorId, TokenId),
    MetadataOf(TokenId),
    URI(TokenId),
    TokensForOwner(ActorId),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum MTKQueryReply {
    Name(String),
    Symbol(String),
    Uri(String),
    Balance(TokenId),
    URI(String),
    MetadataOf(TokenMetadata),
    TokensForOwner(Vec<TokenId>),
}

pub trait MTKTokenState: StateKeeper + BalanceTrait {
    fn get_uri(&self, id: TokenId) -> String {
        self.get()
            .base_uri
            .clone()
            .replace("{id}", &format!("{}", id))
    }

    fn get_metadata(&self, id: TokenId) -> TokenMetadata {
        self.get()
            .token_metadata
            .get(&id)
            .unwrap_or(&TokenMetadata {
                ..Default::default()
            })
            .clone()
    }

    fn tokens_for_owner(&self, owner: &ActorId) -> Vec<TokenId> {
        let mut tokens: Vec<TokenId> = Vec::new();
        let balances = &self.get().balances;
        for (token, bals) in balances {
            if let Some(_user) = bals.get(owner) {
                tokens.push(*token);
            }
        }
        tokens
    }

    fn proc_state(&mut self, bytes: Vec<u8>) -> Option<Vec<u8>> {
        let query = MTKQuery::decode(&mut &bytes[..]).ok()?;
        let encoded = match query {
            MTKQuery::Name => MTKQueryReply::Name(self.get().name.clone()).encode(),
            MTKQuery::Symbol => MTKQueryReply::Symbol(self.get().symbol.clone()).encode(),
            MTKQuery::Uri => MTKQueryReply::Uri(self.get().base_uri.clone()).encode(),
            MTKQuery::BalanceOf(account, id) => {
                MTKQueryReply::Balance(Self::get_balance(self, &account, &id)).encode()
            }
            MTKQuery::URI(id) => MTKQueryReply::URI(Self::get_uri(self, id)).encode(),
            MTKQuery::MetadataOf(id) => {
                MTKQueryReply::MetadataOf(Self::get_metadata(self, id)).encode()
            }
            MTKQuery::TokensForOwner(owner) => {
                MTKQueryReply::TokensForOwner(Self::tokens_for_owner(self, &owner)).encode()
            }
        };
        Some(encoded)
    }
}
