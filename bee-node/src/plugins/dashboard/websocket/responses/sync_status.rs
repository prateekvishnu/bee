// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::workers::event::MilestoneConfirmed;
use bee_tangle::{event::LatestMilestoneChanged, Tangle};
use serde::Serialize;

use crate::{
    plugins::dashboard::websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    storage::NodeStorageBackend,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SyncStatusResponse {
    pub(crate) lmi: u32,
    pub(crate) cmi: u32,
}

pub(crate) fn forward_latest_milestone_changed<S: NodeStorageBackend>(
    latest_milestone: LatestMilestoneChanged,
    tangle: &Tangle<S>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *latest_milestone.index,
            cmi: *tangle.get_confirmed_milestone_index(),
        }),
    )
}

pub(crate) fn forward_confirmed_milestone_changed<S: NodeStorageBackend>(
    event: &MilestoneConfirmed,
    tangle: &Tangle<S>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *tangle.get_latest_milestone_index(),
            cmi: *event.index,
        }),
    )
}
