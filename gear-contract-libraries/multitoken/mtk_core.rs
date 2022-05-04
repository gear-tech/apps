use crate::multitoken::{io::*, state::*};
use gstd::{exec, msg, prelude::*, ActorId};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

pub trait MTKTokenAssert: StateKeeper + BalanceTrait {
    fn assert_can_burn(&mut self, owner: &ActorId, id: &TokenId, amount: u128) {
        if self.get_balance(owner, id) < amount {
            panic!("MTK: Not enough balance");
        }
    }

    fn assert_can_transfer(&self, from: &ActorId, id: &u128, amount: u128) {
        if !(from == &msg::source()
            || from == &exec::origin()
            || self.get_balance(&msg::source(), id) >= amount)
        {
            panic!("MTK: Wrong owner or insufficient balance");
        }
    }

    fn assert_approved(&self, owner: &ActorId, operator: &ActorId) {
        if !self.get().operator_approvals.contains_key(owner)
            && *self.get().operator_approvals[owner]
                .get(operator)
                .unwrap_or(&false)
        {
            panic!("MTK: Caller is not approved");
        }
    }
}

pub trait MTKCore: StateKeeper + BalanceTrait + MTKTokenAssert {
    fn mint_impl(
        &mut self,
        account: &ActorId,
        id: &TokenId,
        amount: u128,
        meta: Option<TokenMetadata>,
    ) {
        if account == &ZERO_ID {
            panic!("MTK: Mint to zero address")
        }
        if let Some(metadata) = meta {
            if amount > 1 {
                panic!("MTK: Mint metadata to a fungible token")
            }
            self.get_mut().token_metadata.insert(*id, metadata);
        }
        let prev_balance = self.get_balance(account, id);
        self.set_balance(account, id, prev_balance.saturating_add(amount));
    }

    fn mint(&mut self, account: &ActorId, id: &TokenId, amount: u128, meta: Option<TokenMetadata>) {
        self.mint_impl(account, id, amount, meta);
        msg::reply(
            MTKEvent::TransferSingle(TransferSingleReply {
                operator: msg::source(),
                from: ZERO_ID,
                to: *account,
                id: *id,
                amount,
            }),
            0,
        )
        .unwrap();
    }

    fn mint_batch(
        &mut self,
        account: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
        meta: Vec<Option<TokenMetadata>>,
    ) {
        if account == &ZERO_ID {
            panic!("MTK: Mint to zero address")
        }

        if ids.len() != amounts.len() {
            panic!("MTK: ids and amounts length mismatch")
        }

        meta.into_iter()
            .enumerate()
            .for_each(|(i, meta)| self.mint_impl(account, &ids[i], amounts[i], meta));

        msg::reply(
            MTKEvent::TransferBatch {
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

    fn burn_impl(&mut self, id: &TokenId, amount: u128) {
        let owner = &msg::source();
        self.assert_can_burn(owner, id, amount);
        self.set_balance(
            &msg::source(),
            id,
            self.get_balance(owner, id).saturating_sub(amount),
        );
    }

    fn burn(&mut self, id: &TokenId, amount: u128) {
        self.burn_impl(id, amount);
        msg::reply(
            MTKEvent::TransferSingle(TransferSingleReply {
                operator: msg::source(),
                from: msg::source(),
                to: ZERO_ID,
                id: *id,
                amount,
            }),
            0,
        )
        .unwrap();
    }

    fn burn_batch(&mut self, ids: &[TokenId], amounts: &[u128]) {
        if ids.len() != amounts.len() {
            panic!("MTK: ids and amounts length mismatch")
        }

        for (id, amount) in ids.iter().zip(amounts) {
            self.assert_can_burn(&msg::source(), id, *amount);
        }

        ids.iter()
            .enumerate()
            .for_each(|(i, id)| self.burn_impl(id, amounts[i]));

        msg::reply(
            MTKEvent::TransferBatch {
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

    fn transfer_from_impl(&mut self, from: &ActorId, to: &ActorId, id: &TokenId, amount: u128) {
        if from == to {
            panic!("MTK: sender and recipient addresses are the same")
        }

        if from != &msg::source() {
            panic!("MTK: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("MTK: transfer to the zero address")
        }
        let from_balance = self.get_balance(from, id);

        if from_balance < amount {
            panic!("MTK: insufficient balance for transfer")
        }
        self.set_balance(from, id, from_balance.saturating_sub(amount));
        let to_balance = self.get_balance(to, id);
        self.set_balance(to, id, to_balance.saturating_add(amount));
    }

    fn transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &TokenId, amount: u128) {
        self.transfer_from_impl(from, to, id, amount);
        msg::reply(
            MTKEvent::TransferSingle(TransferSingleReply {
                operator: msg::source(),
                from: *from,
                to: *to,
                id: *id,
                amount,
            }),
            0,
        )
        .unwrap();
    }

    fn batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
    ) {
        if from == to {
            panic!("MTK: sender and recipient addresses are the same")
        }

        // self.assert_approved(from, &msg::source());
        if from != &msg::source() {
            panic!("MTK: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("MTK: transfer to the zero address")
        }

        if ids.len() != amounts.len() {
            panic!("MTK: ids and amounts length mismatch")
        }

        for (id, amount) in ids.iter().zip(amounts) {
            self.assert_can_transfer(from, id, *amount);
        }

        ids.iter()
            .enumerate()
            .for_each(|(i, id)| self.transfer_from_impl(from, to, id, amounts[i]));

        msg::reply(
            MTKEvent::TransferBatch {
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
            panic!("MTK: approving zero address")
        }
        self.get_mut()
            .operator_approvals
            .entry(msg::source())
            .or_default()
            .insert(*to, true);
        msg::reply(
            MTKEvent::Approve {
                from: msg::source(),
                to: *to,
            },
            0,
        )
        .unwrap();
    }

    fn revoke_approval(&mut self, to: &ActorId) {
        if to == &ZERO_ID {
            panic!("MTK: revoking zero address")
        }
        self.get_mut()
            .operator_approvals
            .entry(msg::source())
            .or_default()
            .remove_entry(to);

        msg::reply(
            MTKEvent::Approve {
                from: msg::source(),
                to: *to,
            },
            0,
        )
        .unwrap();
    }

    fn balance_of(&self, account: &ActorId, id: &TokenId) {
        msg::reply(MTKEvent::Balance(self.get_balance(account, id)), 0).unwrap();
    }

    fn balance_of_batch(&self, accounts: &[ActorId], ids: &[TokenId]) {
        if accounts.len() != ids.len() {
            panic!("MTK: accounts and ids length mismatch")
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

        msg::reply(MTKEvent::BalanceOfBatch(res), 0).unwrap();
    }
}
