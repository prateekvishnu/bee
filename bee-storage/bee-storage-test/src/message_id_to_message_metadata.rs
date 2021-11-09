// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{MessageId, MessageMetadata};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::message::{metadata::rand_message_metadata, rand_message_id};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, MessageMetadata>
    + Fetch<MessageId, MessageMetadata>
    + for<'a> MultiFetch<'a, MessageId, MessageMetadata>
    + Insert<MessageId, MessageMetadata>
    + Delete<MessageId, MessageMetadata>
    + BatchBuilder
    + Batch<MessageId, MessageMetadata>
    + for<'a> AsIterator<'a, MessageId, MessageMetadata>
    + Truncate<MessageId, MessageMetadata>
{
}

impl<S> StorageBackend for S where
    S: backend::StorageBackend
        + Exist<MessageId, MessageMetadata>
        + Fetch<MessageId, MessageMetadata>
        + for<'a> MultiFetch<'a, MessageId, MessageMetadata>
        + Insert<MessageId, MessageMetadata>
        + Delete<MessageId, MessageMetadata>
        + BatchBuilder
        + Batch<MessageId, MessageMetadata>
        + for<'a> AsIterator<'a, MessageId, MessageMetadata>
        + Truncate<MessageId, MessageMetadata>
{
}

/// Generic access tests for the message_id_to_message_metadata table.
pub fn message_id_to_message_metadata_access<S: StorageBackend>(storage: &S) {
    let (message_id, message_metadata) = (rand_message_id(), rand_message_metadata());

    assert!(!Exist::<MessageId, MessageMetadata>::exist(storage, &message_id).unwrap());
    assert!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MessageId, MessageMetadata>::insert(storage, &message_id, &message_metadata).unwrap();

    assert!(Exist::<MessageId, MessageMetadata>::exist(storage, &message_id).unwrap());
    assert_eq!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .unwrap()
            .unwrap(),
        message_metadata
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &message_metadata));

    Delete::<MessageId, MessageMetadata>::delete(storage, &message_id).unwrap();

    assert!(!Exist::<MessageId, MessageMetadata>::exist(storage, &message_id).unwrap());
    assert!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = S::batch_begin();
    let mut message_ids = Vec::new();
    let mut message_metadatas = Vec::new();

    for _ in 0..10 {
        let (message_id, message_metadata) = (rand_message_id(), rand_message_metadata());
        Insert::<MessageId, MessageMetadata>::insert(storage, &message_id, &message_metadata).unwrap();
        Batch::<MessageId, MessageMetadata>::batch_delete(storage, &mut batch, &message_id).unwrap();
        message_ids.push(message_id);
        message_metadatas.push((message_id, None));
    }

    for _ in 0..10 {
        let (message_id, message_metadata) = (rand_message_id(), rand_message_metadata());
        Batch::<MessageId, MessageMetadata>::batch_insert(storage, &mut batch, &message_id, &message_metadata).unwrap();
        message_ids.push(message_id);
        message_metadatas.push((message_id, Some(message_metadata)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MessageId, MessageMetadata>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (message_id, message_metadata) = result.unwrap();
        assert!(message_metadatas.contains(&(message_id, Some(message_metadata))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &message_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), message_ids.len());

    for ((_, message_metadata), result) in message_metadatas.into_iter().zip(results.into_iter()) {
        assert_eq!(message_metadata, result.unwrap());
    }

    Truncate::<MessageId, MessageMetadata>::truncate(storage).unwrap();

    let mut iter = AsIterator::<MessageId, MessageMetadata>::iter(storage).unwrap();

    assert!(iter.next().is_none());

    // Test to truncate an already empty table.
    Truncate::<MessageId, MessageMetadata>::truncate(storage).unwrap();
}