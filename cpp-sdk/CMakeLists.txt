cmake_minimum_required(VERSION 3.15)

# Project information
project(cpp-sdk
    VERSION 1.0.0
    DESCRIPTION "Cross-Platform C++ SDK with Network Utilities"
    LANGUAGES CXX
)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Platform detection and configuration
if(WIN32)
    set(PLATFORM_NAME "Windows")
    set(EXECUTABLE_SUFFIX ".exe")
elseif(APPLE)
    set(PLATFORM_NAME "macOS")
    set(EXECUTABLE_SUFFIX "")
    # Enable universal binary for Apple Silicon and Intel
    set(CMAKE_OSX_ARCHITECTURES "x86_64;arm64")
elseif(UNIX AND NOT APPLE)
    set(PLATFORM_NAME "Linux")
    set(EXECUTABLE_SUFFIX "")
else()
    set(PLATFORM_NAME "Unknown")
    set(EXECUTABLE_SUFFIX "")
endif()

# Compiler-specific options
if(MSVC)
    # Visual Studio specific options
    add_compile_options(/W4)
    add_compile_definitions(_CRT_SECURE_NO_WARNINGS)
else()
    # GCC/Clang options
    add_compile_options(-Wall -Wextra -Wpedantic)

    # Platform-specific optimizations
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
        add_compile_options(-O3)
    endif()

    # Static linking for Windows (MinGW)
    if(WIN32)
        set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static-libgcc -static-libstdc++")
    endif()
endif()

# Include directories
include_directories(include)

# Source files
file(GLOB_RECURSE SOURCES "src/*.cpp")

# Create executable
add_executable(${PROJECT_NAME} ${SOURCES})

# Set output name with platform suffix
set_target_properties(${PROJECT_NAME} PROPERTIES
    OUTPUT_NAME "cpp-sdk-${PLATFORM_NAME}"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

# Platform-specific configurations
if(WIN32)
    # Windows-specific settings
    target_compile_definitions(${PROJECT_NAME} PRIVATE WIN32_LEAN_AND_MEAN)
elseif(APPLE)
    # macOS-specific settings
    target_compile_definitions(${PROJECT_NAME} PRIVATE MACOS_BUILD)

    # Create app bundle (optional)
    option(CREATE_BUNDLE "Create macOS app bundle" OFF)
    if(CREATE_BUNDLE)
        set_target_properties(${PROJECT_NAME} PROPERTIES
            MACOSX_BUNDLE TRUE
            MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/Info.plist.in"
        )
    endif()
elseif(UNIX)
    # Linux-specific settings
    target_compile_definitions(${PROJECT_NAME} PRIVATE LINUX_BUILD)
endif()

# Installation rules
install(TARGETS ${PROJECT_NAME}
    RUNTIME DESTINATION bin
    BUNDLE DESTINATION /Applications
)

# Create custom build configurations
set(CMAKE_CONFIGURATION_TYPES "Debug;Release;MinSizeRel;RelWithDebInfo" CACHE STRING "" FORCE)

# Custom targets for different build types
add_custom_target(debug
    COMMAND ${CMAKE_COMMAND} --build ${CMAKE_BINARY_DIR} --config Debug
    COMMENT "Building debug version"
)

add_custom_target(release
    COMMAND ${CMAKE_COMMAND} --build ${CMAKE_BINARY_DIR} --config Release
    COMMENT "Building release version"
)

# Test target
enable_testing()
add_custom_target(test-run
    COMMAND ${CMAKE_BINARY_DIR}/bin/cpp-sdk-${PLATFORM_NAME}${EXECUTABLE_SUFFIX} --help
    DEPENDS ${PROJECT_NAME}
    COMMENT "Running basic functionality test"
)

# Platform info target
add_custom_target(platform-info
    COMMAND ${CMAKE_BINARY_DIR}/bin/cpp-sdk-${PLATFORM_NAME}${EXECUTABLE_SUFFIX} --platform
    DEPENDS ${PROJECT_NAME}
    COMMENT "Showing platform information"
)

# Print build information
message(STATUS "C++ SDK Build Configuration")
message(STATUS "===========================")
message(STATUS "Platform: ${PLATFORM_NAME}")
message(STATUS "Architecture: ${CMAKE_SYSTEM_PROCESSOR}")
message(STATUS "Compiler: ${CMAKE_CXX_COMPILER_ID}")
message(STATUS "Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "C++ Standard: ${CMAKE_CXX_STANDARD}")
if(APPLE)
    message(STATUS "macOS Architectures: ${CMAKE_OSX_ARCHITECTURES}")
endif()
message(STATUS "Install prefix: ${CMAKE_INSTALL_PREFIX}")

# Generate build info header
configure_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/include/build_info.h.in"
    "${CMAKE_CURRENT_BINARY_DIR}/include/build_info.h"
)
target_include_directories(${PROJECT_NAME} PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/include")