use std::collections::{HashMap};

use near_sdk::{AccountId, log, near_bindgen, require, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap};
use near_sdk::json_types::{U128, U64};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MultisigContract {
    multisigs: LookupMap<U64, Multisig>,
    next_id: U64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Multisig {
    id: U64,
    accounts: HashMap<AccountId, bool>,
    sum: U128,
    approve_ratio: f32,
    destination_wallet: AccountId,
}

impl Default for MultisigContract {
    fn default() -> Self {
        MultisigContract { multisigs: LookupMap::new(b"m"), next_id: U64(0) }
    }
}

#[near_bindgen]
impl MultisigContract {
    fn add_new_multisig(&mut self, ids: Vec<AccountId>, sum: U128, approve_ratio: f32, destination_wallet: AccountId) {
        require!(!ids.is_empty(), "Multisig size should be bigger than 0");
        require!(sum.0 > 0, "Sum should be positive number");
        // todo to check that destination wallet exists
        require!(approve_ratio > 0 && approve_ratio <= 1, "Percents");

        let mut accounts = HashMap::new();
        for k in ids {
            accounts.entry(k).or_insert(false);
        }

        self.multisigs.insert(&self.next_id, &Multisig {
            id: self.next_id,
            accounts,
            sum,
            approve_ratio,
            destination_wallet,
        });

        self.next_id = U64(self.next_id.0 + 1);
    }

    #[payable]
    fn send_transaction(&mut self, multisig_id: U64) {
        let maybe_multisig = self.multisigs.get(&multisig_id);

        require!(maybe_multisig.is_some(), "Multisig has been not found");
        let mut multisig = maybe_multisig.unwrap();

        let near_amount = near_sdk::env::attached_deposit();
        require!(near_amount == multisig.sum.0, "Sum is required");

        let id = near_sdk::env::predecessor_account_id();
        require!(multisig.accounts.get(&id.clone()).is_some(), "Account should be in a multisig map");

        require!(multisig.accounts.get(&id.clone()).unwrap() == &false, "Is possible to send funds only once");
        multisig.accounts.entry(id.clone()).and_modify(|_| { true; });

        let accounts_len = multisig.accounts.len();
        let accepted_accounts_len = multisig.accounts.iter().filter(|&a| { *a.1 == true }).count();
        let i = accounts_len as f32 / accepted_accounts_len as f32;
        if i > multisig.approve_ratio {
            Promise::new(multisig.destination_wallet).transfer(multisig.sum * 5) // todo incorrect, amount has to be fixed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
