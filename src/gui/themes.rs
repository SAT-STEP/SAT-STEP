use std::include_bytes;

pub struct Theme {
    pub url: &[u8]
}

impl Theme{
    pub fn new(url: &[u8]) -> Self {
        Self {
            url
        }
    }
}
pub static DARK_MODE:Theme = Theme::new(include_bytes!("../../assets/half-moon.svg"));
pub static LIGHT_MODE:Theme = Theme::new(include_bytes!("../../assets/sun-light.svg"));
