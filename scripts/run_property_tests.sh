#!/bin/bash
# Property-Based Test Runner for RustyUI
# 
# This script runs all property-based tests across the RustyUI workspace
# with appropriate feature flags and test configurations.

set -e

echo "🧪 Running RustyUI Property-Based Tests"
echo "======================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
PROPTEST_CASES=${PROPTEST_CASES:-100}
PROPTEST_MAX_SHRINK_ITERS=${PROPTEST_MAX_SHRINK_ITERS:-10000}

echo -e "${BLUE}Configuration:${NC}"
echo "  - Property test cases: $PROPTEST_CASES"
echo "  - Max shrink iterations: $PROPTEST_MAX_SHRINK_ITERS"
echo ""

# Export environment variables for proptest
export PROPTEST_CASES
export PROPTEST_MAX_SHRINK_ITERS

# Function to run tests for a specific crate
run_crate_tests() {
    local crate_name=$1
    local features=$2
    local test_name=$3
    
    echo -e "${YELLOW}Testing $crate_name with features: $features${NC}"
    
    if [ -n "$features" ]; then
        cargo test --package "$crate_name" --features "$features" --lib property_tests -- --nocapture
    else
        cargo test --package "$crate_name" --lib property_tests -- --nocapture
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $test_name passed${NC}"
    else
        echo -e "${RED}❌ $test_name failed${NC}"
        return 1
    fi
    echo ""
}

# Function to run specific property test
run_specific_property() {
    local crate_name=$1
    local features=$2
    local property_name=$3
    local description=$4
    
    echo -e "${BLUE}Running Property: $description${NC}"
    
    if [ -n "$features" ]; then
        cargo test --package "$crate_name" --features "$features" --lib "$property_name" -- --nocapture
    else
        cargo test --package "$crate_name" --lib "$property_name" -- --nocapture
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $description passed${NC}"
    else
        echo -e "${RED}❌ $description failed${NC}"
        return 1
    fi
    echo ""
}

# Main test execution
main() {
    echo -e "${BLUE}Starting Property-Based Test Suite${NC}"
    echo ""
    
    # Test rustyui-core (development mode)
    echo -e "${YELLOW}=== RustyUI Core Tests (Development Mode) ===${NC}"
    run_crate_tests "rustyui-core" "dev-ui" "Core Development Mode"
    
    # Test rustyui-core (production mode)
    echo -e "${YELLOW}=== RustyUI Core Tests (Production Mode) ===${NC}"
    run_crate_tests "rustyui-core" "" "Core Production Mode"
    
    # Test rustyui-interpreter (development mode only)
    echo -e "${YELLOW}=== RustyUI Interpreter Tests ===${NC}"
    run_crate_tests "rustyui-interpreter" "dev-ui" "Interpreter Development Mode"
    
    # Test rustyui-adapters
    echo -e "${YELLOW}=== RustyUI Adapters Tests ===${NC}"
    run_crate_tests "rustyui-adapters" "dev-ui" "Adapters Development Mode"
    run_crate_tests "rustyui-adapters" "" "Adapters Production Mode"
    
    # Test rustyui-cli
    echo -e "${YELLOW}=== RustyUI CLI Tests ===${NC}"
    run_crate_tests "rustyui-cli" "dev-ui" "CLI Development Mode"
    
    # Test rustyui-macros
    echo -e "${YELLOW}=== RustyUI Macros Tests ===${NC}"
    run_crate_tests "rustyui-macros" "" "Macros"
    
    echo -e "${GREEN}🎉 All Property-Based Tests Completed Successfully!${NC}"
    echo ""
    
    # Run specific critical properties
    echo -e "${BLUE}=== Running Critical Properties ===${NC}"
    
    run_specific_property "rustyui-core" "dev-ui" "property_dual_mode_operation" "Dual-Mode Operation"
    run_specific_property "rustyui-core" "" "property_zero_overhead_production_builds" "Zero-Overhead Production"
    run_specific_property "rustyui-core" "dev-ui" "property_framework_agnostic_integration" "Framework-Agnostic Integration"
    run_specific_property "rustyui-core" "dev-ui" "property_error_recovery_and_isolation" "Error Recovery & Isolation"
    run_specific_property "rustyui-core" "dev-ui" "property_performance_bounds_compliance" "Performance Bounds"
    run_specific_property "rustyui-core" "dev-ui" "property_cross_platform_compatibility" "Cross-Platform Compatibility"
    run_specific_property "rustyui-core" "dev-ui" "property_conditional_compilation_correctness" "Conditional Compilation"
    
    run_specific_property "rustyui-interpreter" "dev-ui" "property_runtime_interpretation_performance" "Runtime Interpretation Performance"
    run_specific_property "rustyui-interpreter" "dev-ui" "property_safe_runtime_code_evaluation" "Safe Runtime Code Evaluation"
    run_specific_property "rustyui-interpreter" "dev-ui" "property_runtime_interpretation_scope" "Runtime Interpretation Scope"
    
    run_specific_property "rustyui-adapters" "dev-ui" "property_framework_agnostic_integration" "Adapter Framework Integration"
    run_specific_property "rustyui-adapters" "dev-ui" "property_runtime_update_handling" "Runtime Update Handling"
    run_specific_property "rustyui-adapters" "dev-ui" "property_adapter_error_handling" "Adapter Error Handling"
    
    echo -e "${GREEN}🚀 All Critical Properties Validated!${NC}"
}

