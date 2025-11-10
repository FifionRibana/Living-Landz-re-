use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum TreeAge {
    Sapling, // 0-5 ans - petit
    Young,   // 5-20 ans - moyen
    Mature,  // 20-50 ans - grand
    Adult,   // 50-100 ans - très grand
    Ancient, // 100-200 ans - énorme
    Secular, // 200+ ans - géant légendaire
}

impl TreeAge {
    pub fn get_tree_age(age: u32) -> TreeAge {
        match age {
            (0..5) => TreeAge::Sapling,
            (5..20) => TreeAge::Young,
            (20..50) => TreeAge::Mature,
            (50..100) => TreeAge::Adult,
            (100..200) => TreeAge::Ancient,
            (200..) => TreeAge::Secular,
        }
    }

    pub fn to_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn iter() -> impl Iterator<Item = TreeAge> {
        [
            TreeAge::Sapling,
            TreeAge::Young,
            TreeAge::Mature,
            TreeAge::Adult,
            TreeAge::Ancient,
            // TreeAge::Secular,
        ]
        .into_iter()
    }
}
