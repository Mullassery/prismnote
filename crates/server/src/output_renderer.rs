use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputViewConfig {
    pub zoom_level: f32,
    pub fullscreen: bool,
    pub show_code: bool,
    pub max_height: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderedOutput {
    pub id: String,
    pub cell_id: String,
    pub output_type: String,
    pub content: String,
    pub mime_type: String,
    pub view_config: OutputViewConfig,
}

pub struct OutputRenderer {
    pub default_zoom: f32,
    pub max_zoom: f32,
    pub min_zoom: f32,
}

impl OutputRenderer {
    pub fn new() -> Self {
        Self {
            default_zoom: 1.0,
            max_zoom: 3.0,
            min_zoom: 0.5,
        }
    }

    pub fn set_zoom(&self, current: f32, delta: f32) -> f32 {
        let new_zoom = current + delta;
        new_zoom.min(self.max_zoom).max(self.min_zoom)
    }

    pub fn enable_fullscreen(&self, output_id: &str) -> String {
        format!("Opening fullscreen view for output: {}", output_id)
    }

    pub fn get_fullscreen_css(zoom: f32) -> String {
        format!(
            r#"
.output-fullscreen {{
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
    background: white;
    display: flex;
    flex-direction: column;
    overflow: auto;
}}

.output-fullscreen-content {{
    flex: 1;
    padding: 20px;
    transform: scale({});
    transform-origin: top left;
    width: 100%;
    height: auto;
}}

.output-fullscreen-toolbar {{
    position: sticky;
    top: 0;
    background: #f5f5f5;
    border-bottom: 1px solid #ddd;
    padding: 10px;
    display: flex;
    gap: 10px;
    align-items: center;
    z-index: 1001;
}}

.zoom-control {{
    display: flex;
    gap: 5px;
    align-items: center;
}}

.zoom-control button {{
    padding: 5px 10px;
    background: white;
    border: 1px solid #ccc;
    border-radius: 4px;
    cursor: pointer;
}}

.zoom-control button:hover {{
    background: #f0f0f0;
}}

.zoom-value {{
    min-width: 50px;
    text-align: center;
}}
"#,
            zoom
        )
    }

    pub fn format_html_output(content: &str, zoom: f32) -> String {
        format!(
            r#"<div style="transform: scale({zoom}); transform-origin: top left; width: 100%;">{content}</div>"#
        )
    }

    pub fn format_image_output(src: &str, zoom: f32) -> String {
        format!(
            r#"<img src="{src}" style="max-width: 100%; height: auto; transform: scale({zoom}); transform-origin: top left;" />"#
        )
    }

    pub fn format_table_output(html: &str, zoom: f32) -> String {
        format!(
            r#"<div style="overflow-x: auto; transform: scale({zoom}); transform-origin: top left;">{html}</div>"#
        )
    }
}

impl Default for OutputRenderer {
    fn default() -> Self {
        Self::new()
    }
}