# Performance benchmarking
run_performance_benchmarks() {
    echo -e "${BLUE}=== Performance Benchmarks ===${NC}"
    
    # Run criterion benchmarks if available
    if cargo bench --help >/dev/null 2>&1; then
        echo "Running performance benchmarks..."
        cargo bench --features dev-ui
    else
        echo "Criterion benchmarks not available, skipping..."
    fi
}

# Memory usage analysis
analyze_memory_usage() {
    echo -e "${BLUE}=== Memory Usage Analysis ===${NC}"
    
    # Run tests with memory profiling if valgrind is available
    if command -v valgrind >/dev/null 2>&1; then
        echo "Running memory analysis with valgrind..."
        # This would run specific memory-intensive tests
        echo "Memory analysis would run here (placeholder)"
    else
        echo "Valgrind not available, skipping memory analysis..."
    fi
}

# Cross-platform testing
run_cross_platform_tests() {
    echo -e "${BLUE}=== Cross-Platform Tests ===${NC}"
    
    echo "Current platform: $(uname -s) $(uname -m)"
    
    # Run platform-specific property tests
    run_specific_property "rustyui-core" "dev-ui" "property_cross_platform_compatibility" "Cross-Platform Compatibility"
    
    # Test JIT compilation support
    if [[ "$(uname -m)" == "x86_64" || "$(uname -m)" == "aarch64" ]]; then
        echo "JIT compilation should be supported on this platform"
        run_specific_property "rustyui-interpreter" "dev-ui" "property_runtime_interpretation_performance" "JIT Performance"
    else
        echo "JIT compilation may not be supported on this platform"
    fi
}

# Test coverage analysis
analyze_test_coverage() {
    echo -e "${BLUE}=== Test Coverage Analysis ===${NC}"
    
    # Run coverage analysis if tarpaulin is available
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        echo "Running test coverage analysis..."
        cargo tarpaulin --features dev-ui --out Html --output-dir target/coverage
        echo "Coverage report generated in target/coverage/"
    else
        echo "cargo-tarpaulin not available, skipping coverage analysis..."
        echo "Install with: cargo install cargo-tarpaulin"
    fi
}

# Help function
show_help() {
    echo "RustyUI Property-Based Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help              Show this help message"
    echo "  --performance       Run performance benchmarks"
    echo "  --memory            Analyze memory usage"
    echo "  --cross-platform    Run cross-platform tests"
    echo "  --coverage          Analyze test coverage"
    echo "  --all               Run all tests and analyses"
    echo ""
    echo "Environment Variables:"
    echo "  PROPTEST_CASES              Number of test cases per property (default: 100)"
    echo "  PROPTEST_MAX_SHRINK_ITERS   Maximum shrink iterations (default: 10000)"
    echo ""
    echo "Examples:"
    echo "  $0                          # Run basic property tests"
    echo "  $0 --all                    # Run all tests and analyses"
    echo "  PROPTEST_CASES=1000 $0      # Run with 1000 test cases per property"
}

# Parse command line arguments
case "${1:-}" in
    --help)
        show_help
        exit 0
        ;;
    --performance)
        run_performance_benchmarks
        exit 0
        ;;
    --memory)
        analyze_memory_usage
        exit 0
        ;;
    --cross-platform)
        run_cross_platform_tests
        exit 0
        ;;
    --coverage)
        analyze_test_coverage
        exit 0
        ;;
    --all)
        main
        run_performance_benchmarks
        analyze_memory_usage
        run_cross_platform_tests
        analyze_test_coverage
        exit 0
        ;;
    "")
        main
        exit 0
        ;;
    *)
        echo "Unknown option: $1"
        show_help
        exit 1
        ;;
esac