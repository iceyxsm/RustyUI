//! Simple test for JIT compiler functionality

#[cfg(test)]
mod tests {
    use crate::{JITCompiler, Result};

    #[test]
    fn test_jit_compiler_creation() -> Result<()> {
        let mut jit = JITCompiler::new()?;
        jit.initialize()?;
        
        // Test simple function compilation
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = jit.compile_and_execute(code)?;
        
        assert!(result.success);
        assert!(result.execution_time.as_millis() < 1000); // Should be fast
        
        Ok(())
    }

    #[test]
    fn test_jit_cache() -> Result<()> {
        let mut jit = JITCompiler::new()?;
        jit.initialize()?;
        
        let code = "fn test() -> i32 { 42 }";
        
        // First compilation
        let result1 = jit.compile_and_execute(code)?;
        assert!(result1.success);
        
        // Second compilation should hit cache
        let result2 = jit.compile_and_execute(code)?;
        assert!(result2.success);
        
        // Check cache statistics
        let stats = jit.get_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        
        Ok(())
    }
}