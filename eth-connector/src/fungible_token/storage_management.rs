use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
    AccountId,
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema))]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

impl Default for StorageBalance {
    fn default() -> Self {
        Self {
            total: 0.into(),
            available: 0.into(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema))]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

pub trait StorageManagement {
    // if `registration_only=true` MUST refund above the minimum balance if the account didn't exist and
    //     refund full deposit if the account exists.
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    /// Withdraw specified amount of available Ⓝ for predecessor account.
    ///
    /// This method is safe to call. It MUST NOT remove data.
    ///
    /// `amount` is sent as a string representing an unsigned 128-bit integer. If
    /// omitted, contract MUST refund full `available` balance. If `amount` exceeds
    /// predecessor account's available balance, contract MUST panic.
    ///
    /// If predecessor account not registered, contract MUST panic.
    ///
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    /// function-call access-key call (UX wallet security)
    ///
    /// Returns the StorageBalance structure showing updated balances.
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance;

    /// Unregisters the predecessor account and returns the storage NEAR deposit back.
    ///
    /// If the predecessor account is not registered, the function MUST return `false` without panic.
    ///
    /// If `force=true` the function SHOULD ignore account balances (burn them) and close the account.
    /// Otherwise, MUST panic if caller has a positive registered balance (eg token holdings) or
    ///     the contract doesn't support force unregistration.
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted function-call access-key call
    /// (UX wallet security)
    /// Returns `true` iff the account was unregistered.
    /// Returns `false` iff account was not registered before.
    fn storage_unregister(&mut self, force: Option<bool>) -> bool;

    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    fn storage_balance_of(&self, account_id: AccountId) -> StorageBalance;
}
