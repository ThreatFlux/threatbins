use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_author: Option<String>,
    pub default_output_dir: Option<PathBuf>,
    pub template_dir: Option<PathBuf>,
    pub default_format: Option<String>,
    pub style_preferences: StylePreferences,
    pub conversion_settings: ConversionSettings,
    pub security_settings: SecuritySettings,
    pub performance_settings: PerformanceSettings,
    pub custom_templates: HashMap<String, TemplateConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreferences {
    pub font_family: String,
    pub font_size: f64,
    pub line_height: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub header_color: String,
    pub text_color: String,
    pub background_color: String,
    pub accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionSettings {
    pub preserve_formatting: bool,
    pub auto_detect_tables: bool,
    pub auto_detect_lists: bool,
    pub include_metadata: bool,
    pub optimize_images: bool,
    pub compress_output: bool,
    pub validate_output: bool,
    pub backup_originals: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub default_encryption: Option<String>,
    pub require_passwords: bool,
    pub restrict_copying: bool,
    pub restrict_editing: bool,
    pub allow_accessibility: bool,
    pub digital_signatures: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub parallel_workers: usize,
    pub memory_limit_mb: usize,
    pub cache_enabled: bool,
    pub cache_size_mb: usize,
    pub optimize_large_files: bool,
    pub progress_reporting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
    pub document_type: String,
    pub variables: HashMap<String, String>,
    pub styles: HashMap<String, String>,
    pub features: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_author: None,
            default_output_dir: None,
            template_dir: Some(PathBuf::from("templates")),
            default_format: Some("pdf".to_string()),
            style_preferences: StylePreferences::default(),
            conversion_settings: ConversionSettings::default(),
            security_settings: SecuritySettings::default(),
            performance_settings: PerformanceSettings::default(),
            custom_templates: HashMap::new(),
        }
    }
}

impl Default for StylePreferences {
    fn default() -> Self {
        StylePreferences {
            font_family: "Helvetica".to_string(),
            font_size: 11.0,
            line_height: 1.2,
            margin_top: 72.0,    // 1 inch
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            header_color: "#2E4A9C".to_string(), // Professional blue
            text_color: "#000000".to_string(),   // Black
            background_color: "#FFFFFF".to_string(), // White
            accent_color: "#0078D4".to_string(), // Microsoft blue
        }
    }
}

