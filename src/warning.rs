/// Struct for storing and setting warnings (not errors) to be shown to the user.
/// priority is on a scale 0-5, with 0 being the highest priority. Warning is cleared in gui.rs fn update
#[derive(Clone)]
pub struct Warning {
    text: Option<String>,
    priority: i32,
}

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
    pub fn is(&self) -> bool {
        self.text.is_some()
    }
    pub fn banner(&self) -> String {
        if let Some(warning_text) = self.text.clone() {
            return warning_text;
        }
        "".to_string()
    }
}

impl Default for Warning {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_warning() {
        let mut warning = Warning::new();

        warning.set(Some("Warning".to_string()), 3);

        assert_eq!(warning.priority, 3);

        warning.set(Some("Other warning".to_string()), 1);

        assert_eq!(warning.priority, 1);

        warning.set(Some("Third warning".to_string()), 3);

        assert_eq!(warning.priority, 1)
    }
}
