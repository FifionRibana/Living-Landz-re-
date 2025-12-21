use rand::Rng;
use shared::ProfessionEnum;

/// Variantes de portraits disponibles pour les hommes
const MALE_VARIANT_IDS: &[&str] = &["02m", "09m", "13m", "20m", "22m", "24m"];

/// Variantes de portraits disponibles pour les femmes
const FEMALE_VARIANT_IDS: &[&str] = &["01f", "03f", "08f", "10f", "17f", "24f"];

/// Structure pour générer les URLs de portraits
pub struct PortraitGenerator;

impl PortraitGenerator {
    /// Génère un ID de variante aléatoire selon le genre
    /// - "male" retourne un ID masculin (ex: "02m")
    /// - "female" retourne un ID féminin (ex: "01f")
    pub fn generate_random_variant_id(gender: &str) -> String {
        let mut rng = rand::rng();

        match gender {
            "male" => {
                let idx = rng.gen_range(0..MALE_VARIANT_IDS.len());
                MALE_VARIANT_IDS[idx].to_string()
            }
            "female" => {
                let idx = rng.gen_range(0..FEMALE_VARIANT_IDS.len());
                FEMALE_VARIANT_IDS[idx].to_string()
            }
            _ => {
                // Par défaut, choisir masculin
                let idx = rng.gen_range(0..MALE_VARIANT_IDS.len());
                MALE_VARIANT_IDS[idx].to_string()
            }
        }
    }

    /// Convertit un ProfessionEnum en nom de fichier de portrait
    fn profession_to_filename(profession: ProfessionEnum) -> &'static str {
        match profession {
            ProfessionEnum::Farmer => "Farmer",
            ProfessionEnum::Lumberjack => "Woodcutter",
            ProfessionEnum::Miner => "Miner_Coal",
            ProfessionEnum::Blacksmith => "Blacksmith",
            ProfessionEnum::Carpenter => "Luthier",
            ProfessionEnum::Mason => "Quarrymen",
            ProfessionEnum::Baker => "PastryCooker",
            ProfessionEnum::Brewer => "Cooker",
            ProfessionEnum::Cook => "Cooker",
            ProfessionEnum::Fisherman => "Fishermen",
            ProfessionEnum::Hunter => "Hunter",
            ProfessionEnum::Healer => "Doctor",
            ProfessionEnum::Merchant => "Jeweller",
            ProfessionEnum::Scholar => "Architect",
            ProfessionEnum::Warrior => "Knight",
            ProfessionEnum::Unknown => "Farmer", // Défaut
        }
    }

    /// Génère l'URL complète du portrait pour une unité
    ///
    /// # Arguments
    /// * `gender` - Le genre de l'unité ("male" ou "female")
    /// * `variant_id` - L'ID de la variante (ex: "02m", "01f")
    /// * `profession` - La profession de l'unité
    ///
    /// # Returns
    /// L'URL relative du portrait (ex: "sprites/portraits/male/02m/02m_Farmer.jpg")
    pub fn generate_portrait_url(
        gender: &str,
        variant_id: &str,
        profession: ProfessionEnum,
    ) -> String {
        let profession_name = Self::profession_to_filename(profession);

        format!(
            "sprites/portraits/{}/{}/{}_{}.jpg",
            gender,
            variant_id,
            variant_id,
            profession_name
        )
    }

    /// Génère à la fois un variant_id aléatoire et l'URL du portrait
    ///
    /// # Returns
    /// Un tuple (variant_id, avatar_url)
    pub fn generate_variant_and_url(
        gender: &str,
        profession: ProfessionEnum,
    ) -> (String, String) {
        let variant_id = Self::generate_random_variant_id(gender);
        let avatar_url = Self::generate_portrait_url(gender, &variant_id, profession);
        (variant_id, avatar_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_male_variant() {
        let variant = PortraitGenerator::generate_random_variant_id("male");
        assert!(MALE_VARIANT_IDS.contains(&variant.as_str()));
    }

    #[test]
    fn test_generate_female_variant() {
        let variant = PortraitGenerator::generate_random_variant_id("female");
        assert!(FEMALE_VARIANT_IDS.contains(&variant.as_str()));
    }

    #[test]
    fn test_generate_portrait_url() {
        let url = PortraitGenerator::generate_portrait_url(
            "male",
            "02m",
            ProfessionEnum::Farmer,
        );
        assert_eq!(url, "sprites/portraits/male/02m/02m_Farmer.jpg");
    }

    #[test]
    fn test_generate_variant_and_url() {
        let (variant_id, url) = PortraitGenerator::generate_variant_and_url(
            "female",
            ProfessionEnum::Blacksmith,
        );
        assert!(FEMALE_VARIANT_IDS.contains(&variant_id.as_str()));
        assert!(url.contains(&variant_id));
        assert!(url.contains("Blacksmith"));
    }
}
