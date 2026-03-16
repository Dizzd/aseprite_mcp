/// Utility functions for the Aseprite MCP server.

// ============================================================================
// Color Parsing
// ============================================================================

/// Validate a hex color string format. Returns Ok(()) if valid, Err with message otherwise.
pub fn validate_hex_color(hex: &str) -> Result<(), String> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 && h.len() != 8 {
        return Err(format!(
            "expected 6 or 8 hex digits (got {}), format: #rrggbb or #rrggbbaa",
            h.len()
        ));
    }
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("contains non-hex characters".to_string());
    }
    Ok(())
}

/// Parse a hex color string (#rrggbb) to (r, g, b).
/// Assumes input has been validated with `validate_hex_color`.
pub fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

/// Parse a hex color string (#rrggbb or #rrggbbaa) to (r, g, b, a).
/// Assumes input has been validated with `validate_hex_color`.
pub fn parse_hex_color_with_alpha(hex: &str) -> (u8, u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let a = if hex.len() >= 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    (r, g, b, a)
}

// ============================================================================
// Numeric Validation
// ============================================================================

/// Validate that width/height dimensions are valid (> 0).
pub fn validate_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("Width and height must be greater than 0".to_string());
    }
    Ok(())
}

/// Clamp a value between min and max.
#[allow(dead_code)]
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Ensure a value is within a valid range, clamping if necessary.
pub fn clamp_u32(value: u32, min: u32, max: u32) -> u32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
