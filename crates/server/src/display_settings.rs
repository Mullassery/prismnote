use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub id: String,
    pub user_id: String,

    // Theme settings
    pub theme: String, // light, dark, auto
    pub accent_color: String, // hex color code

    // Editor settings
    pub editor_font_family: String,
    pub editor_font_size: u32, // in pixels
    pub editor_line_height: f32,
    pub editor_tab_width: u32,
    pub editor_word_wrap: bool,
    pub editor_minimap: bool,
    pub editor_line_numbers: bool,

    // Notebook display
    pub notebook_cell_line_limit: u32, // cells show N lines before truncation
    pub notebook_output_limit: u32, // MB
    pub notebook_show_execution_count: bool,
    pub notebook_auto_collapse_output: bool,
    pub notebook_show_variable_types: bool,

    // UI settings
    pub sidebar_width: u32, // pixels
    pub sidebar_visible: bool,
    pub right_panel_visible: bool,
    pub status_bar_visible: bool,

    // Data display
    pub table_rows_per_page: u32,
    pub chart_theme: String, // light, dark
    pub chart_animation: bool,

    // Accessibility
    pub font_scaling: f32, // 0.8 to 1.5
    pub high_contrast: bool,
    pub reduce_motion: bool,

    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplaySettingsUpdate {
    pub theme: Option<String>,
    pub accent_color: Option<String>,
    pub editor_font_family: Option<String>,
    pub editor_font_size: Option<u32>,
    pub editor_line_height: Option<f32>,
    pub editor_tab_width: Option<u32>,
    pub editor_word_wrap: Option<bool>,
    pub editor_minimap: Option<bool>,
    pub editor_line_numbers: Option<bool>,
    pub notebook_cell_line_limit: Option<u32>,
    pub notebook_output_limit: Option<u32>,
    pub notebook_show_execution_count: Option<bool>,
    pub notebook_auto_collapse_output: Option<bool>,
    pub notebook_show_variable_types: Option<bool>,
    pub sidebar_width: Option<u32>,
    pub sidebar_visible: Option<bool>,
    pub right_panel_visible: Option<bool>,
    pub status_bar_visible: Option<bool>,
    pub table_rows_per_page: Option<u32>,
    pub chart_theme: Option<String>,
    pub chart_animation: Option<bool>,
    pub font_scaling: Option<f32>,
    pub high_contrast: Option<bool>,
    pub reduce_motion: Option<bool>,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: "default".to_string(),
            theme: "auto".to_string(),
            accent_color: "#2563EB".to_string(),

            editor_font_family: "JetBrains Mono, Monaco, Menlo, monospace".to_string(),
            editor_font_size: 14,
            editor_line_height: 1.6,
            editor_tab_width: 4,
            editor_word_wrap: true,
            editor_minimap: true,
            editor_line_numbers: true,

            notebook_cell_line_limit: 20,
            notebook_output_limit: 50,
            notebook_show_execution_count: true,
            notebook_auto_collapse_output: false,
            notebook_show_variable_types: true,

            sidebar_width: 240,
            sidebar_visible: true,
            right_panel_visible: true,
            status_bar_visible: true,

            table_rows_per_page: 25,
            chart_theme: "auto".to_string(),
            chart_animation: true,

            font_scaling: 1.0,
            high_contrast: false,
            reduce_motion: false,

            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

pub struct DisplaySettingsManager {
    config_dir: PathBuf,
}

impl DisplaySettingsManager {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    pub async fn get_settings(&self, user_id: &str) -> Result<DisplaySettings, String> {
        let config_file = self.config_dir.join(format!("{}_display.json", user_id));

        if config_file.exists() {
            let content = fs::read_to_string(&config_file)
                .await
                .map_err(|e| format!("Failed to read settings: {}", e))?;

            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse settings: {}", e))
        } else {
            Ok(DisplaySettings::default())
        }
    }

    pub async fn update_settings(
        &self,
        user_id: &str,
        update: DisplaySettingsUpdate,
    ) -> Result<DisplaySettings, String> {
        let mut settings = self.get_settings(user_id).await?;

        // Update only provided fields
        if let Some(theme) = update.theme {
            settings.theme = theme;
        }
        if let Some(color) = update.accent_color {
            settings.accent_color = color;
        }
        if let Some(font) = update.editor_font_family {
            settings.editor_font_family = font;
        }
        if let Some(size) = update.editor_font_size {
            settings.editor_font_size = size;
        }
        if let Some(height) = update.editor_line_height {
            settings.editor_line_height = height;
        }
        if let Some(width) = update.editor_tab_width {
            settings.editor_tab_width = width;
        }
        if let Some(wrap) = update.editor_word_wrap {
            settings.editor_word_wrap = wrap;
        }
        if let Some(minimap) = update.editor_minimap {
            settings.editor_minimap = minimap;
        }
        if let Some(numbers) = update.editor_line_numbers {
            settings.editor_line_numbers = numbers;
        }
        if let Some(limit) = update.notebook_cell_line_limit {
            settings.notebook_cell_line_limit = limit;
        }
        if let Some(limit) = update.notebook_output_limit {
            settings.notebook_output_limit = limit;
        }
        if let Some(show) = update.notebook_show_execution_count {
            settings.notebook_show_execution_count = show;
        }
        if let Some(collapse) = update.notebook_auto_collapse_output {
            settings.notebook_auto_collapse_output = collapse;
        }
        if let Some(show) = update.notebook_show_variable_types {
            settings.notebook_show_variable_types = show;
        }
        if let Some(width) = update.sidebar_width {
            settings.sidebar_width = width;
        }
        if let Some(visible) = update.sidebar_visible {
            settings.sidebar_visible = visible;
        }
        if let Some(visible) = update.right_panel_visible {
            settings.right_panel_visible = visible;
        }
        if let Some(visible) = update.status_bar_visible {
            settings.status_bar_visible = visible;
        }
        if let Some(rows) = update.table_rows_per_page {
            settings.table_rows_per_page = rows;
        }
        if let Some(theme) = update.chart_theme {
            settings.chart_theme = theme;
        }
        if let Some(animation) = update.chart_animation {
            settings.chart_animation = animation;
        }
        if let Some(scaling) = update.font_scaling {
            // Clamp to valid range
            settings.font_scaling = scaling.max(0.8).min(1.5);
        }
        if let Some(contrast) = update.high_contrast {
            settings.high_contrast = contrast;
        }
        if let Some(reduce) = update.reduce_motion {
            settings.reduce_motion = reduce;
        }

        settings.updated_at = chrono::Utc::now().to_rfc3339();

        // Save settings
        self.save_settings(user_id, &settings).await?;

        Ok(settings)
    }

    pub async fn reset_settings(&self, user_id: &str) -> Result<DisplaySettings, String> {
        let config_file = self.config_dir.join(format!("{}_display.json", user_id));

        if config_file.exists() {
            fs::remove_file(&config_file)
                .await
                .map_err(|e| format!("Failed to delete settings: {}", e))?;
        }

        let default_settings = DisplaySettings::default();
        self.save_settings(user_id, &default_settings).await?;

        Ok(default_settings)
    }

    pub async fn export_settings(&self, user_id: &str) -> Result<String, String> {
        let settings = self.get_settings(user_id).await?;
        serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))
    }

    pub async fn import_settings(
        &self,
        user_id: &str,
        settings_json: String,
    ) -> Result<DisplaySettings, String> {
        let mut settings: DisplaySettings = serde_json::from_str(&settings_json)
            .map_err(|e| format!("Failed to parse settings: {}", e))?;

        settings.user_id = user_id.to_string();
        settings.updated_at = chrono::Utc::now().to_rfc3339();

        self.save_settings(user_id, &settings).await?;

        Ok(settings)
    }

    async fn save_settings(&self, user_id: &str, settings: &DisplaySettings) -> Result<(), String> {
        fs::create_dir_all(&self.config_dir)
            .await
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let config_file = self.config_dir.join(format!("{}_display.json", user_id));
        let settings_json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&config_file, settings_json)
            .await
            .map_err(|e| format!("Failed to write settings: {}", e))
    }
}
