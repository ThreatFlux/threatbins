# C++ SDK Universal Makefile - Cross-Platform Support
# Automatically detects platform and uses appropriate makefile

# Detect operating system
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Set platform-specific variables
ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
    PLATFORM_MAKEFILE := Makefile.windows
    EXECUTABLE_SUFFIX := .exe
else
    ifeq ($(UNAME_S),Linux)
        DETECTED_OS := Linux
        PLATFORM_MAKEFILE := Makefile.linux
        EXECUTABLE_SUFFIX :=
    endif
    ifeq ($(UNAME_S),Darwin)
        DETECTED_OS := macOS
        PLATFORM_MAKEFILE := Makefile.macos
        EXECUTABLE_SUFFIX :=
    endif
endif

# Default behavior - delegate to platform-specific makefile
all:
	@echo "Building C++ SDK for $(DETECTED_OS)..."
	@$(MAKE) -f $(PLATFORM_MAKEFILE) all

# Platform-specific build targets
linux:
	@echo "Building for Linux..."
	@$(MAKE) -f Makefile.linux all

macos:
	@echo "Building for macOS (Universal Binary)..."
	@$(MAKE) -f Makefile.macos all

windows:
	@echo "Building for Windows..."
	@$(MAKE) -f Makefile.windows all

# Cross-compilation targets (requires appropriate toolchains)
cross-linux:
	@echo "Cross-compiling for Linux..."
	@CXX=x86_64-linux-gnu-g++ $(MAKE) -f Makefile.linux all

cross-windows:
	@echo "Cross-compiling for Windows..."
	@CXX=x86_64-w64-mingw32-g++ $(MAKE) -f Makefile.windows all

# Delegate common targets to platform-specific makefile
clean:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) clean

rebuild:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) rebuild

install:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) install

uninstall:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) uninstall

run:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) run

run-custom:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) run-custom

ping:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) ping

ping-google:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) ping-google

platform-info:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) platform-info

app-help:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) app-help

test-encode:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) test-encode

test-decode:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) test-decode

test-auto-decode:
	@$(MAKE) -f $(PLATFORM_MAKEFILE) test-auto-decode

# Show all available build targets
help:
	@echo "C++ SDK Cross-Platform Build System"
	@echo "===================================="
	@echo "Detected OS: $(DETECTED_OS)"
	@echo "Using makefile: $(PLATFORM_MAKEFILE)"
	@echo ""
	@echo "Platform Detection Targets:"
	@echo "  all           - Build for current platform ($(DETECTED_OS))"
	@echo "  linux         - Build specifically for Linux"
	@echo "  macos         - Build specifically for macOS"
	@echo "  windows       - Build specifically for Windows"
	@echo ""
	@echo "Cross-Compilation Targets (requires toolchains):"
	@echo "  cross-linux   - Cross-compile for Linux"
	@echo "  cross-windows - Cross-compile for Windows"
	@echo ""
	@echo "Common Targets (delegated to platform makefile):"
	@echo "  clean         - Remove build files"
	@echo "  rebuild       - Clean and build"
	@echo "  install       - Install to system"
	@echo "  uninstall     - Remove from system"
	@echo "  run           - Run with default message"
	@echo "  run-custom    - Run with custom message"
	@echo "  ping          - Ping 1.1.1.1"
	@echo "  ping-google   - Ping 8.8.8.8"
	@echo "  platform-info - Show runtime platform info"
	@echo "  app-help      - Show application help"
	@echo "  test-encode   - Test base64 encoding"
	@echo "  test-decode   - Test base64 decoding"
	@echo "  test-auto-decode - Test auto-decode feature"
	@echo "  help          - Show this help"
	@echo ""
	@echo "Platform-specific help:"
	@$(MAKE) -f $(PLATFORM_MAKEFILE) help

# Show platform detection info
platform-detect:
	@echo "Platform Detection Results:"
	@echo "  Operating System: $(DETECTED_OS)"
	@echo "  Architecture: $(UNAME_M)"
	@echo "  Makefile: $(PLATFORM_MAKEFILE)"
	@echo "  Executable suffix: $(EXECUTABLE_SUFFIX)"

.PHONY: all linux macos windows cross-linux cross-windows clean rebuild install uninstall run run-custom ping ping-google platform-info app-help test-encode test-decode test-auto-decode help platform-detect