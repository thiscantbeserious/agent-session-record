//! Background worker for async cache loading
//!
//! Processes load requests on a dedicated thread and sends results
//! back via channels. The worker is generic over key and value types.

use std::sync::mpsc::{Receiver, Sender};

/// Result of a background load operation
pub struct LoadResult<K, V> {
    /// The key that was requested
    pub key: K,
    /// The loaded value, or None if loading failed
    pub value: Option<V>,
}

/// Background worker loop that processes load requests.
///
/// Receives keys from `request_rx`, calls `loader` for each key,
/// and sends `LoadResult` back via `result_tx`. Exits when the
/// request channel is closed (all senders dropped).
pub fn worker_loop<K, V>(
    request_rx: Receiver<K>,
    result_tx: Sender<LoadResult<K, V>>,
    loader: impl Fn(&K) -> Option<V>,
) {
    while let Ok(key) = request_rx.recv() {
        let value = loader(&key);
        // Ignore send errors (main thread may have exited)
        let _ = result_tx.send(LoadResult { key, value });
    }
}
