# C++ SDK Cross-Platform Build Guide

This document describes how to build the C++ SDK on different platforms (Linux, macOS, Windows).

## Supported Platforms

- **Linux** (x86_64) - Native and cross-compilation
- **macOS** (Universal Binary: x86_64 + ARM64) - Native compilation
- **Windows** (x86_64) - Native and cross-compilation

## Build Systems

The project supports two build systems:

1. **Make** - Platform-specific Makefiles with automatic detection
2. **CMake** - Modern cross-platform build system

## Quick Start

### Linux/macOS
```bash
# Simple build for current platform
make

# Or use the build script
./build.sh

# CMake build
./build.sh --cmake
```

### Windows
```cmd
# Simple build
mingw32-make

# Or use the build script
build.bat

# CMake build
build.bat --cmake
```

## Build Options

### Using Make

```bash
# Platform detection (automatic)
make                    # Build for current platform
make platform-detect   # Show detected platform info

# Specific platforms
make linux             # Build for Linux
make macos             # Build for macOS (Universal Binary)
make windows           # Build for Windows

# Cross-compilation (requires toolchains)
make cross-linux       # Cross-compile for Linux
make cross-windows     # Cross-compile for Windows

# Utility targets
make clean             # Clean build files
make rebuild           # Clean and build
make run               # Run with default message
make ping              # Test ping functionality
make platform-info    # Show runtime platform info
make help              # Show available targets
```

### Using CMake

```bash
# Configure and build
mkdir build && cd build
cmake ..
cmake --build .

# Specify build type
cmake -DCMAKE_BUILD_TYPE=Release ..

# macOS Universal Binary
cmake -DCMAKE_OSX_ARCHITECTURES="x86_64;arm64" ..

# Windows with MinGW
cmake -G "MinGW Makefiles" ..
```

### Using Build Scripts

#### Linux/macOS (build.sh)
```bash
# Basic usage
./build.sh                          # Debug build with Make
./build.sh --release                # Release build
./build.sh --cmake                  # Use CMake
./build.sh --clean --test           # Clean build with tests

# Cross-compilation
./build.sh cross-linux              # Cross-compile for Linux
./build.sh cross-windows            # Cross-compile for Windows

# Help
./build.sh --help
```

#### Windows (build.bat)
```cmd
REM Basic usage
build.bat                           REM Debug build with Make
build.bat --release                 REM Release build
build.bat --cmake                   REM Use CMake
build.bat --clean --test            REM Clean build with tests

REM Help
build.bat --help
```

## Prerequisites

### Linux
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install build-essential cmake

# Cross-compilation toolchains (optional)
sudo apt-get install gcc-multilib g++-multilib  # For Linux targets
sudo apt-get install mingw-w64                  # For Windows targets
```

### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install CMake (optional)
brew install cmake
```

### Windows

**Option 1: MSYS2 (Recommended)**
```bash
# Install MSYS2 from https://www.msys2.org/
# In MSYS2 terminal:
pacman -S base-devel mingw-w64-x86_64-toolchain
pacman -S mingw-w64-x86_64-cmake  # For CMake support
```

**Option 2: Visual Studio**
- Install Visual Studio 2019/2022 with C++ tools
- Use CMake with Visual Studio generator

## Output Files

### Make Build System
- **Linux**: `build/cpp-sdk-linux`
- **macOS**: `build/cpp-sdk-macos` (Universal Binary)
- **Windows**: `build/cpp-sdk-windows.exe`

### CMake Build System
- All platforms: `build/bin/cpp-sdk-<platform>[.exe]`

## Cross-Compilation

### Linux → Windows
```bash
# Install MinGW cross-compiler
sudo apt-get install mingw-w64

# Build
make cross-windows
# Or
CXX=x86_64-w64-mingw32-g++ make windows
```

### Linux → Linux (different arch)
```bash
# Install cross-compiler
sudo apt-get install gcc-multilib g++-multilib

# Build
make cross-linux
# Or
CXX=x86_64-linux-gnu-g++ make linux
```

## Testing

### Functional Tests
```bash
# Test platform detection
make platform-info

# Test networking
make ping

# Test custom messages
make run-custom

# Show help
make app-help
```

### Build Tests
```bash
# With build script
./build.sh --test

# Manual CMake tests
cd build && ctest --verbose
```

## Troubleshooting

### Common Issues

**Make not found (Windows)**
- Install MSYS2 or use `mingw32-make` instead of `make`

**CMake not found**
- Install CMake from https://cmake.org/download/

**Cross-compilation toolchain not found**
- Install appropriate toolchains (see Prerequisites)

**Permission denied (Unix)**
- Make build script executable: `chmod +x build.sh`

### Platform-Specific Notes

**Linux**
- Uses `ping -c 1` for network tests
- Static linking available with GCC flags

**macOS**
- Builds Universal Binaries (Intel + Apple Silicon)
- Uses `ping -c 1` for network tests
- Can create `.app` bundles with CMake

**Windows**
- Uses `ping -n 1` for network tests
- Requires MinGW or Visual Studio
- Static linking enabled by default

## Advanced Configuration

### CMake Options
```bash
# Create macOS app bundle
cmake -DCREATE_BUNDLE=ON ..

# Custom install prefix
cmake -DCMAKE_INSTALL_PREFIX=/usr/local ..

# Debug symbols in release
cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo ..
```

### Make Variables
```bash
# Custom compiler
CXX=clang++ make

# Custom flags
CXXFLAGS="-O3 -march=native" make

# Verbose output
VERBOSE=1 make
```

## Integration

### CI/CD Pipeline Example
```yaml
# GitHub Actions example
- name: Build for all platforms
  run: |
    ./build.sh linux
    ./build.sh cross-windows
    # Upload artifacts
```

### IDE Support
- **CLion**: Open CMakeLists.txt
- **Visual Studio**: Use CMake integration
- **VS Code**: Install CMake Tools extension

## Support

For build issues:
1. Check prerequisites are installed
2. Verify toolchain availability
3. Try different build system (Make vs CMake)
4. Check platform-specific notes above