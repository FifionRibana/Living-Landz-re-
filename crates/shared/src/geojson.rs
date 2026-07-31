//! Minimal GeoJSON polyline reader (LL-B/LL-C).
//!
//! Hand-parses `MultiLineString` / `LineString` geometries from a GeoJSON string
//! into polylines, avoiding a `geojson` crate dependency. Lives in `shared` so
//! both the server (Ymir coastline) and the client (Ymir cliff debug overlay)
//! can reuse it. Coordinates are returned verbatim in the source's own space
//! (for Ymir: cell space, y=0 = south); the caller applies any transform/flip.

/// Parse every `MultiLineString`/`LineString` geometry in a GeoJSON document
/// into polylines (`[x, y]` points). Accepts a `FeatureCollection`, a single
/// `Feature`, or a bare geometry. Polylines with fewer than 2 points are
/// dropped. Returns an empty vec on malformed input (callers treat empty as
/// "no data" rather than an error).
pub fn parse_multilinestring_geojson(raw: &str) -> Vec<Vec<[f32; 2]>> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };

    let mut polylines: Vec<Vec<[f32; 2]>> = Vec::new();
    let mut push_geometry = |geom: &serde_json::Value, out: &mut Vec<Vec<[f32; 2]>>| {
        let Some(kind) = geom.get("type").and_then(|t| t.as_str()) else {
            return;
        };
        let Some(coords) = geom.get("coordinates") else {
            return;
        };
        let parse_line = |line: &serde_json::Value| -> Vec<[f32; 2]> {
            line.as_array()
                .map(|pts| {
                    pts.iter()
                        .filter_map(|p| {
                            let a = p.as_array()?;
                            Some([a.first()?.as_f64()? as f32, a.get(1)?.as_f64()? as f32])
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        match kind {
            "MultiLineString" => {
                if let Some(lines) = coords.as_array() {
                    for line in lines {
                        let pl = parse_line(line);
                        if pl.len() >= 2 {
                            out.push(pl);
                        }
                    }
                }
            }
            "LineString" => {
                let pl = parse_line(coords);
                if pl.len() >= 2 {
                    out.push(pl);
                }
            }
            _ => {}
        }
    };

    if let Some(features) = json.get("features").and_then(|f| f.as_array()) {
        for feat in features {
            if let Some(geom) = feat.get("geometry") {
                push_geometry(geom, &mut polylines);
            }
        }
    } else if let Some(geom) = json.get("geometry") {
        push_geometry(geom, &mut polylines);
    } else {
        push_geometry(&json, &mut polylines);
    }

    polylines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_feature_collection_multilinestring() {
        let raw = r#"{
            "type": "FeatureCollection",
            "features": [
                {"geometry": {"type": "MultiLineString",
                    "coordinates": [[[0.0, 1.0], [2.0, 3.0]], [[4.0, 5.0], [6.0, 7.0], [8.0, 9.0]]]}}
            ]
        }"#;
        let pls = parse_multilinestring_geojson(raw);
        assert_eq!(pls.len(), 2);
        assert_eq!(pls[0], vec![[0.0, 1.0], [2.0, 3.0]]);
        assert_eq!(pls[1].len(), 3);
        assert_eq!(pls[1][2], [8.0, 9.0]);
    }

    #[test]
    fn drops_degenerate_and_handles_garbage() {
        // A 1-point line is dropped; malformed input yields empty.
        let one_pt = r#"{"geometry":{"type":"LineString","coordinates":[[0.0,0.0]]}}"#;
        assert!(parse_multilinestring_geojson(one_pt).is_empty());
        assert!(parse_multilinestring_geojson("not json").is_empty());
    }
}
