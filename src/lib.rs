use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde_json::json;
use near_sdk::json_types::Base58PublicKey;
use near_sdk::{
    env, near_bindgen, AccountId, PanicOnDefault, Promise, PublicKey, Gas,
    assert_one_yocto, Balance,
};

///transfer call -> promise accept function, to add key in this contract
mod receiver;

near_sdk::setup_alloc!();

/// Access key allowance for linkdrop keys.
const ACCESS_KEY_ALLOWANCE: u128 = 500_000_000_000_000_000_000_000;

const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_NFT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;

///1 Near
const STORAGE_AMOUNT: u128 = 1_000_000_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NFT {
    contract_id: AccountId,
    token_id: TokenId,
    owner_id: AccountId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    accounts: LookupMap<PublicKey, NFT>,
    drop_deposits: LookupMap<AccountId, Balance>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: LookupMap::new(b"a".to_vec()),
            drop_deposits: LookupMap::new(b"d".to_vec()),
        }
    }

    ///pay the storage money = NFT number * 0.5Near
    #[payable]
    pub fn drop_deposit(&mut self, total_drop: u128) {
        let sender_id = env::predecessor_account_id();

        let total_deposit = total_drop * ACCESS_KEY_ALLOWANCE;

        assert_eq!(
            env::attached_deposit(),
            STORAGE_AMOUNT + total_deposit,
            "Attach deposit not enough"
        );

        self.drop_deposits.insert(&sender_id, &total_deposit);
    }

    ///return Unspent money
    #[payable]
    pub fn drop_withdraw(&mut self) -> bool {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let deposit = self.drop_deposits.get(&sender_id).expect("Drop deposit not found");

        let amount = self.drop_deposits.remove(&sender_id).unwrap_or_default();
        if amount == 0 {
            Promise::new(sender_id).transfer((deposit + STORAGE_AMOUNT).into());
            true
        } else {
            false
        }
    }

    /// Allows given public key to claim sent NFT.
    /// Takes ACCESS_KEY_ALLOWANCE as fee from deposit to cover account creation via an access key.
    #[payable]
     fn send_nft(&mut self, public_key: Base58PublicKey, nft: NFT) -> Promise {
        let deposit = self.drop_deposits.get(&nft.owner_id).expect("Drop deposit not found");
        assert!(
            deposit >= ACCESS_KEY_ALLOWANCE,
            "Drop deposit must be greater than ACCESS_KEY_ALLOWANCE"
        );
        self.drop_deposits.insert(&nft.owner_id, &(deposit - ACCESS_KEY_ALLOWANCE));
        let pk = public_key.into();
        self.accounts.insert(
            &pk,
            &nft,
        );
        Promise::new(env::current_account_id()).add_access_key(
            pk,
            ACCESS_KEY_ALLOWANCE,
            env::current_account_id(),
            b"claim_nft".to_vec(),
        )
    }

    /// Claim NFT for specific account that are attached to the public key this tx is signed with.
    pub fn claim_nft(&mut self, account_id: AccountId) -> Promise {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Claim only can come from this account"
        );
        assert!(
            env::is_valid_account_id(account_id.as_bytes()),
            "Invalid account id"
        );
        let nft = self
            .accounts
            .remove(&env::signer_account_pk())
            .expect("Unexpected public key");
        Promise::new(env::current_account_id()).delete_key(env::signer_account_pk());
        Promise::new(nft.contract_id)
            .function_call(
                b"nft_transfer".to_vec(),
                json!({
                    "receiver_id": account_id,
                    "token_id": nft.token_id,
                    "approval_id": None::<u64>,
                    "memo": "",
                })
                .to_string()
                .as_bytes()
                .to_vec(),
                1,
                GAS_FOR_NFT_TRANSFER_CALL,
            )
    }

    ///Return deposit balance
    pub fn get_deposit(&self,owner_id:AccountId) -> Balance {
        let deposit = self.drop_deposits.get(&owner_id).expect("Drop deposit not found");
        deposit
    }
}
