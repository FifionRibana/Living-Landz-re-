use bevy::ecs::component::Component;


// Cell view components (detailed cell view mode)
#[derive(Component)]
pub struct CellViewPanel;

#[derive(Component)]
pub struct ManagementPanel;

#[derive(Component)]
pub struct MessagesPanel;

#[derive(Component)]
pub struct RankingPanel;

#[derive(Component)]
pub struct RecordsPanel;

#[derive(Component)]
pub struct CalendarPanel;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct InventoryPanel;

/// Marker for inventory item rows (for interaction/updating)
#[derive(Component)]
pub struct InventoryItemRow {
    pub item_id: i32,
}
