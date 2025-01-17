use super::core_impl::FungibleToken;
use super::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};
use crate::{types::panic_err, EngineFungibleToken};
use aurora_engine_types::types::NEP141Wei;
use near_sdk::{assert_one_yocto, env, json_types::U128, AccountId, Balance, Promise};

impl FungibleToken {
    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister(
        &mut self,
        account_id: AccountId,
        force: Option<bool>,
    ) -> Option<(AccountId, NEP141Wei)> {
        assert_one_yocto();

        let force = force.unwrap_or(false);
        if let Some(balance) = self.get_account_eth_balance(&account_id) {
            if balance == NEP141Wei::new(0) || force {
                self.accounts_remove(&account_id);
                self.total_eth_supply_on_near -= balance;
                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
                Some((account_id, balance))
            } else {
                panic_err(error::StorageFundingError::UnRegisterPositiveBalance);
            }
        } else {
            crate::log!("The account {} is not registered", &account_id);
            None
        }
    }

    pub fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        if self.accounts_eth.contains_key(account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }
}

impl StorageManagement for FungibleToken {
    // `registration_only` doesn't affect the implementation for vanilla fungible token.
    #[allow(unused_variables)]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.engine_storage_deposit(env::predecessor_account_id(), account_id, registration_only)
    }

    /// While storage_withdraw normally allows the caller to retrieve `available` balance, the basic
    /// Fungible Token implementation sets storage_balance_bounds.min == storage_balance_bounds.max,
    /// which means available balance will always be 0. So this implementation:
    /// * panics if `amount > 0`
    /// * never transfers Ⓝ to caller
    /// * returns a `storage_balance` struct if `amount` is 0
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        self.engine_storage_withdraw(env::predecessor_account_id(), amount)
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.engine_storage_unregister(env::predecessor_account_id(), force)
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> StorageBalance {
        self.internal_storage_balance_of(&account_id)
            .unwrap_or_default()
    }
}

pub mod error {
    use crate::errors;

    #[derive(Debug)]
    pub enum StorageFundingError {
        NotRegistered,
        NoAvailableBalance,
        InsufficientDeposit,
        UnRegisterPositiveBalance,
    }

    impl AsRef<[u8]> for StorageFundingError {
        fn as_ref(&self) -> &[u8] {
            match self {
                Self::NotRegistered => errors::ERR_ACCOUNT_NOT_REGISTERED,
                Self::NoAvailableBalance => errors::ERR_NO_AVAILABLE_BALANCE,
                Self::InsufficientDeposit => errors::ERR_ATTACHED_DEPOSIT_NOT_ENOUGH,
                Self::UnRegisterPositiveBalance => {
                    errors::ERR_FAILED_UNREGISTER_ACCOUNT_POSITIVE_BALANCE
                }
            }
        }
    }
}
