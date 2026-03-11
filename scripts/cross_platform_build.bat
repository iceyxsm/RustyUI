@echo off
REM Cross-platform build script for RustyUI (Windows version)
REM Builds and tests RustyUI on Windows with proper feature flags

setlocal enabledelayedexpansion

echo [INFO] Starting RustyUI cross-platform build process...

REM Detect platform (Windows)
set PLATFORM=windows
echo [INFO] Detected platform: %PLATFORM%

REM Check dependencies
echo [INFO] Checking dependencies...

where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust toolchain not found. Please install Rust.
    exit /b 1
)

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Cargo not found. Please install Cargo.
    exit /b 1
)

echo [SUCCESS] Dependencies check completed

REM Set Windows-specific features
set FEATURES=windows-native,dev-ui
echo [INFO] Building with features: %FEATURES%

REM Build in development mode
echo [INFO] Building development mode...
cargo build --features %FEATURES%
if %errorlevel% neq 0 (
    echo [ERROR] Development build failed
    exit /b 1
)
echo [SUCCESS] Development build completed

REM Build in release mode
echo [INFO] Building release mode...
cargo build --release --features %FEATURES%
if %errorlevel% neq 0 (
    echo [ERROR] Release build failed
    exit /b 1
)
echo [SUCCESS] Release build completed

REM Run tests
echo [INFO] Running tests with platform-specific features...
cargo test --features %FEATURES%
if %errorlevel% neq 0 (
    echo [ERROR] Unit tests failed
    exit /b 1
)
echo [SUCCESS] Unit tests passed

REM Build CLI binary
echo [INFO] Building CLI binary...
cargo build --bin rustyui --features %FEATURES%
if %errorlevel% neq 0 (
    echo [ERROR] CLI binary build failed
    exit /b 1
)
echo [SUCCESS] CLI binary built successfully

REM Test CLI functionality
echo [INFO] Testing CLI functionality...
target\debug\rustyui.exe --version
if %errorlevel% neq 0 (
    echo [ERROR] CLI version check failed
    exit /b 1
)
echo [SUCCESS] CLI version check passed

target\debug\rustyui.exe --help >nul
if %errorlevel% neq 0 (
    echo [ERROR] CLI help check failed
    exit /b 1
)
echo [SUCCESS] CLI help check passed

REM Generate platform report
echo [INFO] Generating platform compatibility report...
set REPORT_FILE=platform_report_windows_%date:~-4,4%%date:~-10,2%%date:~-7,2%_%time:~0,2%%time:~3,2%%time:~6,2%.txt
set REPORT_FILE=%REPORT_FILE: =0%

(
echo RustyUI Platform Compatibility Report
echo ====================================
echo Generated: %date% %time%
echo Platform: %PLATFORM%
echo Architecture: %PROCESSOR_ARCHITECTURE%
echo.
echo Rust Information:
rustc --version
cargo --version
echo.
echo Platform Features:
echo - Features used: %FEATURES%
echo.
echo Build Results:
if exist target\debug\rustyui.exe (
    echo - Development build: SUCCESS
) else (
    echo - Development build: FAILED
)
if exist target\release\rustyui.exe (
    echo - Release build: SUCCESS
) else (
    echo - Release build: FAILED
)
echo.
echo Test Results:
echo - Unit tests: PASSED
echo.
echo Binary Information:
if exist target\release\rustyui.exe (
    for %%A in (target\release\rustyui.exe) do echo - Release binary size: %%~zA bytes
)
echo.
echo System Information:
systeminfo | findstr /C:"OS Name" /C:"OS Version" /C:"System Type"
) > %REPORT_FILE%

echo [SUCCESS] Platform report generated: %REPORT_FILE%

echo [SUCCESS] Cross-platform build process completed successfully!
echo [INFO] Platform: %PLATFORM%
echo [INFO] Features: %FEATURES%
echo [INFO] Report: %REPORT_FILE%

endlocal