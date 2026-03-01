use bevy::prelude::*;
use rand::Rng;
use rand::seq::IndexedRandom;

// ─── Gender ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gender {
    #[default]
    Male,
    Female,
}

impl Gender {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Male => "Homme",
            Self::Female => "Femme",
        }
    }

    pub fn folder_suffix(&self) -> &'static str {
        match self {
            Self::Male => "m",
            Self::Female => "f",
        }
    }
}

// ─── Name lists ─────────────────────────────────────────────────────────────

const MALE_NAMES: &[&str] = &[
    "Alaric", "Baudouin", "Cédric", "Dieudonné", "Enguerrand",
    "Foulques", "Gaucelm", "Hugues", "Isarn", "Jaufré",
    "Lambert", "Mainard", "Norbert", "Othon", "Pons",
    "Renaud", "Sigebert", "Thibaut", "Ulric", "Valéran",
    "Arnaud", "Bertrand", "Conrad", "Drogon", "Eudes",
    "Ferrand", "Géraud", "Hardouin", "Ithier", "Jourdain",
    "Landry", "Manassès", "Nivelon", "Ogier", "Pépin",
    "Raimbaut", "Sanche", "Tancrède", "Ursion", "Vivien",
    "Adhémar", "Bérenger", "Clodomir", "Dodon", "Ermenric",
    "Foulbert", "Gontran", "Herbrand", "Imbert", "Joceran",
];

const FEMALE_NAMES: &[&str] = &[
    "Adélaïde", "Béatrice", "Clémence", "Dhuoda", "Ermengarde",
    "Fressende", "Gersende", "Héloïse", "Isabeau", "Jehanne",
    "Laudine", "Mahaut", "Norgarde", "Ombeline", "Pétronille",
    "Richilde", "Sibylle", "Tiphaine", "Urraque", "Vierne",
    "Aliénor", "Blanche", "Constance", "Douce", "Elvire",
    "Flandrine", "Guiraude", "Hersende", "Ide", "Juliane",
    "Léonore", "Marguerite", "Nicolette", "Oriane", "Plaisance",
    "Raymonde", "Sancie", "Thomasse", "Ursule", "Yolande",
    "Alix", "Brunehaut", "Cunégonde", "Dieudonnée", "Esclarmonde",
    "Frédesende", "Guibour", "Hildegarde", "Iseult", "Jordane",
];

// ─── Layer categories ───────────────────────────────────────────────────────

/// Categories of visual layers, rendered bottom-to-top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerCategory {
    Bust,
    Face,
    Hair,
    Clothes,
}

impl LayerCategory {
    /// All categories in render order (bottom layer first).
    pub const ALL: &'static [LayerCategory] = &[
        LayerCategory::Bust,
        LayerCategory::Face,
        LayerCategory::Clothes,
        LayerCategory::Hair,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Bust => "Buste",
            Self::Face => "Visage",
            Self::Hair => "Cheveux",
            Self::Clothes => "Habits",
        }
    }

    /// Asset folder name under `sprites/character/layers/`.
    pub fn folder(&self) -> &'static str {
        match self {
            Self::Bust => "bust",
            Self::Face => "face",
            Self::Hair => "hair",
            Self::Clothes => "clothes",
        }
    }

    /// Icon character for the selector row.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Bust => "B",
            Self::Face => "V",
            Self::Hair => "C",
            Self::Clothes => "H",
        }
    }
}

// ─── Layer state ────────────────────────────────────────────────────────────

/// One layer's selection state.
#[derive(Debug, Clone)]
pub struct LayerState {
    pub category: LayerCategory,
    pub current: usize,
    pub total: usize,
}

impl LayerState {
    pub fn new(category: LayerCategory, total: usize) -> Self {
        Self {
            category,
            current: 0,
            total,
        }
    }

    pub fn next(&mut self) {
        if self.total > 0 {
            self.current = (self.current + 1) % self.total;
        }
    }

    pub fn prev(&mut self) {
        if self.total > 0 {
            self.current = (self.current + self.total - 1) % self.total;
        }
    }

    pub fn randomize(&mut self) {
        if self.total > 0 {
            self.current = rand::rng().random_range(0..self.total);
        }
    }

    /// Asset path for the current preset, e.g. `sprites/character/layers/bust/bust_01.png`.
    pub fn asset_path(&self) -> String {
        format!(
            "sprites/character/layers/{}/{}_{:02}.png",
            self.category.folder(),
            self.category.folder(),
            self.current + 1
        )
    }

    /// Display string, e.g. "3 / 8".
    pub fn counter_text(&self) -> String {
        if self.total == 0 {
            "—".to_string()
        } else {
            format!("{} / {}", self.current + 1, self.total)
        }
    }
}

// ─── Main state ─────────────────────────────────────────────────────────────

/// Resource holding the full character creation state.
#[derive(Resource, Debug, Clone)]
pub struct CharacterCreationState {
    pub gender: Gender,
    pub layers: Vec<LayerState>,
    pub first_name: String,
    /// When Some, the name should be pushed to the TextInput buffer.
    /// Consumed by `push_name_to_input`. Prevents sync conflicts with user typing.
    pub pending_name_push: Option<String>,
}

impl Default for CharacterCreationState {
    fn default() -> Self {
        let gender = Gender::Male;
        let first_name = Self::pick_random_name(gender);
        Self {
            gender,
            layers: vec![
                LayerState::new(LayerCategory::Bust, 1),
                LayerState::new(LayerCategory::Face, 8),
                LayerState::new(LayerCategory::Clothes, 3),
                LayerState::new(LayerCategory::Hair, 5),
            ],
            pending_name_push: Some(first_name.clone()),
            first_name,
        }
    }
}

impl CharacterCreationState {
    pub fn layer_mut(&mut self, category: LayerCategory) -> Option<&mut LayerState> {
        self.layers.iter_mut().find(|l| l.category == category)
    }

    pub fn layer(&self, category: LayerCategory) -> Option<&LayerState> {
        self.layers.iter().find(|l| l.category == category)
    }

    /// Pick a random name for the given gender.
    pub fn pick_random_name(gender: Gender) -> String {
        let mut rng = rand::rng();
        let names = match gender {
            Gender::Male => MALE_NAMES,
            Gender::Female => FEMALE_NAMES,
        };
        names
            .choose(&mut rng)
            .unwrap_or(&"Anonyme")
            .to_string()
    }

    /// Randomize the first name only (keeping gender).
    pub fn randomize_name(&mut self) {
        self.first_name = Self::pick_random_name(self.gender);
        self.pending_name_push = Some(self.first_name.clone());
    }

    /// Randomize everything: all layers + first name.
    pub fn randomize_all(&mut self) {
        for layer in &mut self.layers {
            layer.randomize();
        }
        self.randomize_name();
    }

    /// Switch gender — re-rolls name to match.
    pub fn set_gender(&mut self, gender: Gender) {
        if self.gender != gender {
            self.gender = gender;
            self.randomize_name();
        }
    }
}
