@echo off
REM Cross-Platform Build Script for C++ SDK (Windows)

setlocal enabledelayedexpansion

REM Default values
set CLEAN=false
set BUILD_TYPE=Debug
set BUILD_SYSTEM=make
set RUN_TESTS=false
set TARGET=all

REM Parse command line arguments
:parse_args
if "%~1"=="" goto :end_parse
if "%~1"=="-h" goto :show_usage
if "%~1"=="--help" goto :show_usage
if "%~1"=="-c" set CLEAN=true
if "%~1"=="--clean" set CLEAN=true
if "%~1"=="-r" set BUILD_TYPE=Release
if "%~1"=="--release" set BUILD_TYPE=Release
if "%~1"=="-t" set RUN_TESTS=true
if "%~1"=="--test" set RUN_TESTS=true
if "%~1"=="--cmake" set BUILD_SYSTEM=cmake
if "%~1"=="--make" set BUILD_SYSTEM=make
if "%~1"=="all" set TARGET=all
if "%~1"=="windows" set TARGET=windows
shift
goto :parse_args

:end_parse

echo ================================================
echo C++ SDK Cross-Platform Builder (Windows)
echo ================================================

REM Check dependencies
if "%BUILD_SYSTEM%"=="cmake" (
    cmake --version >nul 2>&1
    if errorlevel 1 (
        echo ERROR: CMake not found. Please install CMake.
        exit /b 1
    )
) else (
    mingw32-make --version >nul 2>&1
    if errorlevel 1 (
        echo ERROR: Make not found. Please install MSYS2/MinGW.
        exit /b 1
    )
)

REM Build
echo Building with %BUILD_SYSTEM% for %TARGET%...

if "%BUILD_SYSTEM%"=="cmake" (
    goto :build_cmake
) else (
    goto :build_make
)

:build_make
if "%CLEAN%"=="true" (
    echo Cleaning build directory...
    mingw32-make clean
)

mingw32-make %TARGET%
if errorlevel 1 (
    echo ERROR: Build failed
    exit /b 1
)
goto :test_section

:build_cmake
if not exist "build" mkdir build
cd build

if "%CLEAN%"=="true" (
    echo Cleaning CMake build directory...
    del /q *.*
)

echo Configuring CMake...
cmake -DCMAKE_BUILD_TYPE=%BUILD_TYPE% -G "MinGW Makefiles" ..
if errorlevel 1 (
    echo ERROR: CMake configuration failed
    exit /b 1
)

echo Building...
cmake --build . --config %BUILD_TYPE%
if errorlevel 1 (
    echo ERROR: Build failed
    exit /b 1
)

cd ..
goto :test_section

:test_section
if "%RUN_TESTS%"=="true" (
    echo Running tests...
    if "%BUILD_SYSTEM%"=="cmake" (
        cd build
        ctest --config %BUILD_TYPE% --verbose
        cd ..
    ) else (
        mingw32-make platform-info
        mingw32-make ping
    )
)

echo Build completed successfully!
echo Check the build/ directory for executables.
goto :eof

:show_usage
echo Usage: %0 [OPTIONS] [TARGET]
echo.
echo OPTIONS:
echo   -h, --help          Show this help
echo   -c, --clean         Clean build directory before building
echo   -r, --release       Build in release mode (default: debug)
echo   -t, --test          Run tests after building
echo   --cmake             Use CMake build system
echo   --make              Use Make build system (default)
echo.
echo TARGETS:
echo   all                 Build for current platform (default)
echo   windows             Build for Windows
echo.
echo Examples:
echo   %0                  # Build for Windows using Make
echo   %0 --cmake          # Build using CMake
echo   %0 -r --test        # Release build with tests
goto :eof