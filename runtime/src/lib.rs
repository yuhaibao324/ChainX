// Copyright 2018 Chainpool.

//! The ChainX runtime. This can be compiled with ``#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]


#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "std")]
extern crate serde;

#[macro_use]
extern crate substrate_runtime_io as runtime_io;
#[macro_use]
extern crate substrate_runtime_support;
#[macro_use]
extern crate substrate_runtime_primitives as runtime_primitives;
extern crate substrate_codec as codec;
#[macro_use]
extern crate substrate_codec_derive;
extern crate substrate_runtime_std as rstd;
extern crate substrate_runtime_consensus as consensus;
extern crate substrate_runtime_balances as balances;
extern crate substrate_runtime_council as council;
extern crate substrate_runtime_democracy as democracy;
extern crate substrate_runtime_executive as executive;
extern crate substrate_runtime_session as session;
extern crate substrate_runtime_staking as staking;
extern crate substrate_runtime_system as system;
extern crate substrate_runtime_timestamp as timestamp;
#[macro_use]
extern crate substrate_runtime_version as version;
extern crate chainx_primitives;
#[cfg(feature = "std")]
mod checked_block;

#[cfg(feature = "std")]
pub use checked_block::CheckedBlock;
pub use balances::address::Address as RawAddress;

use rstd::prelude::*;
use chainx_primitives::{AccountId, AccountIndex, Balance, BlockNumber, Hash, Index, Log, SessionKey, Signature};

use runtime_primitives::traits::{Convert, HasPublicAux, BlakeTwo256};
use timestamp::Call as TimestampCall;
use chainx_primitives::InherentData;
use runtime_primitives::generic;
use version::RuntimeVersion;

//pub use chainx_primitives::Header;

pub fn inherent_extrinsics(data: InherentData) -> Vec<UncheckedExtrinsic> {
	let make_inherent = |function| UncheckedExtrinsic::new(
		Extrinsic {
			signed: Default::default(),
			function,
			index: 0,
		},
		Default::default(),
	);

	let mut inherent: Vec<UncheckedExtrinsic> =  Vec::new();
    inherent.push(make_inherent(Call::Timestamp(TimestampCall::set(data.timestamp))));
	inherent
}

#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
/// Concrete runtime type used to parameterize the various modules.
pub struct Concrete;


/// The position of the timestamp set extrinsic.
pub const TIMESTAMP_SET_POSITION: u32 = 0;
/// The position of the offline nodes noting extrinsic.
pub const NOTE_OFFLINE_POSITION: u32 = 2;

/// Runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: ver_str!("chainx"),
    impl_name: ver_str!("chainpool-chainx"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 0,
};

impl_outer_event! {
    pub enum Event for Concrete {
        balances,session, staking
    }
}

/// Version module for this concrete runtime.
pub type Version = version::Module<Concrete>;

impl version::Trait for Concrete {
    const VERSION: RuntimeVersion = VERSION;
}

impl HasPublicAux for Concrete {
    type PublicAux = AccountId;
}

impl system::Trait for Concrete {
    type PublicAux = <Concrete as HasPublicAux>::PublicAux;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Digest = generic::Digest<Log>;
    type AccountId = AccountId;
    type Header = Header;
    
    type Event = Event;
}

/// System module for this concrete runtime.
pub type System = system::Module<Concrete>;

impl balances::Trait for Concrete {
	type Balance = Balance;
	type AccountIndex = AccountIndex;
	type OnFreeBalanceZero = Staking;
	type EnsureAccountLiquid = Staking;
	type Event = Event;
}
/// balances module for this concrete runtime.
pub type Balances = balances::Module<Concrete>;


impl consensus::Trait for Concrete {
    const NOTE_OFFLINE_POSITION: u32 = NOTE_OFFLINE_POSITION;
	type SessionKey = SessionKey;
	type OnOfflineValidator = Staking;

}

/// Consensus module for this concrete runtime.
pub type Consensus = consensus::Module<Concrete>;

impl timestamp::Trait for Concrete {
    const TIMESTAMP_SET_POSITION: u32 = 0;

    type Moment = u64;
}

/// Timestamp module for this concrete runtime.
pub type Timestamp = timestamp::Module<Concrete>;

/// Session key conversion.
pub struct SessionKeyConversion;
impl Convert<AccountId, SessionKey> for SessionKeyConversion {
    fn convert(a: AccountId) -> SessionKey {
        a.0.into()
    }
}

impl session::Trait for Concrete {
    type ConvertAccountIdToSessionKey = SessionKeyConversion;
    type OnSessionChange = Staking;
    type Event = Event;
}

/// Session module for this concrete runtime.
pub type Session = session::Module<Concrete>;

impl staking::Trait for Concrete {
   
    type Event = Event;
}

/// Staking module for this concrete runtime.
pub type Staking = staking::Module<Concrete>;

impl democracy::Trait for Concrete {
    type Proposal = PrivCall;
}

/// Democracy module for this concrete runtime.
pub type Democracy = democracy::Module<Concrete>;

impl council::Trait for Concrete {}

/// Council module for this concrete runtime.
pub type Council = council::Module<Concrete>;
/// Council voting module for this concrete runtime.
pub type CouncilVoting = council::voting::Module<Concrete>;

impl_outer_dispatch! {
	#[derive(Clone, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
	pub enum Call where aux: <Concrete as HasPublicAux>::PublicAux {
		Consensus = 0,
        Balances = 1,
		Session = 2,
		Staking = 3,
		Timestamp = 4,
		Democracy = 5,
		Council = 6,
		CouncilVoting = 7,
	}

	#[derive(Clone, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
	pub enum PrivCall {
		Consensus = 0,
		Balances = 1,
		Session = 2,
		Staking = 3,
		Democracy = 5,
		Council = 6,
		CouncilVoting = 7,

	}
}

/// The address format for describing accounts.
//pub type Address = staking::Address<Concrete>;
pub type Address = balances::Address<Concrete>;
/// Block header type as expected by this runtime.
pub use chainx_primitives::Header;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Index, Call, Signature>;
/// Extrinsic type as expected by this runtime. This is not the type that is signed.
pub type Extrinsic = generic::Extrinsic<Address, Index, Call>;
/// Extrinsic type that is signed.
pub type BareExtrinsic = generic::Extrinsic<AccountId, Index, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<
    Concrete,
    Block,
    Balances,
    Balances,
    ((((((), Council), Democracy), Staking), Session), Timestamp),
>;


impl_outer_config! {
	pub struct GenesisConfig for Concrete {
		ConsensusConfig => consensus,
		SystemConfig => system,
        BalancesConfig => balances,
		SessionConfig => session,
		StakingConfig => staking,
		DemocracyConfig => democracy,
		CouncilConfig => council,
		TimestampConfig => timestamp,
	}
}

pub mod api {
    impl_stubs!(
		version => |()| super::Version::version(),
		authorities => |()| super::Consensus::authorities(),
		initialise_block => |header| super::Executive::initialise_block(&header),
		apply_extrinsic => |extrinsic| super::Executive::apply_extrinsic(extrinsic),
		execute_block => |block| super::Executive::execute_block(block),
		finalise_block => |()| super::Executive::finalise_block(),
        inherent_extrinsics => |inherent| super::inherent_extrinsics(inherent),
		validator_count => |()| super::Session::validator_count(),
        validators => |()| super::Session::validators(),
        timestamp => |()| super::Timestamp::get(),
		random_seed => |()| super::System::random_seed(),
		account_nonce => |account| super::System::account_nonce(&account),
		lookup_address => |address| super::Balances::lookup_address(address)
	);
}
