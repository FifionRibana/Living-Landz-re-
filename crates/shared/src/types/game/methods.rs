use sqlx::PgPool;
use super::{Player, Character};

// Récupérer un joueur par son nom de famille
pub async fn get_player_by_family_name(
    pool: &PgPool,
    family_name: &str,
) -> Result<Option<Player>, sqlx::Error> {
    sqlx::query_as::<_, Player>(
        "SELECT * FROM game.players WHERE family_name = $1"
    )
    .bind(family_name)
    .fetch_optional(pool)
    .await
}

// Récupérer un joueur par son ID
pub async fn get_player_by_id(
    pool: &PgPool,
    player_id: i64,
) -> Result<Option<Player>, sqlx::Error> {
    sqlx::query_as::<_, Player>(
        "SELECT * FROM game.players WHERE id = $1"
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await
}

// Créer un joueur
pub async fn create_player(
    pool: &PgPool,
    family_name: &str,
    language_id: i16,
    origin_location: &str,
    motto: Option<&str>,
) -> Result<Player, sqlx::Error> {
    sqlx::query_as::<_, Player>(
        "INSERT INTO game.players (family_name, language_id, origin_location, motto)
         VALUES ($1, $2, $3, $4)
         RETURNING *"
    )
    .bind(family_name)
    .bind(language_id)
    .bind(origin_location)
    .bind(motto)
    .fetch_one(pool)
    .await
}

// Créer un joueur avec mot de passe
pub async fn create_player_with_password(
    pool: &PgPool,
    family_name: &str,
    language_id: i16,
    origin_location: &str,
    motto: Option<&str>,
    password_hash: &str,
) -> Result<Player, sqlx::Error> {
    sqlx::query_as::<_, Player>(
        "INSERT INTO game.players (family_name, language_id, origin_location, motto, password_hash)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *"
    )
    .bind(family_name)
    .bind(language_id)
    .bind(origin_location)
    .bind(motto)
    .bind(password_hash)
    .fetch_one(pool)
    .await
}

// Mettre à jour la date de dernière connexion
pub async fn update_last_login(
    pool: &PgPool,
    player_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE game.players
         SET last_login_at = NOW(), updated_at = NOW()
         WHERE id = $1"
    )
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

// Récupérer ou créer un joueur
pub async fn get_or_create_player(
    pool: &PgPool,
    family_name: &str,
    language_id: i16,
    origin_location: &str,
    motto: Option<&str>,
) -> Result<Player, sqlx::Error> {
    // Essayer de récupérer le joueur existant
    if let Some(player) = get_player_by_family_name(pool, family_name).await? {
        Ok(player)
    } else {
        // Créer un nouveau joueur s'il n'existe pas
        create_player(pool, family_name, language_id, origin_location, motto).await
    }
}

// Créer un personnage pour un joueur
pub async fn create_character(
    pool: &PgPool,
    player_id: i64,
    first_name: &str,
    family_name: &str,
    second_name: Option<&str>,
    nickname: Option<&str>,
    motto: Option<&str>,
) -> Result<Character, sqlx::Error> {
    sqlx::query_as::<_, Character>(
        "INSERT INTO game.characters 
         (player_id, first_name, family_name, second_name, nickname, motto)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
    .bind(player_id)
    .bind(first_name)
    .bind(family_name)
    .bind(second_name)
    .bind(nickname)
    .bind(motto)
    .fetch_one(pool)
    .await
}

// Récupérer tous les personnages d'un joueur
pub async fn get_player_characters(
    pool: &PgPool,
    player_id: i64,
) -> Result<Vec<Character>, sqlx::Error> {
    sqlx::query_as::<_, Character>(
        "SELECT * FROM game.characters 
         WHERE player_id = $1
         ORDER BY created_at DESC"
    )
    .bind(player_id)
    .fetch_all(pool)
    .await
}

// Récupérer un joueur avec ses personnages
pub async fn get_player_with_characters(
    pool: &PgPool,
    player_id: i64,
) -> Result<(Player, Vec<Character>), sqlx::Error> {
    let player = sqlx::query_as::<_, Player>(
        "SELECT * FROM game.players WHERE id = $1"
    )
    .bind(player_id)
    .fetch_one(pool)
    .await?;

    let characters = get_player_characters(pool, player_id).await?;

    Ok((player, characters))
}

// Ajouter des armoiries à un joueur
pub async fn set_player_coat_of_arms(
    pool: &PgPool,
    player_id: i64,
    coat_of_arms_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE game.players
         SET coat_of_arms_id = $1, updated_at = NOW()
         WHERE id = $2"
    )
    .bind(coat_of_arms_id)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(())
}

// Récupérer ou créer un personnage par défaut pour un joueur
pub async fn get_or_create_default_character(
    pool: &PgPool,
    player_id: i64,
    player_family_name: &str,
) -> Result<Character, sqlx::Error> {
    // Essayer de récupérer les personnages existants
    let characters = get_player_characters(pool, player_id).await?;

    // Si au moins un personnage existe, retourner le premier
    if let Some(character) = characters.into_iter().next() {
        Ok(character)
    } else {
        // Créer un personnage par défaut "Jean-Michel Lambda"
        create_character(
            pool,
            player_id,
            "Jean-Michel",
            player_family_name,
            None, // second_name
            Some("Lambda"), // nickname
            None, // motto
        ).await
    }
}