use codec::{Decode, Encode};
use scale_info::TypeInfo;
use primitive_types::H256;
use gstd::{String, prelude::*, ActorId};

#[derive(Debug, Decode, TypeInfo)]
pub struct Royalties {
    pub accounts: BTreeMap<ActorId, u8>,
    pub fee: u8,
}

#[derive(Debug, Decode, TypeInfo)]
pub struct Payout {
    payout: BTreeMap<ActorId, u128>,
}

impl Royalties {
    pub fn validate(&self) {
        if self.fee > 100 {
            panic!("royalty fee must be less 100");
        }
       
        let mut total_fee: u8 = 0;
        self.accounts.iter().for_each(|(_, fee)| {
            if *fee > 100 {
                panic!("fee must be less than 100");
            }
            total_fee += fee;
        });
        if total_fee >= 100 {
            panic!("total fee  must be less than 100");
        }
    }

    fn calculate_fee(
        fee: u8, 
        value: u128
    ) -> u128 {
        fee as u128 * value  / 100u128
    }

    fn create_payout(
        &self, 
        price: u128, 
        owner_id: &ActorId
    ) -> Payout {
        let royalty_payment = Royalties::calculate_fee(self.fee, price);
        let payout = Payout {
            payout: self
                .accounts
                .iter()
                .map(|(account, fee)| {
                    (
                        account.clone(),
                        Royalties::calculate_fee(*fee, royalty_payment).into(),
                    )
                })
                .collect(),
        };
        // let rest = price - royalty_payment;
        // let owner_payout: u128 = payout.payout.get(owner_id).map_or(0, |x| x.0) + rest;
        // payout.payout.insert(owner_id.clone(), owner_payout.into());
        payout
    }
}

