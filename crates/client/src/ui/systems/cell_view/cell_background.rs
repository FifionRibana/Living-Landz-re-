pub fn update_cell_view_background(
    cell_view_state: Res<CellViewState>,
    world_cache: Res<WorldCache>,
    mut commands: Commands,
    container_query: Query<Entity, With<CellViewContainer>>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    mut last_viewed_cell: Local<Option<shared::grid::GridCell>>,
    units_cache: Res<UnitsCache>,
    units_data_cache: Res<UnitsDataCache>,
) {

}