use egui::Color32;

pub struct Theme {
    pub dark_mode: bool,
    pub colors: Colors,
}
pub struct Colors {
    pub text_color1: Color32,
    pub text_color2: Color32,
    pub background_color1: Color32,
    pub background_color2: Color32,
}

impl Theme {
    pub fn new(dark_mode: bool) -> Self {
        let theme = Self {
            dark_mode,
            colors: Colors::default(),
        };
        Self {
            dark_mode,
            colors: theme.load(dark_mode),
        }
    }

    pub fn theme_switch(&self) -> Self {
        Self {
            dark_mode: !self.dark_mode,
            colors: self.load(!self.dark_mode),
        }
    }
    pub fn load(&self, dark_mode: bool) -> Colors {
        if dark_mode {
            Colors {
                text_color1: Color32::GRAY,
                text_color2: Color32::BLACK,
                background_color1: Color32::BLACK,
                background_color2: Color32::GRAY,
            }
        } else {
            Colors {
                text_color1: Color32::BLACK,
                text_color2: Color32::GRAY,
                background_color1: Color32::GRAY,
                background_color2: Color32::BLACK,
            }
        }
    }
}
impl Default for Colors {
    fn default() -> Colors {
        Colors {
            text_color1: Color32::GRAY,
            text_color2: Color32::GRAY,
            background_color1: Color32::BLACK,
            background_color2: Color32::BLACK,
        }
    }
}
