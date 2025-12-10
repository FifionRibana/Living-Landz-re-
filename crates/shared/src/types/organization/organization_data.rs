use bincode::{Decode, Encode};
use crate::grid::GridCell;
use super::enums::{OrganizationType, RoleType, MembershipStatus, DiplomaticRelationType};

// ============================================================================
// ORGANIZATION DATA
// ============================================================================

/// Core organization data sent over network
#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationData {
    pub id: u64,
    pub name: String,
    pub organization_type: OrganizationType,

    // Hierarchy
    pub parent_organization_id: Option<u64>,

    // Territory
    pub headquarters_cell: Option<GridCell>,
    pub total_area_km2: f32,

    // Economy
    pub treasury_gold: i32,

    // Leadership
    pub leader_unit_id: Option<u64>,

    // Identity
    pub emblem_url: Option<String>,

    // Population
    pub population: i32,

    // Metadata
    pub created_at: i64,
    pub updated_at: i64,
}

/// Full organization data with all relations (for server use)
#[derive(Debug, Clone, Encode, Decode)]
pub struct FullOrganizationData {
    pub organization: OrganizationData,
    pub officers: Vec<OrganizationOfficer>,
    pub members: Vec<OrganizationMember>,
    pub territory_cells: Vec<GridCell>,
    pub buildings: Vec<u64>, // Building IDs
    pub treasury_items: Vec<OrganizationTreasuryItem>,
}

// ============================================================================
// ORGANIZATION OFFICER
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationOfficer {
    pub id: u64,
    pub organization_id: u64,
    pub unit_id: u64,
    pub role: RoleType,
    pub appointed_at: i64,
    pub appointed_by_unit_id: Option<u64>,
}

// ============================================================================
// ORGANIZATION MEMBER
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationMember {
    pub id: u64,
    pub organization_id: u64,
    pub unit_id: u64,
    pub joined_at: i64,
    pub invited_by_unit_id: Option<u64>,
    pub membership_status: MembershipStatus,
}

// ============================================================================
// ORGANIZATION BUILDING
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationBuilding {
    pub id: u64,
    pub organization_id: u64,
    pub building_id: u64,
    pub acquired_at: i64,
    pub acquired_by_unit_id: Option<u64>,
    pub building_role: Option<String>, // "headquarters", "warehouse", etc.
}

// ============================================================================
// ORGANIZATION TREASURY ITEM
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationTreasuryItem {
    pub id: u64,
    pub organization_id: u64,
    pub item_instance_id: u64,
    pub quantity: i32,
    pub stored_at: i64,
}

// ============================================================================
// DIPLOMATIC RELATION
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct DiplomaticRelation {
    pub id: u64,
    pub organization_id: u64,
    pub target_organization_id: u64,
    pub relation_type: DiplomaticRelationType,
    pub established_at: i64,
    pub established_by_unit_id: Option<u64>,
    pub expires_at: Option<i64>,
}

// ============================================================================
// CREATE ORGANIZATION REQUEST
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub organization_type: OrganizationType,
    pub headquarters_cell: Option<GridCell>,
    pub parent_organization_id: Option<u64>,
    pub founder_unit_id: u64,
}

// ============================================================================
// ORGANIZATION SUMMARY (for lists/UI)
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationSummary {
    pub id: u64,
    pub name: String,
    pub organization_type: OrganizationType,
    pub leader_unit_id: Option<u64>,
    pub population: i32,
    pub emblem_url: Option<String>,
}

// ============================================================================
// ORGANIZATION HIERARCHY INFO
// ============================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct OrganizationHierarchy {
    pub organization_id: u64,
    pub parent_id: Option<u64>,
    pub vassals: Vec<u64>, // IDs of vassal organizations
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl OrganizationData {
    /// Check if organization is territorial
    pub fn is_territorial(&self) -> bool {
        matches!(
            self.organization_type,
            OrganizationType::Hamlet
                | OrganizationType::Village
                | OrganizationType::Town
                | OrganizationType::City
                | OrganizationType::Barony
                | OrganizationType::County
                | OrganizationType::Duchy
                | OrganizationType::Kingdom
                | OrganizationType::Empire
        )
    }

    /// Check if organization can have vassals
    pub fn can_have_vassals(&self) -> bool {
        matches!(
            self.organization_type,
            OrganizationType::Town
                | OrganizationType::City
                | OrganizationType::Barony
                | OrganizationType::County
                | OrganizationType::Duchy
                | OrganizationType::Kingdom
                | OrganizationType::Empire
                | OrganizationType::Abbey
                | OrganizationType::Diocese
                | OrganizationType::Archdiocese
                | OrganizationType::Monastery
                | OrganizationType::MerchantGuild
                | OrganizationType::TradingCompany
                | OrganizationType::KnightOrder
                | OrganizationType::CraftersGuild
                | OrganizationType::ScholarsGuild
                | OrganizationType::ThievesGuild
        )
    }

    /// Check if organization requires territory
    pub fn requires_territory(&self) -> bool {
        self.is_territorial()
    }

    /// Get category string
    pub fn category_string(&self) -> String {
        self.organization_type.category().to_string()
    }
}

impl OrganizationOfficer {
    /// Get authority level of this officer
    pub fn authority_level(&self) -> i16 {
        self.role.authority_level()
    }

    /// Get role name
    pub fn role_name(&self) -> String {
        self.role.to_string()
    }
}

impl OrganizationMember {
    /// Check if member is active
    pub fn is_active(&self) -> bool {
        self.membership_status == MembershipStatus::Active
    }

    /// Check if member is suspended
    pub fn is_suspended(&self) -> bool {
        self.membership_status == MembershipStatus::Suspended
    }
}

impl DiplomaticRelation {
    /// Check if relation is hostile
    pub fn is_hostile(&self) -> bool {
        matches!(
            self.relation_type,
            DiplomaticRelationType::Hostile | DiplomaticRelationType::AtWar
        )
    }

    /// Check if relation is friendly
    pub fn is_friendly(&self) -> bool {
        matches!(
            self.relation_type,
            DiplomaticRelationType::Allied | DiplomaticRelationType::TradeAgreement
        )
    }

    /// Check if relation has expired
    pub fn is_expired(&self, current_time: i64) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_time >= expires_at
        } else {
            false
        }
    }
}

impl super::enums::OrganizationCategory {
    pub fn to_string(&self) -> String {
        match self {
            Self::Territorial => "territorial".to_string(),
            Self::Religious => "religious".to_string(),
            Self::Commercial => "commercial".to_string(),
            Self::Social => "social".to_string(),
            Self::Unknown => "unknown".to_string(),
        }
    }
}
