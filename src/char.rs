#[derive(Debug, Clone, Copy)]
pub enum CharColorLayer {
    Foreground,
    Background
}

pub enum AnsiColorMode {
    Ansi256,
    AnsiTrueColor
}

#[derive(Debug, Clone, Copy)]
pub struct CharColor {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl CharColor {
    pub fn to_ansi(&self, layer: &CharColorLayer, mode: &AnsiColorMode) -> String {
        let layer_str = match layer {
            CharColorLayer::Foreground => "38",
            CharColorLayer::Background => "48",
        };

        return match mode {
            AnsiColorMode::Ansi256 => format!("\x1b[{layer_str};5;{}m", self.to_ansi256()),
            AnsiColorMode::AnsiTrueColor => format!("\x1b[{layer_str};2;{};{};{}m", self.r, self.g, self.b)
        }
    }

    fn to_ansi256(&self) -> u8 {
        if self.r == self.g && self.g == self.b {
            if self.r < 8 {
                return 16
            }
    
            if self.r > 248 {
                return 231
            }
    
            return (((self.r - 8) as f32 / 247.0) * 24.0).round() as u8 + 232
        }
        
        return (16.0 + 
            36.0 * (self.r as f32 / 255.0 * 5.0).round() + 
            6.0 * (self.g as f32 / 255.0 * 5.0).round() + 
            (self.b as f32 / 255.0 * 5.0).round()) as u8
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CharInfo {
    pub char_code: char,
    pub fg_color: Option<CharColor>,
    pub bg_color: Option<CharColor>
}

impl CharInfo {
    pub fn default() -> Self {
        return Self {
            char_code: ' ',
            fg_color: None,
            bg_color: None
        };
    }

    pub fn to_ansi(&self, mode: &AnsiColorMode) -> String {
        let mut str = String::with_capacity(45);
        str.push_str("\x1b[0m");

        if self.fg_color.is_none() && self.bg_color.is_none() {
            str.push(' ');
            return str;
        }

        if let Some(fg_color) = &self.fg_color {
            str.push_str(&fg_color.to_ansi(&CharColorLayer::Foreground, mode));
        }
    
        if let Some(bg_color) = &self.bg_color {
            str.push_str(&bg_color.to_ansi(&CharColorLayer::Background, mode));
        }

        str.push(self.char_code);
        
        str
    }
}