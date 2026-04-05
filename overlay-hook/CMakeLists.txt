cmake_minimum_required(VERSION 3.20)
project(overlay_hook)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Sources
set(SOURCES
    src/dllmain.cpp
    src/dx_hook.cpp
    src/dx11_hook.cpp
    src/dx12_hook.cpp
    src/render.cpp
    src/pipe.cpp
)

# Create DLL
add_library(overlay_hook SHARED ${SOURCES})

# Include directories
target_include_directories(overlay_hook PRIVATE
    ${CMAKE_SOURCE_DIR}/include
)

# Link libraries
target_link_libraries(overlay_hook PRIVATE
    d3d11.lib
    dxgi.lib
    d3dcompiler.lib
    ${CMAKE_SOURCE_DIR}/lib/MinHook.x64.lib
)

# Output name
set_target_properties(overlay_hook PROPERTIES
    OUTPUT_NAME "broken_latch_hook"
    SUFFIX ".dll"
)

# Set output directory
set_target_properties(overlay_hook PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_SOURCE_DIR}/../target/debug"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_SOURCE_DIR}/../target/release"
)
