// Terminal Color Themes
// Dracula, OneDark Pro, Monokai, Solarized, and custom embedded themes

use serde::{Deserialize, Serialize};

/// Terminal color theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTheme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub selection: String,
    
    // Standard colors
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    
    // Bright colors
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
    
    // Embedded-specific highlighting
    pub register: String,
    pub address: String,
    pub pin: String,
    pub peripheral: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

/// Get available theme names
pub fn get_available_themes() -> Vec<String> {
    vec![
        "dracula".to_string(),
        "one_dark_pro".to_string(),
        "monokai".to_string(),
        "solarized_dark".to_string(),
        "solarized_light".to_string(),
        "nord".to_string(),
        "gruvbox".to_string(),
        "embedded_dark".to_string(),
    ]
}

/// Get a theme by name
pub fn get_theme(name: &str) -> TerminalTheme {
    match name.to_lowercase().as_str() {
        "dracula" => dracula_theme(),
        "one_dark_pro" | "onedarkpro" => one_dark_pro_theme(),
        "monokai" => monokai_theme(),
        "solarized_dark" => solarized_dark_theme(),
        "solarized_light" => solarized_light_theme(),
        "nord" => nord_theme(),
        "gruvbox" => gruvbox_theme(),
        "embedded_dark" | "embedded" => embedded_dark_theme(),
        _ => dracula_theme(), // Default
    }
}

/// Dracula theme - popular dark theme
fn dracula_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Dracula".to_string(),
        background: "#282a36".to_string(),
        foreground: "#f8f8f2".to_string(),
        cursor: "#f8f8f2".to_string(),
        selection: "#44475a".to_string(),
        
        black: "#21222c".to_string(),
        red: "#ff5555".to_string(),
        green: "#50fa7b".to_string(),
        yellow: "#f1fa8c".to_string(),
        blue: "#bd93f9".to_string(),
        magenta: "#ff79c6".to_string(),
        cyan: "#8be9fd".to_string(),
        white: "#f8f8f2".to_string(),
        
        bright_black: "#6272a4".to_string(),
        bright_red: "#ff6e6e".to_string(),
        bright_green: "#69ff94".to_string(),
        bright_yellow: "#ffffa5".to_string(),
        bright_blue: "#d6acff".to_string(),
        bright_magenta: "#ff92df".to_string(),
        bright_cyan: "#a4ffff".to_string(),
        bright_white: "#ffffff".to_string(),
        
        register: "#bd93f9".to_string(),
        address: "#ffb86c".to_string(),
        pin: "#50fa7b".to_string(),
        peripheral: "#8be9fd".to_string(),
        success: "#50fa7b".to_string(),
        warning: "#ffb86c".to_string(),
        error: "#ff5555".to_string(),
        info: "#8be9fd".to_string(),
    }
}

/// One Dark Pro theme - Atom/VS Code inspired
fn one_dark_pro_theme() -> TerminalTheme {
    TerminalTheme {
        name: "One Dark Pro".to_string(),
        background: "#282c34".to_string(),
        foreground: "#abb2bf".to_string(),
        cursor: "#528bff".to_string(),
        selection: "#3e4451".to_string(),
        
        black: "#282c34".to_string(),
        red: "#e06c75".to_string(),
        green: "#98c379".to_string(),
        yellow: "#e5c07b".to_string(),
        blue: "#61afef".to_string(),
        magenta: "#c678dd".to_string(),
        cyan: "#56b6c2".to_string(),
        white: "#abb2bf".to_string(),
        
        bright_black: "#5c6370".to_string(),
        bright_red: "#e06c75".to_string(),
        bright_green: "#98c379".to_string(),
        bright_yellow: "#e5c07b".to_string(),
        bright_blue: "#61afef".to_string(),
        bright_magenta: "#c678dd".to_string(),
        bright_cyan: "#56b6c2".to_string(),
        bright_white: "#ffffff".to_string(),
        
        register: "#c678dd".to_string(),
        address: "#d19a66".to_string(),
        pin: "#98c379".to_string(),
        peripheral: "#61afef".to_string(),
        success: "#98c379".to_string(),
        warning: "#e5c07b".to_string(),
        error: "#e06c75".to_string(),
        info: "#56b6c2".to_string(),
    }
}

/// Monokai theme - classic Sublime Text inspired
fn monokai_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Monokai".to_string(),
        background: "#272822".to_string(),
        foreground: "#f8f8f2".to_string(),
        cursor: "#f8f8f2".to_string(),
        selection: "#49483e".to_string(),
        
        black: "#272822".to_string(),
        red: "#f92672".to_string(),
        green: "#a6e22e".to_string(),
        yellow: "#f4bf75".to_string(),
        blue: "#66d9ef".to_string(),
        magenta: "#ae81ff".to_string(),
        cyan: "#a1efe4".to_string(),
        white: "#f8f8f2".to_string(),
        
        bright_black: "#75715e".to_string(),
        bright_red: "#f92672".to_string(),
        bright_green: "#a6e22e".to_string(),
        bright_yellow: "#f4bf75".to_string(),
        bright_blue: "#66d9ef".to_string(),
        bright_magenta: "#ae81ff".to_string(),
        bright_cyan: "#a1efe4".to_string(),
        bright_white: "#f9f8f5".to_string(),
        
        register: "#ae81ff".to_string(),
        address: "#fd971f".to_string(),
        pin: "#a6e22e".to_string(),
        peripheral: "#66d9ef".to_string(),
        success: "#a6e22e".to_string(),
        warning: "#f4bf75".to_string(),
        error: "#f92672".to_string(),
        info: "#66d9ef".to_string(),
    }
}

