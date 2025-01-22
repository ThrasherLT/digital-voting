//! A simple batcher that batches items based on a maximum size and a time interval.
//! There's are two batcher crates on crates.io, but one of them is completely undocumented and thus sketchy
//! and the other one seems way focused on batching complex operations rather than just data.

use std::{sync::Arc, time::Duration};

use chrono::Utc;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Notify,
};

/// The Batcher instance itself holding the context for batching and the batched items.
/// The intended way to add items to the batcher is through the Sender returned by the new function.
/// This is done so that a single Batcher instance shouldn't be shared between multiple threads.
pub struct Batcher<T> {
    /// The maximum size of the batch. More items can be added, but the batch will return
    /// only this amount of items at a time, returning the rest in the next batch.
    /// So no items are lost, but the batch will always return the same maximum amount of items.
    batch_size: usize,
    /// Time interval for which the batcher will wait before returning the batch if a
    /// full batch had not yet been accumulated.
    batch_time_interval: Duration,
    /// Variable to track when the next batch should be returned.
    next_batch_time: chrono::DateTime<chrono::Utc>,
    /// Notify instance to notify the batcher that the batch is full and ready to be
    /// returned without waiting for the `batch_time_interval`.
    batch_ready_notify: Arc<Notify>,
    /// Receiver into which the items will be sent to be batched.
    rx: Receiver<T>,
    /// Buffer for the actual batched items.
    batch: Vec<T>,
}

impl<T> Batcher<T> {
    /// Create a new Batcher instance with the given batch size and time interval.
    /// Also returns a Sender through which the application can send items to the batcher.
    ///
    /// # Arguments
    /// - `batch_size` - The maximum size of the batch. More items can be added, but the batch will return.
    /// - `batch_time_interval` - Time interval for which the batcher will wait before returning the batch.
    ///
    /// # Returns
    /// A tuple containing the Batcher instance and a Sender through which the application can send items to the batcher.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use tokio::sync::mpsc::Sender;
    /// use digital_voting::batcher::Batcher;
    ///
    /// let (mut batcher, tx): (Batcher<u32>, Sender<u32>) = Batcher::new(3, Duration::from_secs(1));
    /// ```
    #[must_use]
    pub fn new(batch_size: usize, batch_time_interval: Duration) -> (Self, Sender<T>) {
        let (tx, rx) = mpsc::channel(5);
        let now = Utc::now();
        let batch_ready_notify = Arc::new(Notify::new());
        (
            Self {
                batch_size,
                batch_time_interval,
                next_batch_time: now + batch_time_interval,
                batch_ready_notify,
                rx,
                batch: Vec::new(),
            },
            tx,
        )
    }

    /// Wait for the batch to be full or batch time interval to end and return the batch.
    ///
    /// # Returns
    /// The batched items.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use digital_voting::batcher::Batcher;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (mut batcher, tx) = Batcher::<u32>::new(3, Duration::from_secs(1));
    ///     tx.send(1).await.unwrap();
    ///     tx.send(2).await.unwrap();
    ///     tx.send(3).await.unwrap();
    ///     let batch = batcher.wait_for_batch().await;
    ///     println!("{:?}", batch);
    /// }
    /// ```
    pub async fn wait_for_batch(&mut self) -> Vec<T> {
        loop {
            if self.batch.len() >= self.batch_size {
                // The batch is already full, so just returning.
                return self.flush();
            }
            let time_remaining = self.next_batch_time - Utc::now();
            match time_remaining.to_std() {
                Err(_) => {
                    // Seems like the time for time to return a batch had already passed.
                    return self.flush();
                }
                Ok(time_remaining) => {
                    tokio::select! {
                        () = tokio::time::sleep(time_remaining) => {
                            return self.flush();
                        }
                        () = self.batch_ready_notify.notified() => {
                            return self.flush();
                        }
                        // TODO might need to wait for tx permit here to prevent deadlock.
                        item = self.rx.recv() => {
                            match item {
                                Some(item) => {
                                    // Since we only got an item and it's not necessarily time to return the batch,
                                    // we're looping again to check if it's time to return the batch and continue waiting.
                                    self.batch.push(item);
                                }
                                // Channel is closed, so we're just returning the last batch.
                                // The application should handle dripping this sturct then.
                                None => {
                                    // TODO handle graceful shutdown here.
                                    return self.flush();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Return batched items without waiting.
    ///
    /// # Returns
    /// The batched items.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use digital_voting::batcher::Batcher;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (mut batcher, tx) = Batcher::<u32>::new(3, Duration::from_secs(1));
    ///     tx.send(1).await.unwrap();
    ///     tx.send(2).await.unwrap();
    ///     tx.send(3).await.unwrap();
    ///     let batch = batcher.flush();
    ///     println!("{:?}", batch);
    /// }
    /// ```
    pub fn flush(&mut self) -> Vec<T> {
        self.next_batch_time = Utc::now() + self.batch_time_interval;
        let batch_size = if self.batch.len() < self.batch_size {
            self.batch.len()
        } else {
            self.batch_size
        };
        // Using drain in case there are more than the maximum amount of items in the vector.
        // Not sure how fast this is due to extra allocations.
        self.batch.drain(0..batch_size).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // TODO Split into multiple tests.
    // TODO Maybe advance time manually instead of waiting so the test is faster?
    #[tokio::test]
    async fn test_batcher() {
        let (mut batcher, tx) = Batcher::<u32>::new(3, Duration::from_secs(1));

        let batch = batcher.wait_for_batch().await;
        assert_eq!(batch.len(), 0);

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        tx.send(3).await.unwrap();
        tx.send(4).await.unwrap();
        tx.send(5).await.unwrap();

        let batch = batcher.wait_for_batch().await;
        assert_eq!(batch, vec![1, 2, 3]);

        let batch = batcher.wait_for_batch().await;
        assert_eq!(batch, vec![4, 5]);

        let batch = batcher.wait_for_batch().await;
        assert_eq!(batch.len(), 0);

        // Failure here would indicate that the notification upon full mechanism isn't working.
        tokio::time::pause();
        tx.send(6).await.unwrap();
        tx.send(7).await.unwrap();
        tx.send(8).await.unwrap();
        let batch = batcher.wait_for_batch().await;
        assert_eq!(batch, vec![6, 7, 8]);
    }
}
