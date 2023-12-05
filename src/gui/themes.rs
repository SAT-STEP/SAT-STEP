use egui::Color32;

pub struct Theme {
    pub dark_mode: bool,
    pub text_color: Color32,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            dark_mode: true,
            text_color: Color32::GRAY,
        }
    }
}
