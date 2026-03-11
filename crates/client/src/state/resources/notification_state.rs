use bevy::prelude::*;
use std::collections::VecDeque;

/// A queued notification message to display as a toast.
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub kind: NotificationKind,
    pub spawned_at: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    Error,
    Success,
    Info,
}

/// Global notification queue. Systems push messages, UI systems pop and display.
#[derive(Resource, Default)]
pub struct NotificationState {
    pub queue: VecDeque<Notification>,
}

impl NotificationState {
    pub fn push_error(&mut self, message: impl Into<String>) {
        self.queue.push_back(Notification {
            message: message.into(),
            kind: NotificationKind::Error,
            spawned_at: 0.0,
        });
    }

    pub fn push_info(&mut self, message: impl Into<String>) {
        self.queue.push_back(Notification {
            message: message.into(),
            kind: NotificationKind::Info,
            spawned_at: 0.0,
        });
    }

    pub fn push_success(&mut self, message: impl Into<String>) {
        self.queue.push_back(Notification {
            message: message.into(),
            kind: NotificationKind::Success,
            spawned_at: 0.0,
        });
    }
}
