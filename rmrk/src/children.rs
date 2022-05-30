use crate::*;
use gstd::msg;

fn get_child_vec(child_contract_id: &ActorId, child_token_id: TokenId) -> Vec<u8> {
    let mut nft_contract_and_token: Vec<u8> = <[u8; 32]>::from(*child_contract_id).into();
    let token_id_vec: Vec<u8> = <[u8; 32]>::from(child_token_id).into();
    nft_contract_and_token.extend(token_id_vec);
    nft_contract_and_token
}

impl RMRKToken {
    /// That function is designed to be called from another RMRK contracts
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
        let rmrk_owner = self
            .rmrk_owners
            .get(&parent_token_id)
            .expect("Token does not exist");
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(&msg::source(), child_token_id);
        if let Some(children) = self.pending_children.get(&parent_token_id) {
            // if child already exists
            if children.contains(&child_vec) {
                panic!("RMRKCore: child already exists");
            }
        }

        let root_owner = if rmrk_owner.token_id.is_some() {
            get_root_owner(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap()).await
        } else {
            rmrk_owner.owner_id
        };

        self.pending_children
            .entry(parent_token_id)
            .and_modify(|children| {
                children.insert(child_vec.clone());
            })
            .or_insert_with(|| {
                let mut a = BTreeSet::new();
                a.insert(child_vec.clone());
                a
            });

