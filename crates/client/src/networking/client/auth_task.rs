use bevy::prelude::*;
use bevy::tasks::Task;
use lightyear::netcode::ConnectToken;

/// Result of an HTTP auth request.
pub struct AuthResult {
    pub player_id: u64,
    pub connect_token: ConnectToken,
}

/// Resource that holds the in-flight auth task.
/// Set by the login/register UI, polled by `poll_auth_task` in game_client.
#[derive(Resource, Default)]
pub struct AuthTask {
    pub task: Option<Task<Result<AuthResult, String>>>,
    /// Set to true when auth succeeded and lightyear connection was initiated.
    pub completed: bool,
}
