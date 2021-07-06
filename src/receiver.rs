use crate::*;
use near_sdk::{AccountId, PromiseOrValue};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_sdk::log;
/// transfer callbacks from NFT Contracts
#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    #[payable]
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>{
        let contract_id: AccountId = env::predecessor_account_id();
        let owner_id = sender_id;
        let public_key: Base58PublicKey = near_sdk::serde_json::from_str(&msg).expect("Valid send args");
        let nft = NFT {
            contract_id,
            owner_id,
            token_id,
        };
        self.send_nft(
            public_key,
            nft,
        );
        PromiseOrValue::Value(false)
    }
}
