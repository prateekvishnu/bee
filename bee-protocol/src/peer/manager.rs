// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::Peer;

use bee_network::PeerId;

use futures::channel::oneshot;
use log::debug;
use tokio::sync::{mpsc, RwLock, RwLockReadGuard};

use std::{collections::HashMap, sync::Arc};

pub struct PeerManager {
    // TODO private
    pub(crate) peers: RwLock<HashMap<PeerId, (Arc<Peer>, mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)>>,
    // This is needed to ensure message distribution fairness as iterating over a HashMap is random.
    // TODO private
    pub(crate) peers_keys: RwLock<Vec<PeerId>>,
}

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self {
            peers: Default::default(),
            peers_keys: Default::default(),
        }
    }

    pub(crate) async fn is_empty(&self) -> bool {
        self.peers.read().await.is_empty()
    }

    // TODO find a way to only return a ref to the peer.
    pub(crate) async fn get(
        &self,
        id: &PeerId,
    ) -> Option<impl std::ops::Deref<Target = (Arc<Peer>, mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)> + '_>
    {
        RwLockReadGuard::try_map(self.peers.read().await, |map| map.get(id)).ok()
    }

    pub(crate) async fn add(
        &self,
        peer: Arc<Peer>,
        sender: mpsc::UnboundedSender<Vec<u8>>,
        shutdown: oneshot::Sender<()>,
    ) {
        debug!("Added peer {}.", peer.id());
        self.peers_keys.write().await.push(peer.id().clone());
        self.peers
            .write()
            .await
            .insert(peer.id().clone(), (peer, sender, shutdown));
    }

    pub(crate) async fn remove(
        &self,
        id: &PeerId,
    ) -> Option<(Arc<Peer>, mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)> {
        debug!("Removed peer {}.", id);
        self.peers_keys.write().await.retain(|peer_id| peer_id != id);
        self.peers.write().await.remove(id)
    }

    // TODO bring it back
    // pub(crate) async fn for_each_peer<F: Fn(&PeerId, &Peer)>(&self, f: F) {
    //     for (id, (peer, _, _)) in self.peers.read().await.iter() {
    //         f(id, peer);
    //     }
    // }

    pub(crate) fn connected_peers(&self) -> u8 {
        // TODO impl
        0
    }

    pub(crate) fn synced_peers(&self) -> u8 {
        // TODO impl
        0
    }
}