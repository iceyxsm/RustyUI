#!/bin/bash
# Benchmark runner for RustyUI performance testing
# 
# This script runs comprehensive benchmarks comparing production vs development performance

set -e

echo "🚀 Running RustyUI Performance Benchmarks"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run benchmarks for a specific crate
run_crate_benchmarks() {
    local crate_name=$1
    local features=$2
    
    echo -e "${YELLOW}Running benchmarks for $crate_name${NC}"
    
    if [ -n "$features" ]; then
        echo "  Features: $features"
        cargo bench --package "$crate_name" --features "$features"
    else
        echo "  Features: none (production mode)"
        cargo bench --package "$crate_name"
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ $crate_name benchmarks completed${NC}"
    else
        echo -e "${RED}✗ $crate_name benchmarks failed${NC}"
        return 1
    fi
    echo ""
}

# Main benchmark execution
main() {
    echo -e "${BLUE}Starting comprehensive performance benchmarks${NC}"
    echo ""
    
    # Core benchmarks (production mode)
    echo -e "${YELLOW}=== Production Mode Benchmarks ===${NC}"
    run_crate_benchmarks "rustyui-core" ""
    
    # Core benchmarks (development mode)
    echo -e "${YELLOW}=== Development Mode Benchmarks ===${NC}"
    run_crate_benchmarks "rustyui-core" "dev-ui"
    
    # Interpreter benchmarks (development only)
    echo -e "${YELLOW}=== Interpreter Performance Benchmarks ===${NC}"
    run_crate_benchmarks "rustyui-interpreter" "dev-ui"
    
    echo -e "${GREEN}SUCCESS: All benchmarks completed!${NC}"
    echo ""
    echo "📊 Benchmark results are saved in target/criterion/"
    echo "🌐 Open target/criterion/report/index.html to view detailed results"
}

# Help function
show_help() {
    echo "RustyUI Performance Benchmark Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help              Show this help message"
    echo "  --core-only         Run only core benchmarks"
    echo "  --interpreter-only  Run only interpreter benchmarks"
    echo "  --production-only   Run only production mode benchmarks"
    echo "  --development-only  Run only development mode benchmarks"
    echo ""
    echo "Examples:"
    echo "  $0                      # Run all benchmarks"
    echo "  $0 --core-only          # Run only core benchmarks"
    echo "  $0 --production-only    # Run only production benchmarks"
}

# Parse command line arguments
case "${1:-}" in
    --help)
        show_help
        exit 0
        ;;
    --core-only)
        echo -e "${BLUE}Running core benchmarks only${NC}"
        run_crate_benchmarks "rustyui-core" ""
        run_crate_benchmarks "rustyui-core" "dev-ui"
        exit 0
        ;;
    --interpreter-only)
        echo -e "${BLUE}Running interpreter benchmarks only${NC}"
        run_crate_benchmarks "rustyui-interpreter" "dev-ui"
        exit 0
        ;;
    --production-only)
        echo -e "${BLUE}Running production mode benchmarks only${NC}"
        run_crate_benchmarks "rustyui-core" ""
        exit 0
        ;;
    --development-only)
        echo -e "${BLUE}Running development mode benchmarks only${NC}"
        run_crate_benchmarks "rustyui-core" "dev-ui"
        run_crate_benchmarks "rustyui-interpreter" "dev-ui"
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