use bevy::prelude::*;
use shared::{grid::GridCell, OrganizationSummary};

/// Resource pour stocker l'organisation de la cellule actuelle (survol/sélection)
#[derive(Resource, Default)]
pub struct CurrentOrganization {
    /// Dernière cellule pour laquelle on a demandé l'organisation
    pub last_queried_cell: Option<GridCell>,

    /// Organisation actuelle à cette cellule
    pub organization: Option<OrganizationSummary>,
}

impl CurrentOrganization {
    pub fn new() -> Self {
        Self {
            last_queried_cell: None,
            organization: None,
        }
    }

    pub fn update(&mut self, cell: GridCell, organization: Option<OrganizationSummary>) {
        self.last_queried_cell = Some(cell);
        self.organization = organization;
    }

    pub fn clear(&mut self) {
        self.last_queried_cell = None;
        self.organization = None;
    }

    pub fn has_organization(&self) -> bool {
        self.organization.is_some()
    }

    pub fn get_organization_name(&self) -> Option<&str> {
        self.organization.as_ref().map(|org| org.name.as_str())
    }
}
