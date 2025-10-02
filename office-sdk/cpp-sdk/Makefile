# C++ SDK Makefile
CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -O2
SRCDIR = src
BUILDDIR = build
INCLUDEDIR = include

# Source files
SOURCES = $(wildcard $(SRCDIR)/*.cpp)
OBJECTS = $(SOURCES:$(SRCDIR)/%.cpp=$(BUILDDIR)/%.o)

# Target executable
TARGET = $(BUILDDIR)/cpp-sdk

# Default target
all: $(TARGET)

# Create build directory if it doesn't exist
$(BUILDDIR):
	mkdir -p $(BUILDDIR)

# Link the executable
$(TARGET): $(OBJECTS) | $(BUILDDIR)
	$(CXX) $(OBJECTS) -o $@

# Compile source files to object files
$(BUILDDIR)/%.o: $(SRCDIR)/%.cpp | $(BUILDDIR)
	$(CXX) $(CXXFLAGS) -I$(INCLUDEDIR) -c $< -o $@

# Clean build files
clean:
	rm -rf $(BUILDDIR)/*

# Rebuild everything
rebuild: clean all

# Install (copy to /usr/local/bin)
install: $(TARGET)
	sudo cp $(TARGET) /usr/local/bin/cpp-sdk

# Uninstall
uninstall:
	sudo rm -f /usr/local/bin/cpp-sdk

# Run the program
run: $(TARGET)
	./$(TARGET)

# Run with custom input
run-custom: $(TARGET)
	./$(TARGET) "Custom hello world message"

# Help
help:
	@echo "Available targets:"
	@echo "  all       - Build the executable"
	@echo "  clean     - Remove build files"
	@echo "  rebuild   - Clean and build"
	@echo "  install   - Install to system"
	@echo "  uninstall - Remove from system"
	@echo "  run       - Run with default message"
	@echo "  run-custom- Run with custom message"
	@echo "  help      - Show this help"

.PHONY: all clean rebuild install uninstall run run-custom help