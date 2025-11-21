use sqlx::FromRow;

pub mod methods;

#[derive(Debug, Clone, FromRow)]
pub struct Player {
    pub id: i64,
    pub family_name: String,
    pub language_id: i16,
    pub coat_of_arms_id: Option<i64>,
    pub motto: Option<String>,
    pub origin_location: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Character {
    pub id: i64,
    pub player_id: i64,

    pub first_name: String,
    pub family_name: String,
    pub second_name: Option<String>,
    pub nickname: Option<String>,

    pub coat_of_arms_id: Option<i64>,
    pub image_id: Option<i64>,
    pub motto: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CoatOfArms {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Language {
    pub id: i16,
    pub name: String,
    pub code: String,
}
