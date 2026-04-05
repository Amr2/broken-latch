# Overlay Hook DLL

DirectX 11/12 hook DLL for broken-latch overlay platform.

## Building

### Prerequisites

- CMake 3.20+
- Visual Studio 2019+ (MSVC)
- Windows SDK 10.0+
- MinHook library (place in `lib/` directory)

### Build Steps

```bash
cd overlay-hook
mkdir build
cd build
cmake ..
cmake --build . --config Release
```

The DLL will be output to `../target/release/broken_latch_hook.dll`

## Dependencies

- **MinHook**: Function hooking library (https://github.com/TsudaKageyu/minhook)
  - Download MinHook.x64.lib and place in `lib/` directory
  - Download MinHook.h and place in `include/` directory

- **DirectX SDK**: Included with Windows SDK

## Architecture

- `dllmain.cpp` - DLL entry point and initialization
- `dx11_hook.cpp` - DirectX 11 Present() hooking
- `dx12_hook.cpp` - DirectX 12 hooking (stub)
- `render.cpp` - Overlay rendering with shaders
- `pipe.cpp` - Named pipe communication with platform
- `dx_hook.cpp` - Hook cleanup

## Security Notes

This DLL:

- ✅ Only hooks rendering functions (Present, ResizeBuffers)
- ✅ Only accesses rendering surfaces
- ✅ Uses documented DirectX API
- ❌ Does NOT read game memory
- ❌ Does NOT hook game logic
- ❌ Does NOT intercept network traffic

## Testing

Manual testing only (no automated tests for DLL injection):

1. Build DLL in Release mode
2. Launch League of Legends
3. Inject DLL using platform
4. Verify named pipe message "DX11_HOOKED" received
5. Check overlay renders without tearing
6. Monitor FPS (should have <0.3ms impact)

## MinHook Integration TODO

The current implementation has MinHook calls commented out. To complete:

1. Download MinHook from https://github.com/TsudaKageyu/minhook/releases
2. Place MinHook.x64.lib in lib/
3. Place MinHook.h in include/
4. Uncomment MinHook calls in dx11_hook.cpp
5. Rebuild

## Anti-Cheat Considerations

- User-mode only (no kernel hooks)
- No game memory access beyond rendering surface
- Test in custom games before ranked play
- Riot Vanguard may still flag - use at own risk
