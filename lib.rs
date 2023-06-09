#![cfg_attr(not(feature = "std"), no_std,no_main)]

#[ink::contract]
mod erc20 {
    use ink::{storage::Mapping};
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc20 {
       total_supply: Balance,

       balances: Mapping<AccountId, Balance>,

       allowances: Mapping<(AccountId, AccountId), Balance>,

    }
    #[ink(event)]
    pub struct Transfer{
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error{
        InsufficientBalance,
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T,Error>;


    impl Erc20 {
      
      #[ink(constructor)]
      pub fn new(total_supply: Balance) -> Self{
        let mut balances = Mapping::default();
        let caller = Self::env().caller();
        balances.insert(caller, &total_supply);

        Self::env().emit_event(Transfer{
            from: None,
            to: Some(caller),
            value: total_supply,
        });
        Self{
            total_supply,
            balances,
            allowances: Default::default(),
        }
      }

      #[ink(message)]
      pub fn total_supply(&self)-> Balance{
        self.total_supply
      }
      #[ink(message)]
      pub fn balance_of(&self, owner: AccountId) -> Balance{
        self.balance_of_impl(&owner)
      }

      #[inline]
      fn balance_of_impl(&self, owner: &AccountId)-> Balance{
        self.balances.get(owner).unwrap_or_default()
      }

      #[ink(message)]
      pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance{
        self.allowance_impl(&owner, &spender)
      }

      #[inline]
      fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance{
        self.allowances.get((owner, spender)).unwrap_or_default()
      }
      
      #[ink(message)]
      pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>{
        let from = self.env().caller();
        self.transfer_from_to(&from, &to, value)
      }

      #[ink(message)]
      pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>{
        let owner = self.env().caller();
        self.allowances.insert((&owner, &spender), &value);
        self.env().emit_event(Approval{
          owner,
          spender,
          value,
        });
        Ok(())
      }

      #[ink(message)]
      pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>{
        let caller = self.env().caller();
        let allowance = self.allowance_impl(&from, &caller);
        if allowance < value{
            return Err(Error::InsufficientAllowance)
        }

        self.transfer_from_to(&from, &to, value)?;
        self.allowances.insert((&from, &caller), &(allowance-value));
        Ok(())
      }





    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let erc20 = Erc20::default();
            assert_eq!(erc20.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut erc20 = Erc20::new(false);
            assert_eq!(erc20.get(), false);
            erc20.flip();
            assert_eq!(erc20.get(), true);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = Erc20Ref::default();

            // When
            let contract_account_id = client
                .instantiate("erc20", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<Erc20Ref>(contract_account_id.clone())
                .call(|erc20| erc20.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = Erc20Ref::new(false);
            let contract_account_id = client
                .instantiate("erc20", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<Erc20Ref>(contract_account_id.clone())
                .call(|erc20| erc20.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<Erc20Ref>(contract_account_id.clone())
                .call(|erc20| erc20.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<Erc20Ref>(contract_account_id.clone())
                .call(|erc20| erc20.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
