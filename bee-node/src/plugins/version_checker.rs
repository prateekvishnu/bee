// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::Infallible, time::Duration};

use async_trait::async_trait;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

const CHECK_INTERVAL_SEC: u64 = 3600;

#[derive(Default)]
pub struct VersionChecker {}

#[async_trait]
impl<N: Node> Worker<N> for VersionChecker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(CHECK_INTERVAL_SEC))),
            );

            while ticker.next().await.is_some() {
                // TODO
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
