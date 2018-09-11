// Copyright 2018 chainpool.

//! chainx-specific network implementation.
//!
//! This manages gossip of consensus messages for BFT, communication between validators
//! and more.

extern crate substrate_codec as codec;
#[macro_use]
extern crate substrate_codec_derive;
extern crate substrate_primitives;
extern crate substrate_bft as bft;
extern crate substrate_network;

extern crate chainx_primitives;
extern crate chainx_consensus;
extern crate chainx_api;

extern crate rhododendron;
extern crate ed25519;
extern crate futures;
extern crate tokio;
#[macro_use]
extern crate log;

pub mod consensus;

use substrate_network::StatusMessage as GenericFullStatus;
use substrate_network::consensus_gossip::ConsensusGossip;
use substrate_network::{NodeIndex, Context, Severity};
use substrate_network::specialization::Specialization;
use substrate_network::{message, generic_message};
use codec::Decode;

use chainx_primitives::{Block, SessionKey, Hash, Header};

use std::collections::HashMap;

/// ChainX protocol id.
pub const CHAINX_PROTOCOL_ID: substrate_network::ProtocolId = *b"pcx";

type FullStatus = GenericFullStatus<Block>;

/// Specialization of the network service for the chainx protocol.
pub type NetworkService = substrate_network::Service<Block, ChainXProtocol, Hash>;

struct PeerInfo {
    validator_key: Option<SessionKey>,
    claimed_validator: bool,
}

struct CurrentConsensus {
    parent_hash: Hash,
    local_session_key: SessionKey,
}


/// ChainX-specific messages.
#[derive(Debug, Encode, Decode)]
pub enum Message {
    /// As a validator, tell the peer your current session key.
    // TODO: do this with a cryptographic proof of some kind
    SessionKey(SessionKey),
}

/// ChainX protocol attachment for substrate.
pub struct ChainXProtocol {
    peers: HashMap<NodeIndex, PeerInfo>,
    consensus_gossip: ConsensusGossip<Block>,
    validators: HashMap<SessionKey, NodeIndex>,
    live_consensus: Option<CurrentConsensus>,
}

impl ChainXProtocol {
    /// Instantiate a chainx protocol handler.
    pub fn new() -> Self {
        ChainXProtocol {
            peers: HashMap::new(),
            consensus_gossip: ConsensusGossip::new(),
            validators: HashMap::new(),
            live_consensus: None,
        }
    }

    /// Note new consensus session.
    fn new_consensus(&mut self, _ctx: &mut Context<Block>, consensus: CurrentConsensus) {
        let old_data = self.live_consensus.as_ref().map(|c| {
            (c.parent_hash, c.local_session_key)
        });

        self.live_consensus = Some(consensus);
        self.consensus_gossip.collect_garbage(
            old_data.as_ref().map(
                |&(ref hash, _)| hash,
            ),
        );
    }

    fn on_chainx_message(
        &mut self,
        ctx: &mut Context<Block>,
        who: NodeIndex,
        _raw: Vec<u8>,
        msg: Message,
    ) {
        trace!(target: "p_net", "ChainX message from {}: {:?}", who, msg);
        match msg {
            Message::SessionKey(key) => self.on_session_key(ctx, who, key),
        }
    }

    fn on_session_key(&mut self, ctx: &mut Context<Block>, who: NodeIndex, key: SessionKey) {
        {
            let info = match self.peers.get_mut(&who) {
                Some(peer) => peer,
                None => {
                    trace!(target: "p_net", "Network inconsistency: message received from unconnected peer {}", who);
                    return;
                }
            };

            if !info.claimed_validator {
                ctx.report_peer(
                    who,
                    Severity::Bad("Session key broadcasted without setting authority role"),
                );
                return;
            }

            if let Some(old_key) = ::std::mem::replace(&mut info.validator_key, Some(key)) {
                self.validators.remove(&old_key);
            }
            self.validators.insert(key, who);
        }
    }
}

impl Specialization<Block> for ChainXProtocol {
    fn status(&self) -> Vec<u8> {
        vec![0, 0]
    }

    fn on_connect(&mut self, ctx: &mut Context<Block>, who: NodeIndex, status: FullStatus) {
        let validator = status.roles.contains(substrate_network::Roles::AUTHORITY);

        let peer_info = PeerInfo {
            validator_key: None,
            claimed_validator: validator,
        };

        self.peers.insert(who, peer_info);
        self.consensus_gossip.new_peer(ctx, who, status.roles);
    }

    fn on_disconnect(&mut self, ctx: &mut Context<Block>, who: NodeIndex) {
        if let Some(info) = self.peers.remove(&who) {
            if let Some(validator_key) = info.validator_key {
                self.validators.remove(&validator_key);
            }

            self.consensus_gossip.peer_disconnected(ctx, who);
        }
    }

    fn on_message(
        &mut self,
        ctx: &mut Context<Block>,
        who: NodeIndex,
        message: message::Message<Block>,
    ) {
        match message {
            generic_message::Message::BftMessage(msg) => {
                trace!(target: "p_net", "ChainX BFT message from {}: {:?}", who, msg);
                // TODO: check signature here? what if relevant block is unknown?
                self.consensus_gossip.on_bft_message(ctx, who, msg)
            }
            generic_message::Message::ChainSpecific(raw) => {
                match Message::decode(&mut raw.as_slice()) {
                    Some(msg) => self.on_chainx_message(ctx, who, raw, msg),
                    None => {
                        trace!(target: "p_net", "Bad message from {}", who);
                        ctx.report_peer(
                            who,
                            Severity::Bad("Invalid chainx protocol message format"),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn on_abort(&mut self) {
        self.consensus_gossip.abort();
    }

    fn maintain_peers(&mut self, _ctx: &mut Context<Block>) {
        self.consensus_gossip.collect_garbage(None);
    }

    fn on_block_imported(&mut self, _ctx: &mut Context<Block>, hash: Hash, header: &Header) {
        info!("on_block_imported number:{:?}, hash:{:?}", header.number, hash);
    }
}
