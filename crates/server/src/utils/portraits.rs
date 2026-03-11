use shared::ProfessionEnum;
use sqlx::{PgPool, Row};

use crate::units::PortraitGenerator;

/// Fix avatar URLs that don't match the unit's current profession.
/// Runs at server startup, similar to fix_chunk_assignments.
pub async fn fix_avatar_urls(pool: &PgPool) {
    let rows = match sqlx::query(
        r#"
        SELECT id, gender, portrait_variant_id, profession_id, avatar_url
        FROM units.units
        WHERE portrait_variant_id IS NOT NULL
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to load units for avatar fix: {}", e);
            return;
        }
    };

    let mut fixed = 0;

    for row in &rows {
        let unit_id: i64 = row.get("id");
        let gender: String = row.get("gender");
        let variant_id: String = row.get("portrait_variant_id");
        let profession_id: i16 = row.get("profession_id");
        let current_url: Option<String> = row.get("avatar_url");

        let profession = ProfessionEnum::from_id(profession_id)
            .unwrap_or(ProfessionEnum::Unknown);

        let expected_url = PortraitGenerator::generate_portrait_url(
            &gender, &variant_id, profession,
        );

        if current_url.as_deref() != Some(&expected_url) {
            if let Err(e) = sqlx::query(
                "UPDATE units.units SET avatar_url = $1 WHERE id = $2",
            )
            .bind(&expected_url)
            .bind(unit_id)
            .execute(pool)
            .await
            {
                tracing::error!(
                    "Failed to fix avatar for unit {}: {}", unit_id, e
                );
            } else {
                tracing::info!(
                    "Fixed avatar for unit {} ({:?}): {} → {}",
                    unit_id, profession,
                    current_url.as_deref().unwrap_or("null"),
                    expected_url
                );
                fixed += 1;
            }
        }
    }

    if fixed > 0 {
        tracing::info!("✓ Fixed {} unit avatar URL(s)", fixed);
    } else {
        tracing::debug!("All unit avatars are consistent");
    }
}