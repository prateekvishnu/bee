// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_common_pt2::node::ResHandle;
use bee_protocol::{event::LatestMilestoneChanged, tangle::MsTangle};
use bee_storage::storage::Backend;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SyncStatusResponse {
    lmi: u32,
    lsmi: u32, // Shouldn't it be smi (solid milestone index) instead?
}

pub(crate) fn forward<B: Backend>(
    latest_milestone: LatestMilestoneChanged,
    tangle: &ResHandle<MsTangle<B>>,
) -> WsEvent {
    WsEvent::new(
        WsTopic::SyncStatus,
        WsEventInner::SyncStatus(SyncStatusResponse {
            lmi: *latest_milestone.index,
            lsmi: *tangle.get_latest_solid_milestone_index(),
        }),
    )
}