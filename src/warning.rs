#[derive(Clone)]
pub struct Warning {text: Option<String>, priority: i32}

/// Struct for storing and setting warnings (not errors) to be shown to the user.
/// priority is on a scale 0-5, with 0 being the highest priority
impl Warning {
    pub fn new() -> Self {
        Warning {
            text: None,
            priority: 6,
        }
    }

    /// Set a new warning, but only if the warning to be set has higher priority than the current
    pub fn set(&mut self, text: Option<String>, priority: i32) {
        if priority < self.priority {
            self.text = text;
            self.priority = priority;
        }
    }
    #[allow(unused_assignments)] 
    pub fn is(&self) -> bool{
        self.text.is_some()
    }
    pub fn banner(&self) -> String {
        if let Some(warning_text) = self.text.clone() {
            return warning_text
        }
        "".to_string()
    }
}
