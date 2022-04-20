use crate::erc1155::io::*;
use gstd::{prelude::*, ActorId};

#[derive(Debug, Default)]
pub struct ERC1155State {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub balances: BTreeMap<TokenId, BTreeMap<ActorId, u128>>,
    pub operator_approvals: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
    pub token_metadata: BTreeMap<TokenId, TokenMetadata>,
}

pub trait StateKeeper {
    fn get(&self) -> &ERC1155State;
    fn get_mut(&mut self) -> &mut ERC1155State;
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
pub enum ERC1155Query {
    Name,
    Symbol,
    Uri,
    BalanceOf(ActorId, TokenId),
    MetadataOf(TokenId),
    URI(TokenId),
    TokensForOwner(ActorId),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum ERC1155QueryReply {
    Name(String),
    Symbol(String),
    Uri(String),
    Balance(TokenId),
    URI(String),
    MetadataOf(TokenMetadata),
    TokensForOwner(Vec<TokenId>),
}

pub trait ERC1155TokenState: StateKeeper + BalanceTrait {
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
        let query = ERC1155Query::decode(&mut &bytes[..]).ok()?;
        let encoded = match query {
            ERC1155Query::Name => ERC1155QueryReply::Name(self.get().name.clone()).encode(),
            ERC1155Query::Symbol => ERC1155QueryReply::Symbol(self.get().symbol.clone()).encode(),
            ERC1155Query::Uri => ERC1155QueryReply::Uri(self.get().base_uri.clone()).encode(),
            ERC1155Query::BalanceOf(account, id) => {
                ERC1155QueryReply::Balance(Self::get_balance(self, &account, &id)).encode()
            }
            ERC1155Query::URI(id) => ERC1155QueryReply::URI(Self::get_uri(self, id)).encode(),
            ERC1155Query::MetadataOf(id) => {
                ERC1155QueryReply::MetadataOf(Self::get_metadata(self, id)).encode()
            }
            ERC1155Query::TokensForOwner(owner) => {
                ERC1155QueryReply::TokensForOwner(Self::tokens_for_owner(self, &owner)).encode()
            }
        };
        Some(encoded)
    }
}
