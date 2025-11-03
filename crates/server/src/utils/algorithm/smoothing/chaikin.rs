pub fn smooth_contour_chaikin(points: &[[f64; 2]], iterations: u32) -> Vec<[f64; 2]> {
    // ✨ CRUCIAL: Fermer correctement avant lissage
    let mut closed_points = points.to_vec();
    if closed_points.len() > 1 && (closed_points.first() != closed_points.last()) {
        // Fusionner premier et dernier point
        let first = closed_points[0];
        *closed_points.last_mut().unwrap() = first;
    }

    let mut result = closed_points;

    for _ in 0..iterations {
        let n = result.len();
        let mut smoothed = Vec::new();
        for i in 0..n {
            let p_curr = result[i];
            let p_next = result[(i + 1) % n];

            smoothed.push([
                0.75 * p_curr[0] + 0.25 * p_next[0],
                0.75 * p_curr[1] + 0.25 * p_next[1],
            ]);
            smoothed.push([
                0.25 * p_curr[0] + 0.75 * p_next[0],
                0.25 * p_curr[1] + 0.75 * p_next[1],
            ]);
        }
        result = smoothed;
    }

    result
}
