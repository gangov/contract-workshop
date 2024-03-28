#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(dead_code)] // TODO remove when approaching completion 🏁
#![allow(unused)] // TODO remove when approaching completion 🏁

/// Most individuals can only produce one or two of the resources, and therefore collaboration is
/// necessary for survival. Therefore we create a free market in which participants can contribute
/// resources when they have them. Later members can withdraw resources in proportion to their
/// contributions. You are not required to withdraw the same resources you contributed.
#[ink::contract]
mod resource_market {
	use ink::{codegen::EmitEvent, reflect::ContractEventBase, storage::Mapping};

	/// There are three resources needed to survive: Water, Food, and Wood.
	#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Decode, scale::Encode)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo))]
	pub enum Resource {
		Food,
		Water,
		Wood,
	}

	/// Defines the storage of your contract.
	#[ink(storage)]
	pub struct ResourceMarket {
		/// The amount of food currently available on the market
		food: u64,
		/// The amount of water currently available on the market
		water: u64,
		/// The amount of wood currently available on the market
		wood: u64,
		/// The credit that each previous contributor has in the market.
		/// This is the maximum amount of resources that they can withdraw.
		credits: Mapping<AccountId, u64>,
	}

	/// Errors that can occur upon calling this contract.
	#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
	pub enum Error {
		/// Caller does not have enough credits
		InsufficientCredits,
		/// Insufficient resources available to complete request
		InsufficientResources,
	}

	/// Type alias for the contract's `Result` type.
	pub type Result<T> = core::result::Result<T, Error>;

	pub type Event = <ResourceMarket as ContractEventBase>::Type;

	/// Emitted when resources are contributed
	#[ink(event)]
	pub struct ContributionReceived {
		/// The account which contributed the resource
		#[ink(topic)]
		sender: AccountId,
		/// How much they contributed
		amount: u64,
		/// What type of resource they contributed
		resource: Resource,
		/// The total amount of that resource now available
		total_resource_available: u64,
		/// The total amount of credits the contributing account now has available
		total_credits_available: u64,
	}

	/// Emitted when resources are withdrawn
	#[ink(event)]
	pub struct ResourceWithdrawn {
		/// The account which withdrew the resource
		#[ink(topic)]
		sender: AccountId,
		/// How much they withdrew
		amount: u64,
		/// What type of resource they withdrew
		resource: Resource,
		/// The total amount of that resource now available
		total_resource_available: u64,
		/// The total amount of credits the contributing account now has available
		total_credits_available: u64,
	}

	impl ResourceMarket {
		/// Constructor that initializes the resources values and creates a default mapping
		#[ink(constructor)]
		pub fn new(food: u64, water: u64, wood: u64) -> Self {
			ResourceMarket { food, water, wood, credits: Default::default() }
		}

		/// Contribute some of your own private resources to the market.
		/// Contributions are made one asset at a time.
		#[ink(message)]
		pub fn contribute(&mut self, amount: u64, resource: Resource) -> Result<()> {
			let caller = self.env().caller();
			match resource {
				Resource::Food => self.food += amount,
				Resource::Water => self.water += amount,
				Resource::Wood => self.wood += amount,
			}

			let mut old_balance = self.credits.get(caller).unwrap_or(0);
			self.credits.insert(caller, &(old_balance.saturating_add(amount)));

			let total_resources = match resource {
				Resource::Food => self.food,
				Resource::Water => self.water,
				Resource::Wood => self.wood,
			};

			let sender_available_credits = old_balance.saturating_add(amount);

			Self::emit_event(
				self.env(),
				Event::ContributionReceived(ContributionReceived {
					sender: caller,
					amount,
					resource,
					total_resource_available: total_resources,
					total_credits_available: sender_available_credits,
				}),
			);
			Ok(())
		}

