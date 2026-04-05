#!/bin/bash

# Build script for DirectX Hook DLL

set -e

echo "Building DirectX Hook DLL..."

# Check if CMake is installed
if ! command -v cmake &> /dev/null; then
    echo "Error: CMake is not installed"
    echo "Please install CMake from https://cmake.org/download/"
    exit 1
fi

# Check if we're on Windows
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" ]]; then
    echo "Warning: This build script is designed for Windows"
    echo "Current OS: $OSTYPE"
fi

# Create build directory
cd overlay-hook
mkdir -p build
cd build

# Configure with CMake
echo "Configuring with CMake..."
cmake .. -G "Visual Studio 17 2022" -A x64

# Build Release configuration
echo "Building Release configuration..."
cmake --build . --config Release

echo ""
echo "Build complete!"
echo "DLL location: ../target/release/broken_latch_hook.dll"
echo ""
echo "⚠️  IMPORTANT: MinHook Integration Required"
echo "The DLL has placeholder hooks. To enable actual DirectX hooking:"
echo "1. Download MinHook from https://github.com/TsudaKageyu/minhook/releases"
echo "2. Place MinHook.x64.lib in overlay-hook/lib/"
echo "3. Place MinHook.h in overlay-hook/include/"
echo "4. Uncomment MinHook calls in dx11_hook.cpp (lines marked with // MH_)"
echo "5. Rebuild the DLL"
