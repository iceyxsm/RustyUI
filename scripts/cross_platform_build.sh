#!/bin/bash
# Cross-platform build script for RustyUI
# Builds and tests RustyUI on multiple platforms with proper feature flags

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect current platform
detect_platform() {
    case "$(uname -s)" in
        Linux*)     PLATFORM=linux;;
        Darwin*)    PLATFORM=macos;;
        CYGWIN*|MINGW*|MSYS*) PLATFORM=windows;;
        *)          PLATFORM=unknown;;
    esac
    print_status "Detected platform: $PLATFORM"
}

# Check dependencies
check_dependencies() {
    print_status "Checking dependencies..."
    
    # Check Rust toolchain
    if ! command -v rustc &> /dev/null; then
        print_error "Rust toolchain not found. Please install Rust."
        exit 1
    fi
    
    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Cargo."
        exit 1
    fi
    
    # Check platform-specific dependencies
    case $PLATFORM in
        linux)
            # Check for development packages on Linux
            if ! pkg-config --exists libc; then
                print_warning "libc development packages may be missing"
            fi
            ;;
        macos)
            # Check for Xcode command line tools
            if ! xcode-select -p &> /dev/null; then
                print_warning "Xcode command line tools may be missing"
            fi
            ;;
        windows)
            # Check for Visual Studio Build Tools or similar
            print_status "Windows detected - ensure Visual Studio Build Tools are installed"
            ;;
    esac
    
    print_success "Dependencies check completed"
}

# Build with platform-specific features
build_platform_specific() {
    print_status "Building with platform-specific features..."
    
    case $PLATFORM in
        linux)
            FEATURES="unix-native,dev-ui"
            ;;
        macos)
            FEATURES="macos-native,dev-ui"
            ;;
        windows)
            FEATURES="windows-native,dev-ui"
            ;;
        *)
            FEATURES="dev-ui"
            print_warning "Unknown platform, using default features"
            ;;
    esac
    
    print_status "Building with features: $FEATURES"
    
    # Build in development mode
    cargo build --features "$FEATURES"
    if [ $? -eq 0 ]; then
        print_success "Development build completed"
    else
        print_error "Development build failed"
        exit 1
    fi
    
    # Build in release mode
    cargo build --release --features "$FEATURES"
    if [ $? -eq 0 ]; then
        print_success "Release build completed"
    else
        print_error "Release build failed"
        exit 1
    fi
}

# Test with platform-specific features
test_platform_specific() {
    print_status "Running tests with platform-specific features..."
    
    # Run unit tests
    cargo test --features "$FEATURES"
    if [ $? -eq 0 ]; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        exit 1
    fi
    
    # Run integration tests
    cargo test --features "$FEATURES" --test '*'
    if [ $? -eq 0 ]; then
        print_success "Integration tests passed"
    else
        print_warning "Some integration tests failed (may be expected on some platforms)"
    fi
}

# Test cross-compilation (if supported)
test_cross_compilation() {
    print_status "Testing cross-compilation capabilities..."
    
    # List of common targets to test
    TARGETS=()
    
    case $PLATFORM in
        linux)
            TARGETS+=("x86_64-pc-windows-gnu")
            TARGETS+=("x86_64-apple-darwin")
            ;;
        macos)
            TARGETS+=("x86_64-pc-windows-msvc")
            TARGETS+=("x86_64-unknown-linux-gnu")
            ;;
        windows)
            TARGETS+=("x86_64-unknown-linux-gnu")
            TARGETS+=("x86_64-apple-darwin")
            ;;
    esac
    
    for target in "${TARGETS[@]}"; do
        print_status "Testing cross-compilation for $target..."
        
        # Check if target is installed
        if rustup target list --installed | grep -q "$target"; then
            # Try to build (may fail due to missing system libraries, which is expected)
            cargo build --target "$target" --features "dev-ui" 2>/dev/null
            if [ $? -eq 0 ]; then
                print_success "Cross-compilation for $target succeeded"
            else
                print_warning "Cross-compilation for $target failed (may need system libraries)"
            fi
        else
            print_warning "Target $target not installed, skipping"
        fi
    done
}

# Build CLI binary
build_cli() {
    print_status "Building CLI binary..."
    
    cargo build --bin rustyui --features "$FEATURES"
    if [ $? -eq 0 ]; then
        print_success "CLI binary built successfully"
        
        # Test CLI functionality
        print_status "Testing CLI functionality..."
        ./target/debug/rustyui --version
        if [ $? -eq 0 ]; then
            print_success "CLI version check passed"
        else
            print_error "CLI version check failed"
        fi
        
        ./target/debug/rustyui --help > /dev/null
        if [ $? -eq 0 ]; then
            print_success "CLI help check passed"
        else
            print_error "CLI help check failed"
        fi
    else
        print_error "CLI binary build failed"
        exit 1
    fi
}

# Run benchmarks (if available)
run_benchmarks() {
    print_status "Running benchmarks..."
    
    if cargo bench --features "$FEATURES" 2>/dev/null; then
        print_success "Benchmarks completed"
    else
        print_warning "Benchmarks not available or failed"
    fi
}

# Generate platform report
generate_platform_report() {
    print_status "Generating platform compatibility report..."
    
    REPORT_FILE="platform_report_${PLATFORM}_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "RustyUI Platform Compatibility Report"
        echo "===================================="
        echo "Generated: $(date)"
        echo "Platform: $PLATFORM"
        echo "Architecture: $(uname -m)"
        echo ""
        
        echo "Rust Information:"
        rustc --version
        cargo --version
        echo ""
        
        echo "Platform Features:"
        echo "- Features used: $FEATURES"
        echo ""
        
        echo "Build Results:"
        echo "- Development build: $([ -f target/debug/rustyui ] && echo "SUCCESS" || echo "FAILED")"
        echo "- Release build: $([ -f target/release/rustyui ] && echo "SUCCESS" || echo "FAILED")"
        echo ""
        
        echo "Test Results:"
        echo "- Unit tests: $(cargo test --features "$FEATURES" --quiet 2>/dev/null && echo "PASSED" || echo "FAILED")"
        echo ""
        
        echo "Binary Information:"
        if [ -f target/release/rustyui ]; then
            echo "- Release binary size: $(du -h target/release/rustyui | cut -f1)"
            echo "- Release binary info:"
            file target/release/rustyui 2>/dev/null || echo "  File command not available"
        fi
        
        echo ""
        echo "System Information:"
        uname -a
        
    } > "$REPORT_FILE"
    
    print_success "Platform report generated: $REPORT_FILE"
}

# Main execution
main() {
    print_status "Starting RustyUI cross-platform build process..."
    
    detect_platform
    check_dependencies
    build_platform_specific
    test_platform_specific
    build_cli
    
    # Optional steps
    if [ "$1" = "--full" ]; then
        test_cross_compilation
        run_benchmarks
    fi
    
    generate_platform_report
    
    print_success "Cross-platform build process completed successfully!"
    print_status "Platform: $PLATFORM"
    print_status "Features: $FEATURES"
    print_status "Report: $REPORT_FILE"
}

# Run main function with all arguments
main "$@"