		/// Withdraw some resources from the market into your own private reserves.
		#[ink(message)]
		pub fn withdraw(&mut self, amount: u64, resource: Resource) -> Result<()> {
			let caller = self.env().caller();

			match resource {
				Resource::Food => {
					if self.food < amount {
						return Err(Error::InsufficientResources);
					}

					let caller_credits = self.credits.get(caller).unwrap_or(0);
					if caller_credits < amount {
						return Err(Error::InsufficientCredits);
					}

					self.food = self.food.saturating_sub(amount);
					self.credits.insert(caller, &(caller_credits.saturating_sub(amount)));

					Self::emit_event(
						self.env(),
						Event::ResourceWithdrawn(ResourceWithdrawn {
							sender: caller,
							amount,
							resource,
							total_resource_available: self.food - amount,
							total_credits_available: caller_credits - amount,
						}),
					);
				},
				Resource::Water => {
					if self.water < amount {
						return Err(Error::InsufficientResources);
					}

					let caller_credits = self.credits.get(caller).unwrap_or(0);
					if caller_credits < amount {
						return Err(Error::InsufficientCredits);
					}

					self.water = self.water.saturating_sub(amount);
					self.credits.insert(caller, &(caller_credits.saturating_sub(amount)));

					Self::emit_event(
						self.env(),
						Event::ResourceWithdrawn(ResourceWithdrawn {
							sender: caller,
							amount,
							resource,
							total_resource_available: self.water - amount,
							total_credits_available: caller_credits - amount,
						}),
					);
				},
				Resource::Wood => {
					if self.wood < amount {
						return Err(Error::InsufficientResources);
					}

					let caller_credits = self.credits.get(caller).unwrap_or(0);
					if caller_credits < amount {
						return Err(Error::InsufficientCredits);
					}

					self.wood = self.wood.saturating_sub(amount);
					self.credits.insert(caller, &(caller_credits.saturating_sub(amount)));

					Self::emit_event(
						self.env(),
						Event::ResourceWithdrawn(ResourceWithdrawn {
							sender: caller,
							amount,
							resource,
							total_resource_available: self.wood - amount,
							total_credits_available: caller_credits - amount,
						}),
					);
				},
			}

			Ok(())
		}

		/// Get the amount of resource available
		#[ink(message)]
		pub fn get_resource(&self, resource: Resource) -> Result<u64> {
			match resource {
				Resource::Food => Ok(self.food),
				Resource::Water => Ok(self.water),
				Resource::Wood => Ok(self.wood),
			}
		}

		fn emit_event<EE>(emitter: EE, event: Event)
		where
			EE: EmitEvent<Self>,
		{
			emitter.emit_event(event);
		}
	}

	// Enhancement: The first iteration of this contract allow users to contribute
	// by simply calling a function with an integer parameter. Presumably there is
	// a security guard somewhere near the real-world marketplace confirming the deposits
	// are actually made. But there are no on-chain assets underlying the resource market.
	// Modify the code to interface with three real ERC20 tokens called: Water, Wood, and Food.

	// Enhancement: The resource trading logic in this contract is useful for way more
	// scenarios than our simple wood, food, water trade. Generalize the contract to
	// work with up to 5 arbitrary ERC20 tokens.

	// Enhancement: If we are trading real food, wood, and water, we have real-world incentives
	// to deposit ou excess resources. Storage is hard IRL. Water evaporates, food spoils, and wood
	// rots. And all the resources are subject to robbery. But if we are talking about virtual
	// assets, there are no such risks. And depositing funds into the market comes with an
	// opportunity cost. Design a reward system where there is a small fee on every withdrawal, and
	// that fee is paid to liquidity providers.

	#[cfg(test)]
	mod tests {
		use super::*;

		use ink::env::test::recorded_events;
		type Event = <ResourceMarket as ::ink::reflect::ContractEventBase>::Type;

		fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
			ink::env::test::default_accounts::<Environment>()
		}

		fn set_next_caller(caller: AccountId) {
			ink::env::test::set_caller::<Environment>(caller);
		}

		fn set_next_caller_with_credits(
			caller: AccountId,
			credits: u64,
			market: &mut ResourceMarket,
		) {
			ink::env::test::set_caller::<Environment>(caller);
			market.credits.insert(caller, &credits);
		}

