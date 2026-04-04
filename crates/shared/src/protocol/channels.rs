// === Channels ===

/// RPC-style messages (actions, login data). Bidirectional, ordered reliable.
pub struct ReliableGameChannel;

/// Full exploration map + incremental patches. Server → client, unordered reliable.
pub struct ExplorationChannel;
