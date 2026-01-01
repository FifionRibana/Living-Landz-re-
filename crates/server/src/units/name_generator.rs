use rand::Rng;
use std::fs;
use std::path::Path;

/// Structure pour générer des noms aléatoires à partir de fichiers
pub struct NameGenerator {
    male_first_names: Vec<String>,
    female_first_names: Vec<String>,
    last_names: Vec<String>,
}

impl NameGenerator {
    /// Charge les noms depuis les fichiers texte
    pub fn load_from_files() -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = Path::new("assets/data");

        let male_first_names = Self::load_names_from_file(base_path.join("males.txt"))?;
        let female_first_names = Self::load_names_from_file(base_path.join("females.txt"))?;
        let last_names = Self::load_names_from_file(base_path.join("lastNames.txt"))?;

        tracing::info!(
            "Loaded {} male names, {} female names, {} last names",
            male_first_names.len(),
            female_first_names.len(),
            last_names.len()
        );

        Ok(Self {
            male_first_names,
            female_first_names,
            last_names,
        })
    }

    /// Charge les noms depuis un fichier (format: Nom;Pays)
    fn load_names_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path.as_ref())?;
        let names: Vec<String> = content
            .lines()
            .filter_map(|line| {
                // Parse le format "Nom;Pays" et prend seulement le nom
                line.split(';')
                    .next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
            .collect();

        if names.is_empty() {
            return Err(format!("No names found in file: {:?}", path.as_ref()).into());
        }

        Ok(names)
    }

    /// Génère un nom aléatoire (prénom + nom de famille)
    /// Si gender est Some(true), utilise les prénoms masculins
    /// Si gender est Some(false), utilise les prénoms féminins
    /// Si gender est None, choisit aléatoirement
    pub fn generate_random_name(&self, gender: Option<bool>) -> (String, String) {
        let mut rng = rand::rng();

        // Choisir le prénom
        let first_name = match gender {
            Some(true) => {
                // Masculin
                if self.male_first_names.is_empty() {
                    "John".to_string()
                } else {
                    let idx = rng.random_range(0..self.male_first_names.len());
                    self.male_first_names[idx].clone()
                }
            }
            Some(false) => {
                // Féminin
                if self.female_first_names.is_empty() {
                    "Jane".to_string()
                } else {
                    let idx = rng.random_range(0..self.female_first_names.len());
                    self.female_first_names[idx].clone()
                }
            }
            None => {
                // Aléatoire
                if rng.random_bool(0.5) {
                    if self.male_first_names.is_empty() {
                        "John".to_string()
                    } else {
                        let idx = rng.random_range(0..self.male_first_names.len());
                        self.male_first_names[idx].clone()
                    }
                } else if self.female_first_names.is_empty() {
                    "Jane".to_string()
                } else {
                    let idx = rng.random_range(0..self.female_first_names.len());
                    self.female_first_names[idx].clone()
                }
            }
        };

        // Choisir le nom de famille
        let last_name = if self.last_names.is_empty() {
            "Smith".to_string()
        } else {
            let idx = rng.random_range(0..self.last_names.len());
            self.last_names[idx].clone()
        };

        (first_name, last_name)
    }

    /// Génère un prénom masculin aléatoire
    pub fn generate_male_first_name(&self) -> String {
        if self.male_first_names.is_empty() {
            return "John".to_string();
        }
        let mut rng = rand::rng();
        let idx = rng.random_range(0..self.male_first_names.len());
        self.male_first_names[idx].clone()
    }

    /// Génère un prénom féminin aléatoire
    pub fn generate_female_first_name(&self) -> String {
        if self.female_first_names.is_empty() {
            return "Jane".to_string();
        }
        let mut rng = rand::rng();
        let idx = rng.random_range(0..self.female_first_names.len());
        self.female_first_names[idx].clone()
    }

    /// Génère un nom de famille aléatoire
    pub fn generate_last_name(&self) -> String {
        if self.last_names.is_empty() {
            return "Smith".to_string();
        }
        let mut rng = rand::rng();
        let idx = rng.random_range(0..self.last_names.len());
        self.last_names[idx].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generation() {
        // Ce test ne fonctionnera que si les fichiers existent
        if let Ok(generator) = NameGenerator::load_from_files() {
            // Tester génération masculine
            let (first, last) = generator.generate_random_name(Some(true));
            assert!(!first.is_empty());
            assert!(!last.is_empty());

            // Tester génération féminine
            let (first, last) = generator.generate_random_name(Some(false));
            assert!(!first.is_empty());
            assert!(!last.is_empty());

            // Tester génération aléatoire
            let (first, last) = generator.generate_random_name(None);
            assert!(!first.is_empty());
            assert!(!last.is_empty());
        }
    }
}
