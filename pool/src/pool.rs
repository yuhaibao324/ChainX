// Copyright 2018 Chainpool.

use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::Arc,
    result,
};


use codec::{Encode, Decode};
use chainx_runtime::{Address, UncheckedExtrinsic};
use runtime_primitives;
use runtime_primitives::{generic, traits::{Hash as HashT, BlindCheckable, BlakeTwo256}};
use substrate_primitives::{KeccakHasher, RlpCodec};
use substrate_client::{self, Client};


use extrinsic_pool::{
    Pool,
    ChainApi,
    VerifiedFor,
    ExtrinsicFor,
    scoring,
    Readiness,
    VerifiedTransaction,
    Transaction,
    Error,
    ErrorKind,
    Options,
};

use substrate_network;
use chainx_primitives::{Block, Hash, BlockId, AccountId};
/*
pub type CheckedExtrinsic =
    <UncheckedExtrinsic as Checkable<
        fn(Address)
           -> result::Result<
            AccountId,
            &'static str,
        >,
    >>::Checked;
    */
pub type Backend = substrate_client::in_mem::Backend<Block, KeccakHasher, RlpCodec>;
use chainx_executor;
pub type Executor = substrate_client::LocalCallExecutor<
    Backend,
    NativeExecutor<chainx_executor::Executor>,
>;
use substrate_executor::NativeExecutor;



#[derive(Debug, Clone)]
pub struct VerifiedExtrinsic {
    pub sender: AccountId,
    pub hash: Hash,
}

pub struct Scoring;

impl VerifiedTransaction for VerifiedExtrinsic {
    type Hash = Hash;
    type Sender = AccountId;

    fn hash(&self) -> &Self::Hash {
        &self.hash
    }

    fn sender(&self) -> &Self::Sender {
        &self.sender
    }

    fn mem_usage(&self) -> usize {
        0
    }
}

impl BlindCheckable for VerifiedExtrinsic {
	type Checked = Self;

	fn check(self) -> Result<Self, &'static str> {
		// runtime_primitives::verify_encoded_lazy(&self.signature, &self.transfer, &self.transfer.from) {
		if true {
			Ok(self)
		} else {
			Err("bad signature")
		}
	}
}

fn index(at: &BlockId, _account: AccountId) -> u64 {
	(_account[0] as u64) + number_of(at)
}

fn number_of(at: &BlockId) -> u64 {
	match at {
		generic::BlockId::Number(n) => *n as u64,
		_ => 0,
	}
}

pub struct PoolApi;
impl PoolApi {
    pub fn default() -> Self {
        PoolApi
    }
	
}

impl ChainApi for PoolApi {
    type Block = Block;
    type Hash = Hash;
    type Sender = AccountId;
    type VEx = VerifiedExtrinsic;
    type Ready = HashMap<AccountId, u64>;
    type Error = Error;
    type Score = u64;
    type Event = ();


	fn verify_transaction(&self, at: &BlockId, xt: &ExtrinsicFor<Self>) -> Result<Self::VEx, Self::Error> {
		let encoded = xt.encode();
		let uxt = UncheckedExtrinsic::decode(&mut encoded.as_slice());

		let hash = BlakeTwo256::hash(&uxt.encode());
		let xt = uxt.clone();
		Ok(VerifiedExtrinsic {
			sender: hash,	//error TODO
			hash,
		})
	}
	

    fn ready(&self) -> Self::Ready {

        HashMap::default()
    }


    fn is_ready(
        &self,
        at: &BlockId,
        nonce_cache: &mut Self::Ready,
        xt: &VerifiedFor<Self>,
    ) -> Readiness {
    
		let sender = xt.verified.sender;
		
		
		let next_index = nonce_cache.entry(sender)
			.or_insert_with(|| index(at, sender));
		// TODO
		/*
		let result = match xt.original.transfer.nonce.cmp(&next_index) {
			Ordering::Greater => Readiness::Future,
			Ordering::Equal => Readiness::Ready,
			Ordering::Less => Readiness::Stale,
		};
		*/
		
		let result = Readiness::Ready;
		// remember to increment `next_index`
		*next_index = next_index.saturating_add(1);

		result
    }

	fn compare(old: &VerifiedFor<Self>, other: &VerifiedFor<Self>) -> Ordering {
		//old.original.transfer.nonce.cmp(&other.original.transfer.nonce) TODO
		Ordering::Greater
	}

	fn choose(old: &VerifiedFor<Self>, new: &VerifiedFor<Self>) -> scoring::Choice {
		//TODO
		/*
		assert!(new.verified.sender == old.verified.sender, "Scoring::choose called with transactions from different senders");
		if old.original.transfer.nonce == new.original.transfer.nonce {
			return scoring::Choice::RejectNew;
		}
		*/
		
		scoring::Choice::InsertNew
	}


    fn update_scores(
        xts: &[Transaction<VerifiedFor<Self>>],
        scores: &mut [Self::Score],
        _change: scoring::Change<()>,
    ) {
			for i in 0..xts.len() {
				scores[i] = 100;	//_xts[i].original.transfer.amount; TODO
			}
    }

    fn should_replace(_old: &VerifiedFor<Self>, _new: &VerifiedFor<Self>) -> scoring::Choice {
        unimplemented!()
    }
}


pub struct TransactionPool {
    pub inner: Arc<Pool<PoolApi>>,
    client: Arc<Client<Backend, Executor, Block>>,
}

impl TransactionPool {
    /// Create a new transaction pool.
    pub fn new(
        options: Options,
        api: PoolApi,
        client: Arc<Client<Backend, Executor, Block>>,
    ) -> Self {
        TransactionPool {
            inner: Arc::new(Pool::new(options, api)),
            client,
        }
    }

    pub fn best_block_id(&self) -> Option<BlockId> {
        self.client
            .info()
            .map(|info| BlockId::hash(info.chain.best_hash))
            .ok()
    }
}
impl substrate_network::TransactionPool<Hash, Block> for TransactionPool {
    fn transactions(&self) -> Vec<(Hash, Vec<u8>)> {
        println!("-------------transactions-------------");
        let best_block_id = match self.best_block_id() {
            Some(id) => id,
            None => return vec![],
        };
        self.inner
            .cull_and_get_pending(&best_block_id, |pending| {
                pending
                    .map(|t| {
                        let hash = t.hash().clone();
                        let ex = t.original.clone();
                        (hash, ex)
                    })
                    .collect()
            })
            .unwrap_or_else(|_e| {
                //warn!("Error retrieving pending set: {}", e);
                vec![]
            })
    }

    fn import(&self, transaction: &Vec<u8>) -> Option<Hash> {
        println!("-------------import-------------");
        let encoded = transaction.encode();
        if let Some(uxt) = Decode::decode(&mut &encoded[..]) {
            let best_block_id = self.best_block_id()?;
            match self.inner.submit_one(&best_block_id, uxt) {
                Ok(xt) => Some(*xt.hash()),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::AlreadyImported(hash) => Some(
                            ::std::str::FromStr::from_str(&hash)
                                .map_err(|_| {})
                                .expect("Hash string is always valid"),
                        ),
                        _ => {
                            //debug!("Error adding transaction to the pool: {:?}", e);
                            None
                        }
                    }
                }
            }

        } else {
            //debug!("Error decoding transaction");
            None
        }
    }

    fn on_broadcasted(&self, propagations: HashMap<Hash, Vec<String>>) {
        println!("-------------on_broadcasted-------------");
        self.inner.on_broadcasted(propagations)
    }
}