/// Solarized Dark theme
fn solarized_dark_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Solarized Dark".to_string(),
        background: "#002b36".to_string(),
        foreground: "#839496".to_string(),
        cursor: "#839496".to_string(),
        selection: "#073642".to_string(),
        
        black: "#073642".to_string(),
        red: "#dc322f".to_string(),
        green: "#859900".to_string(),
        yellow: "#b58900".to_string(),
        blue: "#268bd2".to_string(),
        magenta: "#d33682".to_string(),
        cyan: "#2aa198".to_string(),
        white: "#eee8d5".to_string(),
        
        bright_black: "#002b36".to_string(),
        bright_red: "#cb4b16".to_string(),
        bright_green: "#586e75".to_string(),
        bright_yellow: "#657b83".to_string(),
        bright_blue: "#839496".to_string(),
        bright_magenta: "#6c71c4".to_string(),
        bright_cyan: "#93a1a1".to_string(),
        bright_white: "#fdf6e3".to_string(),
        
        register: "#6c71c4".to_string(),
        address: "#cb4b16".to_string(),
        pin: "#859900".to_string(),
        peripheral: "#268bd2".to_string(),
        success: "#859900".to_string(),
        warning: "#b58900".to_string(),
        error: "#dc322f".to_string(),
        info: "#2aa198".to_string(),
    }
}

/// Solarized Light theme
fn solarized_light_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Solarized Light".to_string(),
        background: "#fdf6e3".to_string(),
        foreground: "#657b83".to_string(),
        cursor: "#657b83".to_string(),
        selection: "#eee8d5".to_string(),
        
        black: "#073642".to_string(),
        red: "#dc322f".to_string(),
        green: "#859900".to_string(),
        yellow: "#b58900".to_string(),
        blue: "#268bd2".to_string(),
        magenta: "#d33682".to_string(),
        cyan: "#2aa198".to_string(),
        white: "#eee8d5".to_string(),
        
        bright_black: "#002b36".to_string(),
        bright_red: "#cb4b16".to_string(),
        bright_green: "#586e75".to_string(),
        bright_yellow: "#657b83".to_string(),
        bright_blue: "#839496".to_string(),
        bright_magenta: "#6c71c4".to_string(),
        bright_cyan: "#93a1a1".to_string(),
        bright_white: "#fdf6e3".to_string(),
        
        register: "#6c71c4".to_string(),
        address: "#cb4b16".to_string(),
        pin: "#859900".to_string(),
        peripheral: "#268bd2".to_string(),
        success: "#859900".to_string(),
        warning: "#b58900".to_string(),
        error: "#dc322f".to_string(),
        info: "#2aa198".to_string(),
    }
}

/// Nord theme - Arctic inspired
fn nord_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Nord".to_string(),
        background: "#2e3440".to_string(),
        foreground: "#d8dee9".to_string(),
        cursor: "#d8dee9".to_string(),
        selection: "#434c5e".to_string(),
        
        black: "#3b4252".to_string(),
        red: "#bf616a".to_string(),
        green: "#a3be8c".to_string(),
        yellow: "#ebcb8b".to_string(),
        blue: "#81a1c1".to_string(),
        magenta: "#b48ead".to_string(),
        cyan: "#88c0d0".to_string(),
        white: "#e5e9f0".to_string(),
        
        bright_black: "#4c566a".to_string(),
        bright_red: "#bf616a".to_string(),
        bright_green: "#a3be8c".to_string(),
        bright_yellow: "#ebcb8b".to_string(),
        bright_blue: "#81a1c1".to_string(),
        bright_magenta: "#b48ead".to_string(),
        bright_cyan: "#8fbcbb".to_string(),
        bright_white: "#eceff4".to_string(),
        
        register: "#b48ead".to_string(),
        address: "#d08770".to_string(),
        pin: "#a3be8c".to_string(),
        peripheral: "#81a1c1".to_string(),
        success: "#a3be8c".to_string(),
        warning: "#ebcb8b".to_string(),
        error: "#bf616a".to_string(),
        info: "#88c0d0".to_string(),
    }
}

