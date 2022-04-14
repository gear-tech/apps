use crate::erc1155::io::*;
use gstd::{exec, msg, prelude::*, ActorId};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

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

pub trait ERC1155TokenAssert: StateKeeper + BalanceTrait {
    fn assert_can_burn(&mut self, owner: &ActorId, id: &TokenId, amount: u128) {
        if self.get_balance(owner, id) < amount {
            panic!("ERC1155: Not enough balance");
        }
        self.assert_owner(id);
    }

    fn assert_owner(&self, id: &TokenId) {
        if self.get_balance(&msg::source(), id) != 0 || self.get_balance(&exec::origin(), id) != 0 {
            panic!("ERC11555: Not allowed to apporve");
        }
    }

    fn assert_can_transfer(&self, from: &ActorId, id: &u128, amount: u128) {
        if !(from == &msg::source()
            || from == &exec::origin()
            || self.get_balance(&msg::source(), id) >= amount)
        {
            panic!("ERC1155: Wrong owner or insufficient balance");
        }
    }

    fn assert_approved(&self, owner: &ActorId, operator: &ActorId) {
        if !self.get().operator_approvals.contains_key(owner)
            && *self.get().operator_approvals[owner]
                .get(operator)
                .unwrap_or(&false)
        {
            panic!("ERC1155: Caller is not approved");
        }
    }
}

