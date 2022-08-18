use crate::admin_controlled::PAUSE_DEPOSIT;
use crate::connector::Connector;
use crate::deposit_event::{DepositedEvent, TokenMessageData};
use crate::proof::Proof;
use crate::{log, AdminControlled, PausedMask};
use aurora_engine_types::types::{Address, Fee, NEP141Wei};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::panic_str;
use near_sdk::json_types::Base64VecU8;
use near_sdk::{env, AccountId};

/// transfer eth-connector call args
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct TransferCallCallArgs {
    pub receiver_id: AccountId,
    pub amount: NEP141Wei,
    pub memo: Option<String>,
    pub msg: String,
}

/// Finish deposit NEAR eth-connector call args
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct FinishDepositCallArgs {
    pub new_owner_id: AccountId,
    pub amount: NEP141Wei,
    pub proof_key: String,
    pub relayer_id: AccountId,
    pub fee: Fee,
    pub msg: Option<Vec<u8>>,
}

/// Connector specific data. It always should contain `prover account` -
#[derive(BorshSerialize, BorshDeserialize)]
pub struct EthConnector {
    /// It used in the Deposit flow, to verify log entry form incoming proof.
    pub prover_account: AccountId,
    /// It is Eth address, used in the Deposit and Withdraw logic.
    pub eth_custodian_address: Address,

    // Admin controlled
    pub paused_mask: PausedMask,
}

impl AdminControlled for EthConnector {
    fn get_paused(&self) -> PausedMask {
        self.paused_mask
    }

    fn set_paused(&mut self, paused: PausedMask) {
        self.paused_mask = paused;
    }
}

impl Connector for EthConnector {
    fn withdraw(&mut self) {
        todo!()
    }

    fn deposit(&mut self, raw_proof: Base64VecU8) {
        let current_account_id = env::current_account_id();
        let predecessor_account_id = env::predecessor_account_id();
        // Check is current account owner
        let is_owner = current_account_id == predecessor_account_id;
        // Check is current flow paused. If it's owner account just skip it.
        self.assert_not_paused(PAUSE_DEPOSIT, is_owner)
            .unwrap_or_else(|_| env::panic_str("PausedError"));

        env::log_str("[Deposit tokens]");
        let proof: Proof = Proof::try_from_slice(Vec::from(raw_proof).as_slice()).unwrap();

        // Fetch event data from Proof
        let event = DepositedEvent::from_log_entry_data(&proof.log_entry_data).unwrap();

        log!(format!(
            "Deposit started: from {} to recipient {:?} with amount: {:?} and fee {:?}",
            event.sender.encode(),
            event.token_message_data.get_recipient(),
            event.amount,
            event.fee
        ));

        log!(&format!(
            "Event's address {}, custodian address {}",
            event.eth_custodian_address.encode(),
            self.eth_custodian_address.encode(),
        ));

        if event.eth_custodian_address != self.eth_custodian_address {
            panic_str("CustodianAddressMismatch");
        }

        if NEP141Wei::new(event.fee.as_u128()) >= event.amount {
            panic_str("InsufficientAmountForFee");
        }

        // Finalize deposit
        let _data = match event.token_message_data {
            // Deposit to NEAR accounts
            TokenMessageData::Near(account_id) => FinishDepositCallArgs {
                new_owner_id: account_id,
                amount: event.amount,
                proof_key: proof.get_key(),
                relayer_id: predecessor_account_id,
                fee: event.fee,
                msg: None,
            },
            // Deposit to Eth accounts
            // fee is being minted in the `ft_on_transfer` callback method
            TokenMessageData::Eth {
                receiver_id,
                message,
            } => {
                // Transfer to self and then transfer ETH in `ft_on_transfer`
                // address - is NEAR account
                let transfer_data = TransferCallCallArgs {
                    receiver_id,
                    amount: event.amount,
                    memo: None,
                    msg: message.encode(),
                }
                .try_to_vec()
                .unwrap();

                // Send to self - current account id
                FinishDepositCallArgs {
                    new_owner_id: current_account_id,
                    amount: event.amount,
                    proof_key: proof.get_key(),
                    relayer_id: predecessor_account_id,
                    fee: event.fee,
                    msg: Some(transfer_data),
                }
            }
        };
    }

    fn finish_deposit(&mut self) {
        todo!()
    }
}
