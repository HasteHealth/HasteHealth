use axum::Json;
use axum_extra::routing::TypedPath;
use serde_json::{Value, json};

#[derive(TypedPath)]
#[typed_path("/smart-style.json")]
pub struct SmartStylePath;

/// SMART App Launch styling parameters, fixed until tenant branding feeds them.
pub async fn smart_style_get(_: SmartStylePath) -> Json<Value> {
    Json(json!({
        "color_background": "#ffffff",
        "color_error": "#c62828",
        "color_highlight": "#1565c0",
        "color_modal_backdrop": "rgba(0, 0, 0, 0.5)",
        "color_success": "#2e7d32",
        "color_text": "#212121",
        "dim_border_radius": "4px",
        "dim_font_size": "16px",
        "dim_spacing_size": "8px",
        "font_family_body": "system-ui, -apple-system, \"Segoe UI\", Roboto, sans-serif",
        "font_family_heading": "system-ui, -apple-system, \"Segoe UI\", Roboto, sans-serif"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serves_every_smart_style_property() {
        let Json(style) = smart_style_get(SmartStylePath).await;
        for key in [
            "color_background",
            "color_error",
            "color_highlight",
            "color_modal_backdrop",
            "color_success",
            "color_text",
            "dim_border_radius",
            "dim_font_size",
            "dim_spacing_size",
            "font_family_body",
            "font_family_heading",
        ] {
            assert!(style[key].is_string(), "missing {key}");
        }
    }
}
