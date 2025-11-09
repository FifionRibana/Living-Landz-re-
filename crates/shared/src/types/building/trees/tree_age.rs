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