pub trait ERC1155Core: StateKeeper + BalanceTrait + ERC1155TokenAssert {
    fn mint(&mut self, account: &ActorId, id: &TokenId, amount: u128, meta: Option<TokenMetadata>) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address")
        }
        if let Some(metadata) = meta {
            if amount > 1 {
                panic!("ERC1155: Mint metadata to a fungible token")
            }
            self.get_mut().token_metadata.insert(*id, metadata);
        }
        let prev_balance = self.get_balance(account, id);
        self.set_balance(account, id, prev_balance.saturating_add(amount));
    }

    fn mint_batch(
        &mut self,
        account: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
        meta: Vec<Option<TokenMetadata>>,
    ) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address")
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }
        meta.into_iter()
            .enumerate()
            .for_each(|(i, meta)| self.mint(account, &ids[i], amounts[i], meta));

        msg::reply(
            Event::TransferBatch {
                operator: msg::source(),
                from: ZERO_ID,
                to: *account,
                ids: ids.to_vec(),
                values: amounts.to_vec(),
            },
            0,
        )
        .unwrap();
    }

    fn burn(&mut self, id: &TokenId, amount: u128) {
        let owner = &msg::source();
        self.assert_can_burn(owner, id, amount);
        self.set_balance(
            &msg::source(),
            id,
            self.get_balance(owner, id).saturating_sub(amount),
        );
    }

    fn burn_batch(&mut self, ids: &[TokenId], amounts: &[u128]) {
        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        for (id, amount) in ids.iter().zip(amounts) {
            self.assert_can_burn(&msg::source(), id, *amount);
        }

        ids.iter()
            .enumerate()
            .for_each(|(i, id)| self.burn(id, amounts[i]));

        msg::reply(
            Event::TransferBatch {
                operator: msg::source(),
                from: msg::source(),
                to: ZERO_ID,
                ids: ids.to_vec(),
                values: amounts.to_vec(),
            },
            0,
        )
        .unwrap();
    }

    fn transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &TokenId, amount: u128) {
        if from == to {
            panic!("ERC1155: sender and recipient addresses are the same")
        }

        self.assert_approved(from, &msg::source());
        if from != &msg::source() {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        let from_balance = self.get_balance(from, id);

        if from_balance < amount {
            panic!("ERC1155: insufficient balance for transfer")
        }
        self.set_balance(from, id, from_balance.saturating_sub(amount));
        let to_balance = self.get_balance(to, id);
        self.set_balance(to, id, to_balance.saturating_add(amount));
    }

    fn batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
    ) {
        if from == to {
            panic!("ERC1155: sender and recipient addresses are the same")
        }

        self.assert_approved(from, &msg::source());
        if from != &msg::source() {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        for (id, amount) in ids.iter().zip(amounts) {
            self.assert_can_transfer(from, id, *amount);
        }

        ids.iter()
            .enumerate()
            .for_each(|(i, id)| self.transfer_from(from, to, id, amounts[i]));

        msg::reply(
            Event::TransferBatch {
                operator: msg::source(),
                from: *from,
                to: *to,
                ids: ids.to_vec(),
                values: amounts.to_vec(),
            },
            0,
        )
        .unwrap();
    }

    fn approve(&mut self, to: &ActorId) {
        if to == &ZERO_ID {
            panic!("ERC1155: approving zero address")
        }
        self.get_mut()
            .operator_approvals
            .entry(msg::source())
            .or_default()
            .insert(*to, true);
        msg::reply(
            Event::Approve {
                from: msg::source(),
                to: *to,
            },
            0,
        )
        .unwrap();
    }

    fn revoke_approval(&mut self, to: &ActorId) {
        if to == &ZERO_ID {
            panic!("ERC1155: revoking zero address")
        }
        self.get_mut()
            .operator_approvals
            .entry(msg::source())
            .or_default()
            .remove_entry(to);

        msg::reply(
            Event::Approve {
                from: msg::source(),
                to: *to,
            },
            0,
        )
        .unwrap();
    }

    fn balance_of_batch(&self, accounts: &[ActorId], ids: &[TokenId]) {
        if accounts.len() != ids.len() {
            panic!("ERC1155: accounts and ids length mismatch")
        }

        let res = ids
            .iter()
            .zip(accounts)
            .map(|(id, account)| BalanceOfBatchReply {
                account: *account,
                id: *id,
                amount: self.get_balance(account, id),
            })
            .collect();

        msg::reply(Event::BalanceOfBatch(res), 0).unwrap();
    }

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        let action = Action::decode(&mut &bytes[..]).ok()?;
        match action {
            Action::Mint(account, id, amount, meta) => {
                Self::mint(self, &account, &id, amount, meta);
                msg::reply(
                    Event::TransferSingle(TransferSingleReply {
                        operator: msg::source(),
                        from: ZERO_ID,
                        to: account,
                        id,
                        amount,
                    }),
                    0,
                )
                .unwrap();
            }
            Action::MintBatch(account, ids, amounts, metas) => {
                Self::mint_batch(self, &account, &ids, &amounts, metas)
            }

            Action::Burn(id, amount) => {
                Self::burn(self, &id, amount);
                msg::reply(
                    Event::TransferSingle(TransferSingleReply {
                        operator: msg::source(),
                        from: msg::source(),
                        to: ZERO_ID,
                        id,
                        amount,
                    }),
                    0,
                )
                .unwrap();
            }
            Action::BurnBatch(ids, amounts) => Self::burn_batch(self, &ids, &amounts),

            Action::TransferFrom(from, to, id, amount) => {
                Self::transfer_from(self, &from, &to, &id, amount);
                msg::reply(
                    Event::TransferSingle(TransferSingleReply {
                        operator: msg::source(),
                        from,
                        to,
                        id,
                        amount,
                    }),
                    0,
                )
                .unwrap();
            }
            Action::BatchTransferFrom(from, to, ids, amounts) => {
                Self::batch_transfer_from(self, &from, &to, &ids, &amounts)
            }

            Action::BalanceOf(account, id) => {
                msg::reply(Event::Balance(Self::get_balance(self, &account, &id)), 0).unwrap();
            }
            Action::BalanceOfBatch(accounts, ids) => Self::balance_of_batch(self, &accounts, &ids),

            Action::Approve(to) => Self::approve(self, &to),
            Action::RevokeApproval(to) => Self::revoke_approval(self, &to),
        };
        Some(())
    }
}
