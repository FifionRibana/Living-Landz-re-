use sqlx::{PgPool, Row};
use shared::{
    grid::GridCell, CreateOrganizationRequest, DiplomaticRelation, DiplomaticRelationType,
    FullOrganizationData, MembershipStatus, OrganizationBuilding, OrganizationData,
    OrganizationMember, OrganizationOfficer, OrganizationSummary, OrganizationType,
    OrganizationTreasuryItem, RoleType,
};

/// Database handler for organizations
pub struct OrganizationsTable {
    pool: PgPool,
}

impl OrganizationsTable {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // CREATE
    // ========================================================================

    /// Create a new organization
    pub async fn create_organization(
        &self,
        request: CreateOrganizationRequest,
    ) -> Result<u64, String> {
        let org_type_id = request.organization_type.to_id();

        let org_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO organizations.organizations
            (name, organization_type_id, parent_organization_id,
             headquarters_cell_q, headquarters_cell_r,
             leader_unit_id, emblem_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(&request.name)
        .bind(org_type_id)
        .bind(request.parent_organization_id.map(|id| id as i64))
        .bind(request.headquarters_cell.as_ref().map(|c| c.q))
        .bind(request.headquarters_cell.as_ref().map(|c| c.r))
        .bind(request.founder_unit_id as i64)
        .bind(None::<String>) // emblem_url
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create organization: {}", e))?;

        // Add founder as first member
        self.add_member(org_id as u64, request.founder_unit_id, None)
            .await?;

        Ok(org_id as u64)
    }

    /// Load organization data
    pub async fn load_organization(&self, organization_id: u64) -> Result<OrganizationData, String> {
        let row = sqlx::query(
            r#"
            SELECT id, name, organization_type_id, parent_organization_id,
                   headquarters_cell_q, headquarters_cell_r,
                   total_area_km2, treasury_gold, leader_unit_id, emblem_url, population,
                   EXTRACT(EPOCH FROM created_at)::BIGINT as created_at,
                   EXTRACT(EPOCH FROM updated_at)::BIGINT as updated_at
            FROM organizations.organizations
            WHERE id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load organization: {}", e))?;

        let headquarters_cell = if let (Some(q), Some(r)) = (
            row.try_get::<i32, _>("headquarters_cell_q").ok(),
            row.try_get::<i32, _>("headquarters_cell_r").ok(),
        ) {
            Some(GridCell { q, r })
        } else {
            None
        };

        let area_val: f64 = row.try_get("total_area_km2").unwrap_or(0.0);

        Ok(OrganizationData {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            organization_type: OrganizationType::from_id(row.get("organization_type_id")),
            parent_organization_id: row
                .try_get::<i64, _>("parent_organization_id")
                .ok()
                .map(|id| id as u64),
            headquarters_cell,
            total_area_km2: area_val as f32,
            treasury_gold: row.get("treasury_gold"),
            leader_unit_id: row
                .try_get::<i64, _>("leader_unit_id")
                .ok()
                .map(|id| id as u64),
            emblem_url: row.try_get("emblem_url").ok(),
            population: row.get("population"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Load full organization data with all relations
    pub async fn load_full_organization(
        &self,
        organization_id: u64,
    ) -> Result<FullOrganizationData, String> {
        let organization = self.load_organization(organization_id).await?;
        let officers = self.load_officers(organization_id).await?;
        let members = self.load_members(organization_id).await?;
        let territory_cells = self.load_territory_cells(organization_id).await?;
        let buildings = self.load_organization_buildings(organization_id).await?;
        let treasury_items = self.load_treasury_items(organization_id).await?;

        Ok(FullOrganizationData {
            organization,
            officers,
            members,
            territory_cells,
            buildings: buildings.iter().map(|b| b.building_id).collect(),
            treasury_items,
        })
    }

    // ========================================================================
    // OFFICERS
    // ========================================================================

    /// Add an officer to the organization
    pub async fn add_officer(
        &self,
        organization_id: u64,
        unit_id: u64,
        role: RoleType,
        appointed_by: Option<u64>,
    ) -> Result<u64, String> {
        let officer_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO organizations.officers
            (organization_id, unit_id, role_type_id, appointed_by_unit_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(organization_id as i64)
        .bind(unit_id as i64)
        .bind(role.to_id())
        .bind(appointed_by.map(|id| id as i64))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to add officer: {}", e))?;

        Ok(officer_id as u64)
    }

    /// Load all officers for an organization
    pub async fn load_officers(
        &self,
        organization_id: u64,
    ) -> Result<Vec<OrganizationOfficer>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, unit_id, role_type_id,
                   EXTRACT(EPOCH FROM appointed_at)::BIGINT as appointed_at,
                   appointed_by_unit_id
            FROM organizations.officers
            WHERE organization_id = $1
            ORDER BY role_type_id ASC
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load officers: {}", e))?;

        let mut officers = Vec::new();
        for row in rows {
            officers.push(OrganizationOfficer {
                id: row.get::<i64, _>("id") as u64,
                organization_id: row.get::<i64, _>("organization_id") as u64,
                unit_id: row.get::<i64, _>("unit_id") as u64,
                role: RoleType::from_id(row.get("role_type_id")),
                appointed_at: row.get("appointed_at"),
                appointed_by_unit_id: row
                    .try_get::<i64, _>("appointed_by_unit_id")
                    .ok()
                    .map(|id| id as u64),
            });
        }

        Ok(officers)
    }

    /// Remove an officer
    pub async fn remove_officer(&self, officer_id: u64) -> Result<(), String> {
        sqlx::query("DELETE FROM organizations.officers WHERE id = $1")
            .bind(officer_id as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove officer: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // MEMBERS
    // ========================================================================

    /// Add a member to the organization
    pub async fn add_member(
        &self,
        organization_id: u64,
        unit_id: u64,
        invited_by: Option<u64>,
    ) -> Result<u64, String> {
        let member_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO organizations.members
            (organization_id, unit_id, invited_by_unit_id, membership_status)
            VALUES ($1, $2, $3, 'active')
            RETURNING id
            "#,
        )
        .bind(organization_id as i64)
        .bind(unit_id as i64)
        .bind(invited_by.map(|id| id as i64))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to add member: {}", e))?;

        Ok(member_id as u64)
    }

    /// Load all members for an organization
    pub async fn load_members(
        &self,
        organization_id: u64,
    ) -> Result<Vec<OrganizationMember>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, unit_id,
                   EXTRACT(EPOCH FROM joined_at)::BIGINT as joined_at,
                   invited_by_unit_id, membership_status
            FROM organizations.members
            WHERE organization_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load members: {}", e))?;

        let mut members = Vec::new();
        for row in rows {
            members.push(OrganizationMember {
                id: row.get::<i64, _>("id") as u64,
                organization_id: row.get::<i64, _>("organization_id") as u64,
                unit_id: row.get::<i64, _>("unit_id") as u64,
                joined_at: row.get("joined_at"),
                invited_by_unit_id: row
                    .try_get::<i64, _>("invited_by_unit_id")
                    .ok()
                    .map(|id| id as u64),
                membership_status: MembershipStatus::from_string(&row.get::<String, _>("membership_status")),
            });
        }

        Ok(members)
    }

    /// Update member status
    pub async fn update_member_status(
        &self,
        member_id: u64,
        status: MembershipStatus,
    ) -> Result<(), String> {
        sqlx::query("UPDATE organizations.members SET membership_status = $1 WHERE id = $2")
            .bind(status.to_string())
            .bind(member_id as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update member status: {}", e))?;

        Ok(())
    }

    /// Remove a member
    pub async fn remove_member(&self, member_id: u64) -> Result<(), String> {
        sqlx::query("DELETE FROM organizations.members WHERE id = $1")
            .bind(member_id as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove member: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // TERRITORY
    // ========================================================================

    /// Add a territory cell to the organization
    pub async fn add_territory_cell(
        &self,
        organization_id: u64,
        cell: &GridCell,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO organizations.territory_cells
            (organization_id, cell_q, cell_r)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(organization_id as i64)
        .bind(cell.q)
        .bind(cell.r)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add territory cell: {}", e))?;

        Ok(())
    }

    /// Load all territory cells for an organization
    pub async fn load_territory_cells(
        &self,
        organization_id: u64,
    ) -> Result<Vec<GridCell>, String> {
        let rows = sqlx::query(
            r#"
            SELECT cell_q, cell_r
            FROM organizations.territory_cells
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load territory cells: {}", e))?;

        let mut cells = Vec::new();
        for row in rows {
            cells.push(GridCell {
                q: row.get("cell_q"),
                r: row.get("cell_r"),
            });
        }

        Ok(cells)
    }

    /// Remove a territory cell
    pub async fn remove_territory_cell(
        &self,
        organization_id: u64,
        cell: &GridCell,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM organizations.territory_cells
            WHERE organization_id = $1 AND cell_q = $2 AND cell_r = $3
            "#,
        )
        .bind(organization_id as i64)
        .bind(cell.q)
        .bind(cell.r)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove territory cell: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // BUILDINGS
    // ========================================================================

    /// Add a building to the organization
    pub async fn add_building(
        &self,
        organization_id: u64,
        building_id: u64,
        building_role: Option<String>,
        acquired_by: Option<u64>,
    ) -> Result<u64, String> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO organizations.buildings
            (organization_id, building_id, building_role, acquired_by_unit_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(organization_id as i64)
        .bind(building_id as i64)
        .bind(building_role)
        .bind(acquired_by.map(|id| id as i64))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to add building: {}", e))?;

        Ok(id as u64)
    }

    /// Load all buildings for an organization
    pub async fn load_organization_buildings(
        &self,
        organization_id: u64,
    ) -> Result<Vec<OrganizationBuilding>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, building_role,
                   EXTRACT(EPOCH FROM acquired_at)::BIGINT as acquired_at,
                   acquired_by_unit_id
            FROM organizations.buildings
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load buildings: {}", e))?;

        let mut buildings = Vec::new();
        for row in rows {
            buildings.push(OrganizationBuilding {
                id: row.get::<i64, _>("id") as u64,
                organization_id: row.get::<i64, _>("organization_id") as u64,
                building_id: row.get::<i64, _>("building_id") as u64,
                acquired_at: row.get("acquired_at"),
                acquired_by_unit_id: row
                    .try_get::<i64, _>("acquired_by_unit_id")
                    .ok()
                    .map(|id| id as u64),
                building_role: row.try_get("building_role").ok(),
            });
        }

        Ok(buildings)
    }

    /// Remove a building from organization
    pub async fn remove_building(&self, building_record_id: u64) -> Result<(), String> {
        sqlx::query("DELETE FROM organizations.buildings WHERE id = $1")
            .bind(building_record_id as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove building: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // TREASURY
    // ========================================================================

    /// Update organization treasury gold
    pub async fn update_treasury_gold(
        &self,
        organization_id: u64,
        amount: i32,
    ) -> Result<(), String> {
        sqlx::query("UPDATE organizations.organizations SET treasury_gold = $1 WHERE id = $2")
            .bind(amount)
            .bind(organization_id as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update treasury: {}", e))?;

        Ok(())
    }

    /// Add item to treasury
    pub async fn add_treasury_item(
        &self,
        organization_id: u64,
        item_instance_id: u64,
        quantity: i32,
    ) -> Result<u64, String> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO organizations.treasury_items
            (organization_id, item_instance_id, quantity)
            VALUES ($1, $2, $3)
            ON CONFLICT (organization_id, item_instance_id)
            DO UPDATE SET quantity = organizations.treasury_items.quantity + $3
            RETURNING id
            "#,
        )
        .bind(organization_id as i64)
        .bind(item_instance_id as i64)
        .bind(quantity)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to add treasury item: {}", e))?;

        Ok(id as u64)
    }

    /// Load treasury items
    pub async fn load_treasury_items(
        &self,
        organization_id: u64,
    ) -> Result<Vec<OrganizationTreasuryItem>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, item_instance_id, quantity,
                   EXTRACT(EPOCH FROM stored_at)::BIGINT as stored_at
            FROM organizations.treasury_items
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load treasury items: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(OrganizationTreasuryItem {
                id: row.get::<i64, _>("id") as u64,
                organization_id: row.get::<i64, _>("organization_id") as u64,
                item_instance_id: row.get::<i64, _>("item_instance_id") as u64,
                quantity: row.get("quantity"),
                stored_at: row.get("stored_at"),
            });
        }

        Ok(items)
    }

    // ========================================================================
    // DIPLOMATIC RELATIONS
    // ========================================================================

    /// Create a diplomatic relation
    pub async fn create_diplomatic_relation(
        &self,
        organization_id: u64,
        target_organization_id: u64,
        relation_type: DiplomaticRelationType,
        established_by: Option<u64>,
        expires_at: Option<i64>,
    ) -> Result<u64, String> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO organizations.diplomatic_relations
            (organization_id, target_organization_id, relation_type, established_by_unit_id, expires_at)
            VALUES ($1, $2, $3, $4, to_timestamp($5))
            RETURNING id
            "#,
        )
        .bind(organization_id as i64)
        .bind(target_organization_id as i64)
        .bind(relation_type.to_string())
        .bind(established_by.map(|id| id as i64))
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create diplomatic relation: {}", e))?;

        Ok(id as u64)
    }

    /// Load diplomatic relations for an organization
    pub async fn load_diplomatic_relations(
        &self,
        organization_id: u64,
    ) -> Result<Vec<DiplomaticRelation>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, target_organization_id, relation_type,
                   EXTRACT(EPOCH FROM established_at)::BIGINT as established_at,
                   established_by_unit_id,
                   EXTRACT(EPOCH FROM expires_at)::BIGINT as expires_at
            FROM organizations.diplomatic_relations
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load diplomatic relations: {}", e))?;

        let mut relations = Vec::new();
        for row in rows {
            relations.push(DiplomaticRelation {
                id: row.get::<i64, _>("id") as u64,
                organization_id: row.get::<i64, _>("organization_id") as u64,
                target_organization_id: row.get::<i64, _>("target_organization_id") as u64,
                relation_type: DiplomaticRelationType::from_string(&row.get::<String, _>("relation_type")),
                established_at: row.get("established_at"),
                established_by_unit_id: row
                    .try_get::<i64, _>("established_by_unit_id")
                    .ok()
                    .map(|id| id as u64),
                expires_at: row.try_get("expires_at").ok(),
            });
        }

        Ok(relations)
    }

    // ========================================================================
    // QUERIES
    // ========================================================================

    /// Get all organizations of a specific type
    pub async fn get_organizations_by_type(
        &self,
        org_type: OrganizationType,
    ) -> Result<Vec<OrganizationSummary>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, organization_type_id, leader_unit_id, population, emblem_url
            FROM organizations.organizations
            WHERE organization_type_id = $1
            ORDER BY population DESC
            "#,
        )
        .bind(org_type.to_id())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get organizations by type: {}", e))?;

        let mut orgs = Vec::new();
        for row in rows {
            orgs.push(OrganizationSummary {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                organization_type: OrganizationType::from_id(row.get("organization_type_id")),
                leader_unit_id: row
                    .try_get::<i64, _>("leader_unit_id")
                    .ok()
                    .map(|id| id as u64),
                population: row.get("population"),
                emblem_url: row.try_get("emblem_url").ok(),
            });
        }

        Ok(orgs)
    }

    /// Get vassals of an organization
    pub async fn get_vassals(&self, organization_id: u64) -> Result<Vec<u64>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id FROM organizations.organizations
            WHERE parent_organization_id = $1
            "#,
        )
        .bind(organization_id as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get vassals: {}", e))?;

        Ok(rows.iter().map(|row| row.get::<i64, _>("id") as u64).collect())
    }
}
