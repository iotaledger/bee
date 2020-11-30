// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::wait_priority_queue::WaitPriorityQueue;

use std::cmp::Ordering;

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct TestMinHeapEntry(u64, char);

impl PartialOrd for TestMinHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for TestMinHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

#[tokio::test]
async fn min_heap() {
    let queue = WaitPriorityQueue::default();

    queue.push(TestMinHeapEntry(5, 'F'));
    queue.push(TestMinHeapEntry(1, 'B'));
    queue.push(TestMinHeapEntry(9, 'J'));
    queue.push(TestMinHeapEntry(0, 'A'));
    queue.push(TestMinHeapEntry(7, 'H'));
    queue.push(TestMinHeapEntry(6, 'G'));
    queue.push(TestMinHeapEntry(2, 'C'));
    queue.push(TestMinHeapEntry(3, 'D'));
    queue.push(TestMinHeapEntry(8, 'I'));
    queue.push(TestMinHeapEntry(4, 'E'));

    assert_eq!(queue.pop().await, TestMinHeapEntry(0, 'A'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(1, 'B'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(2, 'C'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(3, 'D'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(4, 'E'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(5, 'F'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(6, 'G'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(7, 'H'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(8, 'I'));
    assert_eq!(queue.pop().await, TestMinHeapEntry(9, 'J'));
}