        self.children_status.insert(child_vec, ChildStatus::Pending);
        msg::reply(
            RMRKEvent::PendingChild {
                child_token_address: msg::source(),
                child_token_id,
                parent_token_id,
                root_owner,
            },
            0,
        )
        .unwrap();
    }

    /// That function is designed to be called from another RMRK contracts
    /// when owner transfers his accepted child to another parent token in one contract
    /// It adds a child to the RMRK token with tokenId `parent_token_id`
    /// Requirements:
    /// * The `msg::source()` must be a deployed RMRK contract
    /// * The `from` must be an existing RMRK token that must have `child_token_id` in its `accepted_children`
    /// * The `to` must be an existing RMRK token
    /// * The `root_owner` of `to` and `from` must be the same
    /// Arguments:
    /// * `from`: RMRK token from which the child token will be transferred
    /// * `to`: RMRK token to which the child token will be transferred
    /// * `child_token_id`: is the tokenId of the child of the RMRK child contract
    pub async fn transfer_child(&mut self, from: TokenId, to: TokenId, child_token_id: TokenId) {
        // checks that `msg::source()` is a deployed program

        self.assert_token_does_not_exist(to);
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(&msg::source(), child_token_id);

        // check the status of the child
        let child_status = self
            .children_status
            .get(&child_vec)
            .expect("The child does not exist");

        let from_root_owner = self.find_root_owner(from).await;
        let to_root_owner = self.find_root_owner(to).await;
        self.assert_exec_origin(&from_root_owner);

        match child_status {
            ChildStatus::Pending => {
                self.pending_children.entry(from).and_modify(|c| {
                    c.remove(&child_vec);
                });
                self.pending_children
                    .entry(to)
                    .and_modify(|c| {
                        c.insert(child_vec.clone());
                    })
                    .or_insert_with(|| {
                        let mut c = BTreeSet::new();
                        c.insert(child_vec);
                        c
                    });
            }
            ChildStatus::Accepted => {
                self.accepted_children.entry(from).and_modify(|c| {
                    c.remove(&child_vec);
                });
                if from_root_owner == to_root_owner {
                    self.accepted_children
                        .entry(to)
                        .and_modify(|c| {
                            c.insert(child_vec.clone());
                        })
                        .or_insert_with(|| {
                            let mut c = BTreeSet::new();
                            c.insert(child_vec);
                            c
                        });
                } else {
                    self.pending_children
                        .entry(to)
                        .and_modify(|c| {
                            c.insert(child_vec.clone());
                        })
                        .or_insert_with(|| {
                            let mut c = BTreeSet::new();
                            c.insert(child_vec);
                            c
                        });
                }
            }
        }
        msg::reply(
            RMRKEvent::ChildTransferred {
                from,
                to,
                child_contract_id: msg::source(),
                child_token_id,
            },
            0,
        )
        .unwrap();
    }

    /// That function is designed to be called from another RMRK contracts
    /// when owner transfers his accepted child to another parent token in one contract
    /// It adds a child to the RMRK token with tokenId `parent_token_id`
    /// Requirements:
    /// * The `msg::source()` must be a deployed RMRK contract
    /// * The `parent_token_id` must be an existing RMRK token that must have `child_token_id` in its `accepted_children`
    /// * The `root_owner` of `to` and `from` must be the same
    /// Arguments:
    /// * `from`: RMRK token from which the child token will be transferred
    /// * `to`: RMRK token to which the child token will be transferred
    /// * `child_token_id`: is the tokenId of the child of the RMRK child contract
    pub async fn add_accepted_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        // checks that `msg::source()` is a deployed program

        self.assert_token_does_not_exist(parent_token_id);
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(&msg::source(), child_token_id);
        let root_owner = self.find_root_owner(parent_token_id).await;
        self.assert_exec_origin(&root_owner);

        self.accepted_children
            .entry(parent_token_id)
            .and_modify(|children| {
                children.insert(child_vec.clone());
            })
            .or_insert_with(|| {
                let mut a = BTreeSet::new();
                a.insert(child_vec.clone());
                a
            });
        self.children_status
            .insert(child_vec, ChildStatus::Accepted);

        msg::reply(
            RMRKEvent::AcceptedChild {
                child_token_address: msg::source(),
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }
    /// Accepts an RMRK child being in the `Pending` status
    /// Removes RMRK child from `pending_children` and adds to `accepted_children`
    /// Requirements:
    /// * The `msg::source()` must be an RMRK owner or an approved account
    /// * The parent's address of the NFT in the child RMRK contract must be the address of that program
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_token_id`: is the tokenId of the child instance
    pub async fn accept_child(
        &mut self,
        parent_token_id: TokenId,
        child_contract_id: &ActorId,
        child_token_id: TokenId,
    ) {
        let root_owner = self.find_root_owner(parent_token_id).await;
        self.assert_approved_or_owner(parent_token_id, &root_owner);
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(child_contract_id, child_token_id);

        if let Some(children) = self.pending_children.get_mut(&parent_token_id) {
            if children.contains(&child_vec) {
                children.remove(&child_vec);
                self.accepted_children
                    .entry(parent_token_id)
                    .and_modify(|c| {
                        c.insert(child_vec.clone());
                    })
                    .or_insert_with(|| {
                        let mut a = BTreeSet::new();
                        a.insert(child_vec.clone());
                        a
                    });
            } else {
                panic!("RMRKCore: child does not exist or has already been accepted");
            }
        }
        self.children_status
            .insert(child_vec, ChildStatus::Accepted);
        msg::reply(
            RMRKEvent::AcceptedChild {
                child_token_address: *child_contract_id,
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    /// Rejects an RMRK child being in the `Pending` status
    /// Requirements:
    /// * The `msg::source()` must be an RMRK owner or an approved account
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_contract_id`: is the address of the child RMRK contract
    /// * `child_token_id`: is the tokenId of the child instance
    pub async fn reject_child(
        &mut self,
        parent_token_id: TokenId,
        child_contract_id: &ActorId,
        child_token_id: TokenId,
    ) {
        let root_owner = self.find_root_owner(parent_token_id).await;
        self.assert_approved_or_owner(parent_token_id, &root_owner);

        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(child_contract_id, child_token_id);

        if let Some(children) = self.pending_children.get_mut(&parent_token_id) {
            if children.contains(&child_vec) {
                children.remove(&child_vec);
                // sends message to child contract to burn RMRK token from it
                burn_from_parent(child_contract_id, vec![child_token_id], &root_owner).await;
            } else {
                panic!("RMRKCore: child does not exist or has already been approved");
            }
        } else {
            panic!("RMRKCore: there are no pending children");
        }
        self.children_status.remove(&child_vec);
        msg::reply(
            RMRKEvent::RejectedChild {
                child_token_address: *child_contract_id,
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    /// Rmoves an RMRK child being in the `Accepted` status
    /// Requirements:
    /// * The `msg::source()` must be an RMRK owner or an approved account
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_contract_id`: is the address of the child RMRK contract
    /// * `child_token_id`: is the tokenId of the child instance
    pub async fn remove_child(
        &mut self,
        parent_token_id: TokenId,
        child_contract_id: &ActorId,
        child_token_id: TokenId,
    ) {
        let root_owner = self.find_root_owner(parent_token_id).await;
        self.assert_approved_or_owner(parent_token_id, &root_owner);
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(child_contract_id, child_token_id);

        if let Some(children) = self.accepted_children.get_mut(&parent_token_id) {
            if children.contains(&child_vec) {
                children.remove(&child_vec);
                burn_from_parent(child_contract_id, vec![child_token_id], &root_owner).await;
            } else {
                panic!("RMRKCore: child does not exist");
            }
        } else {
            panic!("RMRKCore: there are no accepted children");
        }
        self.children_status.remove(&child_vec);
        msg::reply(
            RMRKEvent::RemovedChild {
                child_token_address: *child_contract_id,
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();
    }

    /// Burns a child of RMRK token
    /// That function must be called from the child RMRK contract during `transfer`, `transfer_to_nft` and `burn` functions
    /// Requirements:
    /// * The `msg::source()` must be a child RMRK contract
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_token_id`: is the tokenId of the child instance
    pub fn burn_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        let child_vec = get_child_vec(&msg::source(), child_token_id);
        let child_status = self
            .children_status
            .remove(&child_vec)
            .expect("Child does not exist");
        debug!("CHILD STATUS {:?}", child_status);
        match child_status {
            ChildStatus::Pending => {
                if let Some(children) = self.pending_children.get_mut(&parent_token_id) {
                    children.remove(&child_vec);
                }
            }
            ChildStatus::Accepted => {
                if let Some(children) = self.accepted_children.get_mut(&parent_token_id) {
                    children.remove(&child_vec);
                }
            }
        }

        msg::reply(
            RMRKEvent::ChildBurnt {
                parent_token_id,
                child_token_id,
            },
            0,
        )
        .unwrap();
    }
}
