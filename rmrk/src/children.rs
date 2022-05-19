use crate::*;
use gstd::msg;

fn get_child_vec(child_contract_id: &ActorId, child_token_id: TokenId) -> Vec<u8> {
    let mut nft_contract_and_token: Vec<u8> = <[u8; 32]>::from(*child_contract_id).into();
    let token_id_vec: Vec<u8> = <[u8; 32]>::from(child_token_id).into();
    nft_contract_and_token.extend(token_id_vec);
    nft_contract_and_token
}

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
        let rmrk_owner = self
            .rmrk_owners
            .get(&parent_token_id)
            .expect("Token does not exist");
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(&msg::source(), child_token_id);
        if let Some(children) = self.parent_to_children.get(&parent_token_id) {
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

        self.parent_to_children
            .entry(parent_token_id)
            .and_modify(|children| {
                children.push(child_vec.clone());
            })
            .or_insert_with(|| vec![child_vec.clone()]);

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

    /// Accepts an RMRK child being in the `Pending` status
    /// The status of NFT child becomes `Accepted`
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
        self.assert_approved_or_owner(parent_token_id).await;
        // get the vector of `child_nft_contract` + `child_token_id`
        let child_vec = get_child_vec(child_contract_id, child_token_id);

        if let Some(children) = self.parent_to_children.get(&parent_token_id) {
            if children.contains(&child_vec) {
                self.children_status
                    .insert(child_vec, ChildStatus::Accepted);
            } else {
                panic!("RMRKCore: child does not exist");
            }
        } else {
            panic!("RMRKCore: token has no children");
        }

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

    /// Burns a child of RMRK token
    /// That function must be called from the child RMRK contract during `transfer`, `transfer_to_nft` and `burn` functions
    /// Requirements:
    /// * The `msg::source()` must be a child RMRK contract
    /// Arguments:
    /// * `parent_token_id`: is the tokenId of the parent NFT
    /// * `child_token_id`: is the tokenId of the child instance
    pub fn burn_child(&mut self, parent_token_id: TokenId, child_token_id: TokenId) {
        let child_vec = get_child_vec(&msg::source(), child_token_id);
        if let Some(children) = self.parent_to_children.get_mut(&parent_token_id) {
            if let Some(index) = children.iter().position(|child| child == &child_vec) {
                children.swap_remove(index);
            } else {
                panic!("RMRKCore: child does not exist");
            }
        } else {
            panic!("RMRKCore: token has no children");
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
