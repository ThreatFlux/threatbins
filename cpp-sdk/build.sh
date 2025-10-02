#!/bin/bash
# Cross-Platform Build Script for C++ SDK

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Detect platform
detect_platform() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        PLATFORM="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        PLATFORM="macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
        PLATFORM="windows"
    else
        PLATFORM="unknown"
    fi
}

# Show usage
show_usage() {
    echo "Usage: $0 [OPTIONS] [TARGET]"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help          Show this help"
    echo "  -c, --clean         Clean build directory before building"
    echo "  -r, --release       Build in release mode (default: debug)"
    echo "  -t, --test          Run tests after building"
    echo "  --cmake             Use CMake build system"
    echo "  --make              Use Make build system (default)"
    echo ""
    echo "TARGETS:"
    echo "  all                 Build for current platform (default)"
    echo "  linux               Build for Linux"
    echo "  macos               Build for macOS"
    echo "  windows             Build for Windows"
    echo "  cross-linux         Cross-compile for Linux"
    echo "  cross-windows       Cross-compile for Windows"
    echo ""
    echo "Examples:"
    echo "  $0                  # Build for current platform using Make"
    echo "  $0 --cmake          # Build using CMake"
    echo "  $0 -r --test        # Release build with tests"
    echo "  $0 linux            # Build specifically for Linux"
}

# Build using Make
build_with_make() {
    local target=${1:-"all"}

    print_header "Building with Make for $target"

    if [[ "$CLEAN" == "true" ]]; then
        print_status "Cleaning build directory..."
        make clean
    fi

    case $target in
        "all"|"linux"|"macos"|"windows")
            make $target
            ;;
        "cross-linux")
            if command -v x86_64-linux-gnu-g++ &> /dev/null; then
                make cross-linux
            else
                print_error "Linux cross-compilation toolchain not found"
                print_warning "Install: sudo apt-get install gcc-multilib g++-multilib"
                exit 1
            fi
            ;;
        "cross-windows")
            if command -v x86_64-w64-mingw32-g++ &> /dev/null; then
                make cross-windows
            else
                print_error "Windows cross-compilation toolchain not found"
                print_warning "Install: sudo apt-get install mingw-w64"
                exit 1
            fi
            ;;
        *)
            print_error "Unknown target: $target"
            show_usage
            exit 1
            ;;
    esac
}

# Build using CMake
build_with_cmake() {
    local build_type=${1:-"Debug"}

    print_header "Building with CMake ($build_type)"

    # Create build directory
    mkdir -p build
    cd build

    if [[ "$CLEAN" == "true" ]]; then
        print_status "Cleaning CMake build directory..."
        rm -rf *
    fi

    # Configure
    print_status "Configuring CMake..."
    cmake -DCMAKE_BUILD_TYPE=$build_type ..

    # Build
    print_status "Building..."
    cmake --build . --config $build_type

    cd ..
}

# Run tests
run_tests() {
    print_header "Running Tests"

    if [[ "$BUILD_SYSTEM" == "cmake" ]]; then
        cd build
        ctest --config $CMAKE_BUILD_TYPE --verbose
        cd ..
    else
        print_status "Testing basic functionality..."
        make platform-info
        make ping
    fi
}

# Parse command line arguments
CLEAN="false"
BUILD_TYPE="Debug"
BUILD_SYSTEM="make"
RUN_TESTS="false"
TARGET="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -c|--clean)
            CLEAN="true"
            shift
            ;;
        -r|--release)
            BUILD_TYPE="Release"
            shift
            ;;
        -t|--test)
            RUN_TESTS="true"
            shift
            ;;
        --cmake)
            BUILD_SYSTEM="cmake"
            shift
            ;;
        --make)
            BUILD_SYSTEM="make"
            shift
            ;;
        all|linux|macos|windows|cross-linux|cross-windows)
            TARGET="$1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
print_header "C++ SDK Cross-Platform Builder"

# Detect platform
detect_platform
print_status "Detected platform: $PLATFORM"

# Check dependencies
if [[ "$BUILD_SYSTEM" == "cmake" ]]; then
    if ! command -v cmake &> /dev/null; then
        print_error "CMake not found. Please install CMake."
        exit 1
    fi
else
    if ! command -v make &> /dev/null; then
        print_error "Make not found. Please install Make."
        exit 1
    fi
fi

# Build
if [[ "$BUILD_SYSTEM" == "cmake" ]]; then
    build_with_cmake "$BUILD_TYPE"
else
    build_with_make "$TARGET"
fi

# Run tests if requested
if [[ "$RUN_TESTS" == "true" ]]; then
    run_tests
fi

print_status "Build completed successfully!"
print_status "Check the build/ directory for executables."