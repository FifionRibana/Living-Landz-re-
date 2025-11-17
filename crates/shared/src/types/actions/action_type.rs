use bincode::{Decode, Encode};
use sqlx::Type;


#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Type, Copy)]
#[sqlx(type_name = "command_type_enum")]
pub enum ActionType {
    Unknown,
    BuildBuilding,
    BuildRoad,
    MoveUnit,
    SendMessage,
    HarvestResource,
    CraftResource,
}
