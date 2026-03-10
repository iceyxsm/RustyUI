use rustyui_core::{DualModeEngine, DualModeConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test project for RustyUI development mode");
    
    let config = DualModeConfig::default();
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("Engine initialized successfully!");
    
    Ok(())
}