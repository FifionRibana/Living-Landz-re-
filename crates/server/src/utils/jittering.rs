use bevy::prelude::*;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Applique un jitter déterministe aux points du contour.
/// Le même point produira toujours le même jitter, garantissant
/// que les frontières adjacentes coïncident.
pub fn apply_jitter(points: &mut Vec<Vec2>, amplitude: f32, seed: u64) {
    for point in points.iter_mut() {
        let jitter = deterministic_jitter(*point, amplitude, seed);
        *point += jitter;
    }
}

/// Calcule un jitter déterministe basé sur les coordonnées du point.
///
/// On utilise un hash des coordonnées quantifiées pour garantir que
/// deux points suffisamment proches (même après erreurs de flottants)
/// produisent le même jitter.
fn deterministic_jitter(point: Vec2, amplitude: f32, seed: u64) -> Vec2 {
    // Quantifier les coordonnées pour éviter les problèmes de précision
    // On arrondit à 0.1 unité près (ajuster selon la taille de tes hexagones)
    let quantization = 0.1;
    let qx = (point.x / quantization).round() as i64;
    let qy = (point.y / quantization).round() as i64;

    // Créer un hash déterministe à partir des coordonnées quantifiées
    let hash_x = compute_hash((qx, qy, seed, 0u8));
    let hash_y = compute_hash((qx, qy, seed, 1u8));

    // Convertir les hash en valeurs dans [-1, 1]
    let normalized_x = hash_to_normalized(hash_x);
    let normalized_y = hash_to_normalized(hash_y);

    // Appliquer l'amplitude
    Vec2::new(normalized_x * amplitude, normalized_y * amplitude)
}

/// Calcule un hash 64 bits à partir d'une valeur hashable
fn compute_hash<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Convertit un hash u64 en une valeur normalisée dans [-1, 1]
fn hash_to_normalized(hash: u64) -> f32 {
    // Prendre les 32 bits de poids faible et normaliser
    let normalized = (hash & 0xFFFFFFFF) as f32 / (0xFFFFFFFFu32 as f32);
    // Transformer [0, 1] en [-1, 1]
    normalized * 2.0 - 1.0
}
