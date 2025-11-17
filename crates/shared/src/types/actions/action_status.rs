use bincode::{Decode, Encode};
use sqlx::Type;


#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Type, Copy)]
#[sqlx(type_name = "command_status_enum")]
pub enum ActionStatus {
    InProgress,
    Pending,
    Completed,
    Failed,
}
