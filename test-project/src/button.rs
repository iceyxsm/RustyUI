// Test UI component for hot reload demonstration

pub struct Button {
    pub text: String,
    pub clicked: bool,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            clicked: false,
        }
    }
    
    pub fn click(&mut self) {
        self.clicked = true;
        println!("Button '{}' was clicked!", self.text);
    }
    
    pub fn render(&self) {
        println!("Rendering button: {} (clicked: {})", self.text, self.clicked);
    }
}