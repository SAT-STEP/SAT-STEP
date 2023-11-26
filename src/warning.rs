#[derive(Clone)]
pub struct Warning {text: Option<String>, priority: i32}


impl Warning {
    pub fn new() -> Self {
        Warning {
            text: None,
            priority: 0,
        }
    }
    pub fn set(&mut self, text : Option<String>) {
        self.text = text;
        self.priority = 0;
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
