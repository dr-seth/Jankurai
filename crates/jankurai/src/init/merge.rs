use anyhow::Result;

pub fn merge_marker(path: &str) -> String {
    format!("\n<!-- jankurai merge marker: review and merge canonical guidance for {path} -->\n")
}

pub fn merge_json(existing: &str, template: &str) -> Result<String> {
    let mut base: serde_json::Value = if existing.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(existing).unwrap_or_else(|_| serde_json::json!({}))
    };
    let new: serde_json::Value = serde_json::from_str(template)?;

    merge_json_values(&mut base, &new);
    Ok(serde_json::to_string_pretty(&base)?)
}

fn merge_json_values(base: &mut serde_json::Value, new: &serde_json::Value) {
    match (base, new) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(new_map)) => {
            for (k, v) in new_map {
                if !base_map.contains_key(k) {
                    base_map.insert(k.clone(), v.clone());
                } else {
                    let existing = base_map.get_mut(k).unwrap();
                    merge_json_values(existing, v);
                }
            }
        }
        (serde_json::Value::Array(base_arr), serde_json::Value::Array(new_arr)) => {
            for item in new_arr {
                if !base_arr.contains(item) {
                    base_arr.push(item.clone());
                }
            }
        }
        _ => {} // Don't overwrite existing primitives or mismatched types
    }
}

pub fn merge_toml(existing: &str, template: &str) -> Result<String> {
    let mut base: toml::Value = if existing.trim().is_empty() {
        toml::Value::Table(toml::map::Map::new())
    } else {
        toml::from_str(existing).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
    };
    let new: toml::Value = toml::from_str(template)?;

    merge_toml_values(&mut base, &new);
    // toml::to_string_pretty handles serialization cleanly
    Ok(toml::to_string_pretty(&base)?)
}

fn merge_toml_values(base: &mut toml::Value, new: &toml::Value) {
    match (base, new) {
        (toml::Value::Table(base_map), toml::Value::Table(new_map)) => {
            for (k, v) in new_map {
                if !base_map.contains_key(k) {
                    base_map.insert(k.clone(), v.clone());
                } else {
                    let existing = base_map.get_mut(k).unwrap();
                    merge_toml_values(existing, v);
                }
            }
        }
        (toml::Value::Array(base_arr), toml::Value::Array(new_arr)) => {
            for item in new_arr {
                if !base_arr.contains(item) {
                    base_arr.push(item.clone());
                }
            }
        }
        _ => {} // Don't overwrite existing primitives or mismatched types
    }
}

pub fn merge_lines(existing: &str, template: &str) -> Result<String> {
    let mut out = String::from(existing);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    let existing_lines: std::collections::HashSet<&str> =
        existing.lines().map(|s| s.trim()).collect();
    for line in template.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || existing_lines.contains(trimmed) {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    Ok(out)
}