impl Default for ConversionSettings {
    fn default() -> Self {
        ConversionSettings {
            preserve_formatting: true,
            auto_detect_tables: true,
            auto_detect_lists: true,
            include_metadata: true,
            optimize_images: true,
            compress_output: false,
            validate_output: true,
            backup_originals: false,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        SecuritySettings {
            default_encryption: None,
            require_passwords: false,
            restrict_copying: false,
            restrict_editing: false,
            allow_accessibility: true,
            digital_signatures: false,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        PerformanceSettings {
            parallel_workers: 4,
            memory_limit_mb: 1024,
            cache_enabled: true,
            cache_size_mb: 256,
            optimize_large_files: true,
            progress_reporting: true,
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = if let Some(path) = config_path {
            path.to_path_buf()
        } else {
            // Try standard locations
            let home_config = dirs::home_dir()
                .map(|home| home.join(".config").join("office-cli").join("config.json"));

            let local_config = PathBuf::from("office-cli.json");

            if local_config.exists() {
                local_config
            } else if let Some(home_path) = home_config {
                if home_path.exists() {
                    home_path
                } else {
                    return Ok(Config::default());
                }
            } else {
                return Ok(Config::default());
            }
        };

        if config_file.exists() {
            let content = fs::read_to_string(&config_file)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, config_path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
        let config_file = if let Some(path) = config_path {
            path.to_path_buf()
        } else {
            // Save to home directory
            let home_config = dirs::home_dir()
                .map(|home| home.join(".config").join("office-cli"))
                .ok_or("Could not determine home directory")?;

            fs::create_dir_all(&home_config)?;
            home_config.join("config.json")
        };

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_file, content)?;

        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "default_author" => self.default_author.clone(),
            "default_format" => self.default_format.clone(),
            "font_family" => Some(self.style_preferences.font_family.clone()),
            "font_size" => Some(self.style_preferences.font_size.to_string()),
            "parallel_workers" => Some(self.performance_settings.parallel_workers.to_string()),
            _ => None,
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            "default_author" => {
                self.default_author = if value.is_empty() { None } else { Some(value.to_string()) };
            },
            "default_format" => {
                self.default_format = if value.is_empty() { None } else { Some(value.to_string()) };
            },
            "font_family" => {
                self.style_preferences.font_family = value.to_string();
            },
            "font_size" => {
                self.style_preferences.font_size = value.parse()?;
            },
            "parallel_workers" => {
                self.performance_settings.parallel_workers = value.parse()?;
            },
            _ => {
                return Err(format!("Unknown configuration key: {}", key).into());
            }
        }

        Ok(())
    }
}

pub struct ConfigManager {
    current_config: Config,
}

impl ConfigManager {
    pub fn new() -> Self {
        ConfigManager {
            current_config: Config::default(),
        }
    }

    pub fn show_config(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Current Configuration:");
        println!();

        println!("🎯 General Settings:");
        println!("   Default Author: {}", config.default_author.as_deref().unwrap_or("Not set"));
        println!("   Default Format: {}", config.default_format.as_deref().unwrap_or("pdf"));
        println!("   Template Dir: {}", config.template_dir.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "Not set".to_string()));

        println!("\n🎨 Style Preferences:");
        println!("   Font Family: {}", config.style_preferences.font_family);
        println!("   Font Size: {}", config.style_preferences.font_size);
        println!("   Line Height: {}", config.style_preferences.line_height);
        println!("   Header Color: {}", config.style_preferences.header_color);
        println!("   Accent Color: {}", config.style_preferences.accent_color);

        println!("\n🔄 Conversion Settings:");
        println!("   Preserve Formatting: {}", config.conversion_settings.preserve_formatting);
        println!("   Auto-detect Tables: {}", config.conversion_settings.auto_detect_tables);
        println!("   Auto-detect Lists: {}", config.conversion_settings.auto_detect_lists);
        println!("   Include Metadata: {}", config.conversion_settings.include_metadata);
        println!("   Optimize Images: {}", config.conversion_settings.optimize_images);
        println!("   Validate Output: {}", config.conversion_settings.validate_output);

        println!("\n🔒 Security Settings:");
        println!("   Default Encryption: {}", config.security_settings.default_encryption.as_deref().unwrap_or("None"));
        println!("   Require Passwords: {}", config.security_settings.require_passwords);
        println!("   Restrict Copying: {}", config.security_settings.restrict_copying);
        println!("   Restrict Editing: {}", config.security_settings.restrict_editing);

        println!("\n⚡ Performance Settings:");
        println!("   Parallel Workers: {}", config.performance_settings.parallel_workers);
        println!("   Memory Limit: {} MB", config.performance_settings.memory_limit_mb);
        println!("   Cache Enabled: {}", config.performance_settings.cache_enabled);
        println!("   Cache Size: {} MB", config.performance_settings.cache_size_mb);

        Ok(())
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.current_config.set_value(key, value)?;
        self.current_config.save(None)?;
        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.current_config.get_value(key)
            .ok_or_else(|| format!("Configuration key '{}' not found", key).into())
    }

    pub fn reset_to_defaults(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_config = Config::default();
        self.current_config.save(None)?;
        Ok(())
    }

    pub fn edit_interactive(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use dialoguer::{Input, Confirm, Select};

        println!("🔧 Interactive Configuration Editor");
        println!();

        // Edit basic settings
        let current_author = self.current_config.default_author.as_deref().unwrap_or("");
        let new_author: String = Input::new()
            .with_prompt("Default Author")
            .default(current_author.to_string())
            .allow_empty(true)
            .interact_text()?;

        if !new_author.is_empty() {
            self.current_config.default_author = Some(new_author);
        } else {
            self.current_config.default_author = None;
        }

        // Edit style preferences
        let edit_styles = Confirm::new()
            .with_prompt("Edit style preferences?")
            .default(false)
            .interact()?;

        if edit_styles {
            let font_family: String = Input::new()
                .with_prompt("Font Family")
                .default(self.current_config.style_preferences.font_family.clone())
                .interact_text()?;
            self.current_config.style_preferences.font_family = font_family;

            let font_size: f64 = Input::new()
                .with_prompt("Font Size")
                .default(self.current_config.style_preferences.font_size)
                .interact_text()?;
            self.current_config.style_preferences.font_size = font_size;
        }

        // Edit performance settings
        let edit_performance = Confirm::new()
            .with_prompt("Edit performance settings?")
            .default(false)
            .interact()?;

        if edit_performance {
            let workers: usize = Input::new()
                .with_prompt("Parallel Workers")
                .default(self.current_config.performance_settings.parallel_workers)
                .interact_text()?;
            self.current_config.performance_settings.parallel_workers = workers;

            let memory_limit: usize = Input::new()
                .with_prompt("Memory Limit (MB)")
                .default(self.current_config.performance_settings.memory_limit_mb)
                .interact_text()?;
            self.current_config.performance_settings.memory_limit_mb = memory_limit;
        }

        // Save configuration
        self.current_config.save(None)?;

        println!("✅ Configuration saved successfully!");

        Ok(())
    }
}