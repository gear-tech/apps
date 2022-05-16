use crate::*;
use gstd::msg;

impl RMRKToken {
    /// That function is designed to be from another RMRK contracts
    /// when minting tokens to NFT
    /// It adds a child to the NFT with tokenId `parent_token_id`
    /// The status of added child is `Pending`
    /// Requirements:
    /// * The `msg::source()` must be a deployed RMRK contract
    /// * The parent's address of the NFT in the child RMRK contract must be the address of that program
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_token_id`: is the tokenId of the child instance
    pub async fn add_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        // checks that `msg::source()` is a deployed program
        // checks that parent indicated in the child contract is the address of that program
        self.assert_parent(child_token_id).await;

        // if child already exists
        if let Some(parent) = self.children.get(&parent_token_id) {
            if let Some(_child) = parent.get(&child_token_id) {}
        }
        let child = Child {
            token_id: msg::source(),
            status: ChildStatus::Pending,
        };

        self.children
            .entry(parent_token_id)
            .and_modify(|children| {
                children.insert(child_token_id, child.clone());
            })
            .or_insert_with(|| {
                let mut a = BTreeMap::new();
                a.insert(child_token_id, child);
                a
            });

        msg::reply(
            RMRKEvent::PendingChild {
                child_token_address: msg::source(),
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    pub async fn add_accepted_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        // checks that `msg::source()` is a deployed program
        // checks that parent indicated in the child contract is the address of that program
        self.assert_parent(child_token_id).await;
        let child = Child {
            token_id: msg::source(),
            status: ChildStatus::Pending,
        };

        self.children
            .entry(parent_token_id)
            .and_modify(|children| {
                children.insert(child_token_id, child.clone());
            })
            .or_insert_with(|| {
                let mut a = BTreeMap::new();
                a.insert(child_token_id, child);
                a
            });

        msg::reply(
            RMRKEvent::PendingChild {
                child_token_address: msg::source(),
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    /// Accepts an NFT child being in the `Pending` status
    /// The status of NFT child becomes `Accepted`
    /// Requirements:
    /// * The `msg::source()` must be an NFT owner or an approved account
    /// * The parent's address of the NFT in the child RMRK contract must be the address of that program
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_token_id`: is the tokenId of the child instance
    pub fn accept_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        self.assert_approved_or_owner(parent_token_id);
        let child = self
            .children
            .get_mut(&parent_token_id)
            .expect("Parent does not exist")
            .get_mut(&child_token_id)
            .expect("Child does not exist");
        child.status = ChildStatus::Accepted;

        msg::reply(
            RMRKEvent::AcceptedChild {
                child_token_address: child.token_id,
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    pub fn reject_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        self.assert_approved_or_owner(parent_token_id);

        let children_map = self
            .children
            .get_mut(&parent_token_id)
            .expect("Parent does not exist");
        let child_token_address = children_map
            .get_mut(&child_token_id)
            .expect("Child does not exist")
            .token_id;
        children_map.remove(&child_token_id);
        msg::reply(
            RMRKEvent::PendingChildRemoved {
                child_token_address,
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    // is a copy of reject some how, need to distinguish from pending?
    pub fn remove_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        // simply remove from children and emit event
        self.assert_approved_or_owner(parent_token_id);
        let children_map = self
            .children
            .get_mut(&parent_token_id)
            .expect("Parent does not exist");
        let child_token_address = children_map
            .get_mut(&child_token_id)
            .expect("Child does not exist")
            .token_id;
        children_map.remove(&child_token_id);
        msg::reply(
            RMRKEvent::ChildRejected {
                child_token_address,
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    pub fn burn_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        let child = self
            .children
            .get_mut(&parent_token_id)
            .expect("Parent does not exist")
            .get_mut(&child_token_id)
            .expect("Child does not exist")
            .clone();
        if child.token_id != msg::source() {
            panic!("the caller must be the child nft contract");
        }
        self.children.entry(parent_token_id).and_modify(|children| {
            children.remove(&child_token_id);
        });

        msg::reply(
            RMRKEvent::ChildBurnt {
                parent_token_id,
                child_token_id,
                child_status: child.status,
            },
            0,
        )
        .unwrap();
    }
}