		/// Testing the constructor
		#[ink::test]
		fn test_constructor_works() {
			let resource_market = ResourceMarket::new(10, 20, 30);
			assert_eq!(resource_market.get_resource(Resource::Food), Ok(10));
			assert_eq!(resource_market.get_resource(Resource::Water), Ok(20));
			assert_eq!(resource_market.get_resource(Resource::Wood), Ok(30));
		}

		#[ink::test]
		fn test_contributing_works() {
			let default_accounts = default_accounts();
			set_next_caller(default_accounts.alice);

			let mut resource_market = ResourceMarket::new(0, 0, 0);
			let result = resource_market.contribute(10, Resource::Water);

			assert_eq!(result, Ok(()));
			assert_eq!(resource_market.get_resource(Resource::Water), Ok(10));
			assert_eq!(resource_market.credits.get(default_accounts.alice), Some(10));
		}

		#[ink::test]
		fn test_withdrawing_works() {
			let default_accounts = default_accounts();

			let mut resource_market = ResourceMarket::new(100, 100, 100);
			set_next_caller_with_credits(default_accounts.bob, 100, &mut resource_market);

			let result = resource_market.withdraw(50, Resource::Water);
			assert_eq!(result, Ok(()));
			assert_eq!(resource_market.get_resource(Resource::Water), Ok(50));
			assert_eq!(resource_market.credits.get(default_accounts.bob), Some(50)); // contributed 100 water, took 50 water
		}

		#[ink::test]
		fn test_withdrawing_more_than_contributed_fails() {
			let default_accounts = default_accounts();
			set_next_caller(default_accounts.alice);

			let mut resource_market = ResourceMarket::new(50, 50, 50);
			let contribute_result = resource_market.contribute(10, Resource::Food);

			let last_event = recorded_events().last().unwrap();
			let decoded_event = <Event as scale::Decode>::decode(&mut &last_event.data[..])
				.expect("Failed to decode event");

			let Event::ContributionReceived(ContributionReceived {
				sender,
				amount,
				resource,
				total_credits_available,
				total_resource_available,
			}) = decoded_event
			else {
				panic!("AddressAdded event should be emitted")
			};

			assert_eq!(sender, default_accounts.alice);
			assert_eq!(amount, 10);
			assert_eq!(resource, Resource::Food);
			assert_eq!(total_credits_available, 10);
			assert_eq!(total_resource_available, 60);

			let withdraw_result = resource_market.withdraw(15, Resource::Water);

			assert_eq!(contribute_result, Ok(()));
			assert_eq!(withdraw_result, Err(Error::InsufficientCredits));
			assert_eq!(resource_market.get_resource(Resource::Food), Ok(60));
			assert_eq!(resource_market.get_resource(Resource::Water), Ok(50));
		}

		#[ink::test]
		fn test_withdrawing_resources_not_available_fails() {
			let default_accounts = default_accounts();
			set_next_caller(default_accounts.bob);

			let mut resource_market = ResourceMarket::new(0, 0, 0);
			let result = resource_market.withdraw(50, Resource::Water);
			assert_eq!(result, Err(Error::InsufficientResources));
		}

		#[ink::test]
		fn test_withdrawing_resources_contributed_by_someone_else() {
			let default_accounts = default_accounts();
			set_next_caller(default_accounts.bob);

			let mut resource_market = ResourceMarket::new(0, 0, 0);
			resource_market.contribute(100, Resource::Food);
			resource_market.contribute(50, Resource::Water);
			resource_market.contribute(150, Resource::Wood);

			assert_eq!(resource_market.get_resource(Resource::Water), Ok(50));
			assert_eq!(resource_market.get_resource(Resource::Food), Ok(100));
			assert_eq!(resource_market.get_resource(Resource::Wood), Ok(150));
			assert_eq!(resource_market.credits.get(default_accounts.bob), Some(300));

			set_next_caller_with_credits(default_accounts.alice, 500, &mut resource_market);
			for resource in [Resource::Water, Resource::Food, Resource::Wood] {
				resource_market.withdraw(10, resource);
			}

			assert_eq!(resource_market.credits.get(default_accounts.alice), Some(470)); // contributed nothing, took 30 in total
		}
	}
}