/// Gruvbox theme - retro groove
fn gruvbox_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Gruvbox".to_string(),
        background: "#282828".to_string(),
        foreground: "#ebdbb2".to_string(),
        cursor: "#ebdbb2".to_string(),
        selection: "#504945".to_string(),
        
        black: "#282828".to_string(),
        red: "#cc241d".to_string(),
        green: "#98971a".to_string(),
        yellow: "#d79921".to_string(),
        blue: "#458588".to_string(),
        magenta: "#b16286".to_string(),
        cyan: "#689d6a".to_string(),
        white: "#a89984".to_string(),
        
        bright_black: "#928374".to_string(),
        bright_red: "#fb4934".to_string(),
        bright_green: "#b8bb26".to_string(),
        bright_yellow: "#fabd2f".to_string(),
        bright_blue: "#83a598".to_string(),
        bright_magenta: "#d3869b".to_string(),
        bright_cyan: "#8ec07c".to_string(),
        bright_white: "#ebdbb2".to_string(),
        
        register: "#d3869b".to_string(),
        address: "#fe8019".to_string(),
        pin: "#b8bb26".to_string(),
        peripheral: "#83a598".to_string(),
        success: "#b8bb26".to_string(),
        warning: "#fabd2f".to_string(),
        error: "#fb4934".to_string(),
        info: "#8ec07c".to_string(),
    }
}

/// Custom embedded systems dark theme
fn embedded_dark_theme() -> TerminalTheme {
    TerminalTheme {
        name: "Embedded Dark".to_string(),
        background: "#0d1117".to_string(),
        foreground: "#c9d1d9".to_string(),
        cursor: "#58a6ff".to_string(),
        selection: "#264f78".to_string(),
        
        black: "#0d1117".to_string(),
        red: "#f85149".to_string(),
        green: "#3fb950".to_string(),
        yellow: "#d29922".to_string(),
        blue: "#58a6ff".to_string(),
        magenta: "#bc8cff".to_string(),
        cyan: "#39c5cf".to_string(),
        white: "#b1bac4".to_string(),
        
        bright_black: "#484f58".to_string(),
        bright_red: "#ff7b72".to_string(),
        bright_green: "#56d364".to_string(),
        bright_yellow: "#e3b341".to_string(),
        bright_blue: "#79c0ff".to_string(),
        bright_magenta: "#d2a8ff".to_string(),
        bright_cyan: "#56d4dd".to_string(),
        bright_white: "#f0f6fc".to_string(),
        
        // Special embedded colors
        register: "#bc8cff".to_string(),    // Purple for registers (RCC, GPIOA)
        address: "#ffa657".to_string(),      // Orange for addresses (0x40020000)
        pin: "#7ee787".to_string(),          // Light green for pins (PA5, PB3)
        peripheral: "#79c0ff".to_string(),   // Light blue for peripherals (USART1, SPI1)
        success: "#3fb950".to_string(),
        warning: "#d29922".to_string(),
        error: "#f85149".to_string(),
        info: "#58a6ff".to_string(),
    }
}

/// Convert theme to CSS variables
pub fn theme_to_css(theme: &TerminalTheme) -> String {
    format!(r#"
:root {{
    --term-bg: {};
    --term-fg: {};
    --term-cursor: {};
    --term-selection: {};
    --term-black: {};
    --term-red: {};
    --term-green: {};
    --term-yellow: {};
    --term-blue: {};
    --term-magenta: {};
    --term-cyan: {};
    --term-white: {};
    --term-bright-black: {};
    --term-bright-red: {};
    --term-bright-green: {};
    --term-bright-yellow: {};
    --term-bright-blue: {};
    --term-bright-magenta: {};
    --term-bright-cyan: {};
    --term-bright-white: {};
    --term-register: {};
    --term-address: {};
    --term-pin: {};
    --term-peripheral: {};
    --term-success: {};
    --term-warning: {};
    --term-error: {};
    --term-info: {};
}}
"#,
        theme.background,
        theme.foreground,
        theme.cursor,
        theme.selection,
        theme.black,
        theme.red,
        theme.green,
        theme.yellow,
        theme.blue,
        theme.magenta,
        theme.cyan,
        theme.white,
        theme.bright_black,
        theme.bright_red,
        theme.bright_green,
        theme.bright_yellow,
        theme.bright_blue,
        theme.bright_magenta,
        theme.bright_cyan,
        theme.bright_white,
        theme.register,
        theme.address,
        theme.pin,
        theme.peripheral,
        theme.success,
        theme.warning,
        theme.error,
        theme.info,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_theme() {
        let theme = get_theme("dracula");
        assert_eq!(theme.name, "Dracula");
        assert_eq!(theme.background, "#282a36");
    }

    #[test]
    fn test_theme_to_css() {
        let theme = get_theme("one_dark_pro");
        let css = theme_to_css(&theme);
        assert!(css.contains("--term-bg"));
        assert!(css.contains("#282c34"));
    }

    #[test]
    fn test_available_themes() {
        let themes = get_available_themes();
        assert!(themes.len() >= 8);
        assert!(themes.contains(&"dracula".to_string()));
    }
}
