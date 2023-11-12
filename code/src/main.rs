mod balances;
mod proof_of_existence;
mod support;
mod system;

use crate::support::Dispatch;

// These are the concrete types we will use in our simple state machine.
// Modules are configured for these types directly, and they satisfy all of our
// trait requirements.
mod types {
	pub type AccountId = &'static str;
	pub type BlockNumber = u32;
	pub type Nonce = u32;
	pub type Balance = u128;
	pub type Extrinsic = crate::support::Extrinsic<AccountId, crate::RuntimeCall>;
	pub type Block = crate::support::Block<BlockNumber, Extrinsic>;
	pub type Content = &'static str;
}

// This is our main Runtime.
// It accumulates all of the different modules we want to use,
// functions implemented on the Runtime allow us to access those modules and execute blocks of
// transactions.
#[derive(Debug)]
#[macros::runtime]
pub struct Runtime {
	system: system::SystemModule<Self>,
	balances: balances::BalancesModule<Self>,
	proof_of_existence: proof_of_existence::POEModule<Self>,
}

impl system::Config for Runtime {
	type AccountId = types::AccountId;
	type BlockNumber = types::BlockNumber;
	type Nonce = types::Nonce;
}

impl balances::Config for Runtime {
	type Balance = types::Balance;
}

impl proof_of_existence::Config for Runtime {
	type Content = types::Content;
}

impl Runtime {
	// Create a new instance of the main Runtime, by creating a new instance of each module.
	fn new() -> Self {
		Self {
			system: system::SystemModule::new(),
			balances: balances::BalancesModule::new(),
			proof_of_existence: proof_of_existence::POEModule::new(),
		}
	}

	// Execute a block of extrinsics. Increments the block number.
	fn execute_block(&mut self, block: types::Block) -> Result<(), &'static str> {
		self.system.inc_block_number();
		if block.header.block_number != self.system.block_number() {
			return Err(&"block number does not match what is expected")
		}
		for (i, support::Extrinsic { caller, call }) in block.extrinsics.into_iter().enumerate() {
			let _res = self.dispatch(caller, call).map_err(|e| {
				eprintln!(
					"Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
					block.header.block_number, i, e
				)
			});
		}
		Ok(())
	}
}

// The main entry point for our simple state machine.
fn main() {
	// Create a new instance of the Runtime.
	// It will instantiate with it all the modules it uses.
	let mut runtime = Runtime::new();

	// Initialize the system with some initial balance.
	runtime.balances.set_balance(&"alice", 100);

	// Here are the extrinsics in our block.
	// You can add or remove these based on the modules and calls you have set up.
	let block_1 = types::Block {
		header: support::Header { block_number: 1 },
		extrinsics: vec![
			support::Extrinsic {
				caller: &"alice",
				call: RuntimeCall::balances(balances::Call::transfer { to: &"bob", amount: 20 }),
			},
			support::Extrinsic {
				caller: &"alice",
				call: RuntimeCall::balances(balances::Call::transfer {
					to: &"charlie",
					amount: 20,
				}),
			},
		],
	};

	let block_2 = types::Block {
		header: support::Header { block_number: 2 },
		extrinsics: vec![
			support::Extrinsic {
				caller: &"alice",
				call: RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim {
					claim: &"Hello, world!",
				}),
			},
			support::Extrinsic {
				caller: &"bob",
				call: RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim {
					claim: &"Hello, world!",
				}),
			},
		],
	};

	let block_3 = types::Block {
		header: support::Header { block_number: 3 },
		extrinsics: vec![
			support::Extrinsic {
				caller: &"alice",
				call: RuntimeCall::proof_of_existence(proof_of_existence::Call::revoke_claim {
					claim: &"Hello, world!",
				}),
			},
			support::Extrinsic {
				caller: &"bob",
				call: RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim {
					claim: &"Hello, world!",
				}),
			},
		],
	};

	// Execute the extrinsics which make up our block.
	// If there are any errors, our system panics, since we should not execute invalid blocks.
	runtime.execute_block(block_1).expect("invalid block");
	runtime.execute_block(block_2).expect("invalid block");
	runtime.execute_block(block_3).expect("invalid block");

	// Simply print the debug format of our runtime state.
	println!("{:#?}", runtime);
}
