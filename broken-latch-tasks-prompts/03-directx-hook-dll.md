# Task 03: DirectX Hook DLL

**Platform: broken-latch**  
**Dependencies:** Task 01, 02  
**Estimated Complexity:** Very High  
**Priority:** P0 (Critical Path)

---

## Objective

Build the C++ DLL that injects into LeagueOfLegends.exe and hooks DirectX 11/12's `IDXGISwapChain::Present()` function. This hook enables seamless compositing of the overlay window content onto the game's back buffer, creating a smooth, integrated overlay experience without screen tearing or flicker.

---

## Context

The DirectX hook is what makes the overlay truly "inside" the game rather than just a window on top of it. By hooking at the Present() call (the exact moment before each frame is displayed), we can:

- Render overlay content directly on the game's back buffer
- Avoid Alt+Tab detection by anti-cheat
- Prevent screen capture from seeing the overlay (if desired)
- Achieve perfect frame sync with the game

This is the most technically complex task in the platform. It requires deep knowledge of DirectX, Windows process injection, and careful handling to avoid crashes or anti-cheat flags.

**Important**: This hook ONLY reads/writes the rendering surface. It does NOT touch game memory, logic, or network traffic.

---

## What You Need to Build

### 1. Project Structure

```
overlay-hook/
├── CMakeLists.txt
├── src/
│   ├── dllmain.cpp          # DLL entry point
│   ├── dx_hook.cpp          # DirectX hooking logic
│   ├── dx11_hook.cpp        # DirectX 11 specific
│   ├── dx12_hook.cpp        # DirectX 12 specific
│   ├── render.cpp           # Overlay compositing
│   ├── pipe.cpp             # Named pipe communication
│   └── hook.h               # Shared definitions
├── include/
│   ├── MinHook.h            # MinHook library header
│   └── d3d11.h              # DirectX headers
└── lib/
    └── MinHook.x64.lib      # MinHook static lib
```

### 2. CMakeLists.txt

```cmake
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
```

### 3. DLL Entry Point (`src/dllmain.cpp`)

```cpp
#include <Windows.h>
#include <iostream>
#include <thread>
#include "hook.h"

HMODULE g_hModule = nullptr;

DWORD WINAPI InitializeHook(LPVOID lpParam) {
    // Wait for game to initialize DirectX
    Sleep(2000);

    // Try DirectX 11 first (LoL's current renderer)
    if (InitializeDX11Hook()) {
        SendPipeMessage("DX11_HOOKED");
        return 0;
    }

    // Fallback to DirectX 12
    if (InitializeDX12Hook()) {
        SendPipeMessage("DX12_HOOKED");
        return 0;
    }

    SendPipeMessage("HOOK_FAILED");
    return 1;
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD dwReason, LPVOID lpReserved) {
    g_hModule = hModule;

    switch (dwReason) {
        case DLL_PROCESS_ATTACH:
            DisableThreadLibraryCalls(hModule);
            CreateThread(nullptr, 0, InitializeHook, nullptr, 0, nullptr);
            break;

        case DLL_PROCESS_DETACH:
            CleanupHooks();
            break;
    }

    return TRUE;
}
```

### 4. DirectX 11 Hook (`src/dx11_hook.cpp`)

```cpp
#include <d3d11.h>
#include <dxgi.h>
#include "MinHook.h"
#include "hook.h"

typedef HRESULT(__stdcall* Present_t)(IDXGISwapChain*, UINT, UINT);
typedef HRESULT(__stdcall* ResizeBuffers_t)(IDXGISwapChain*, UINT, UINT, UINT, DXGI_FORMAT, UINT);

Present_t oPresent = nullptr;
ResizeBuffers_t oResizeBuffers = nullptr;

ID3D11Device* g_pd3dDevice = nullptr;
ID3D11DeviceContext* g_pd3dContext = nullptr;
ID3D11RenderTargetView* g_mainRenderTargetView = nullptr;

// Overlay rendering state
ID3D11Texture2D* g_overlayTexture = nullptr;
ID3D11ShaderResourceView* g_overlaySRV = nullptr;
ID3D11VertexShader* g_vertexShader = nullptr;
ID3D11PixelShader* g_pixelShader = nullptr;
ID3D11SamplerState* g_samplerState = nullptr;
ID3D11BlendState* g_blendState = nullptr;

bool InitializeRenderTarget(IDXGISwapChain* pSwapChain) {
    HRESULT hr;

    // Get device
    hr = pSwapChain->GetDevice(__uuidof(ID3D11Device), (void**)&g_pd3dDevice);
    if (FAILED(hr)) return false;

    g_pd3dDevice->GetImmediateContext(&g_pd3dContext);

    // Get back buffer
    ID3D11Texture2D* pBackBuffer = nullptr;
    hr = pSwapChain->GetBuffer(0, __uuidof(ID3D11Texture2D), (void**)&pBackBuffer);
    if (FAILED(hr)) return false;

    // Create render target view
    hr = g_pd3dDevice->CreateRenderTargetView(pBackBuffer, nullptr, &g_mainRenderTargetView);
    pBackBuffer->Release();

    return SUCCEEDED(hr);
}

void CleanupRenderTarget() {
    if (g_mainRenderTargetView) {
        g_mainRenderTargetView->Release();
        g_mainRenderTargetView = nullptr;
    }
}

HRESULT __stdcall hkPresent(IDXGISwapChain* pSwapChain, UINT SyncInterval, UINT Flags) {
    static bool initialized = false;

    if (!initialized) {
        if (InitializeRenderTarget(pSwapChain)) {
            InitializeOverlayResources();
            initialized = true;
        }
    }

    if (initialized && g_mainRenderTargetView) {
        // Save current render target
        ID3D11RenderTargetView* pOldRTV = nullptr;
        ID3D11DepthStencilView* pOldDSV = nullptr;
        g_pd3dContext->OMGetRenderTargets(1, &pOldRTV, &pOldDSV);

        // Set our render target (game's back buffer)
        g_pd3dContext->OMSetRenderTargets(1, &g_mainRenderTargetView, nullptr);

        // Render overlay on top of game frame
        RenderOverlay(g_pd3dContext);

        // Restore original render target
        g_pd3dContext->OMSetRenderTargets(1, &pOldRTV, pOldDSV);
        if (pOldRTV) pOldRTV->Release();
        if (pOldDSV) pOldDSV->Release();
    }

    return oPresent(pSwapChain, SyncInterval, Flags);
}

HRESULT __stdcall hkResizeBuffers(IDXGISwapChain* pSwapChain, UINT BufferCount, UINT Width, UINT Height, DXGI_FORMAT NewFormat, UINT SwapChainFlags) {
    CleanupRenderTarget();

    HRESULT hr = oResizeBuffers(pSwapChain, BufferCount, Width, Height, NewFormat, SwapChainFlags);

    if (SUCCEEDED(hr)) {
        InitializeRenderTarget(pSwapChain);
    }

    return hr;
}

bool InitializeDX11Hook() {
    // Create temporary swap chain to find Present() address
    WNDCLASSEX wc = { sizeof(WNDCLASSEX), CS_CLASSDC, DefWindowProc, 0L, 0L, GetModuleHandle(nullptr), nullptr, nullptr, nullptr, nullptr, L"DX", nullptr };
    RegisterClassEx(&wc);
    HWND hWnd = CreateWindow(wc.lpszClassName, nullptr, WS_OVERLAPPEDWINDOW, 0, 0, 100, 100, nullptr, nullptr, wc.hInstance, nullptr);

    DXGI_SWAP_CHAIN_DESC sd = {};
    sd.BufferCount = 1;
    sd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    sd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    sd.OutputWindow = hWnd;
    sd.SampleDesc.Count = 1;
    sd.Windowed = TRUE;
    sd.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    IDXGISwapChain* pSwapChain = nullptr;
    ID3D11Device* pDevice = nullptr;
    ID3D11DeviceContext* pContext = nullptr;

    HRESULT hr = D3D11CreateDeviceAndSwapChain(
        nullptr,
        D3D_DRIVER_TYPE_HARDWARE,
        nullptr,
        0,
        nullptr,
        0,
        D3D11_SDK_VERSION,
        &sd,
        &pSwapChain,
        &pDevice,
        nullptr,
        &pContext
    );

    if (FAILED(hr)) {
        DestroyWindow(hWnd);
        UnregisterClass(wc.lpszClassName, wc.hInstance);
        return false;
    }

    // Get vtable
    void** pSwapChainVTable = *reinterpret_cast<void***>(pSwapChain);
    void* pPresentAddr = pSwapChainVTable[8];  // Present is at index 8
    void* pResizeBuffersAddr = pSwapChainVTable[13];  // ResizeBuffers at 13

    // Cleanup temp resources
    pSwapChain->Release();
    pDevice->Release();
    pContext->Release();
    DestroyWindow(hWnd);
    UnregisterClass(wc.lpszClassName, wc.hInstance);

    // Hook using MinHook
    MH_Initialize();

    MH_CreateHook(pPresentAddr, &hkPresent, reinterpret_cast<LPVOID*>(&oPresent));
    MH_CreateHook(pResizeBuffersAddr, &hkResizeBuffers, reinterpret_cast<LPVOID*>(&oResizeBuffers));

    MH_EnableHook(pPresentAddr);
    MH_EnableHook(pResizeBuffersAddr);

    return true;
}
```

### 5. Overlay Rendering (`src/render.cpp`)

```cpp
#include <d3d11.h>
#include "hook.h"

// Vertex structure for fullscreen quad
struct Vertex {
    float pos[3];
    float uv[2];
};

// Vertex shader (simple passthrough)
const char* g_vertexShaderSrc = R"(
    struct VS_INPUT {
        float3 pos : POSITION;
        float2 uv : TEXCOORD0;
    };

    struct PS_INPUT {
        float4 pos : SV_POSITION;
        float2 uv : TEXCOORD0;
    };

    PS_INPUT main(VS_INPUT input) {
        PS_INPUT output;
        output.pos = float4(input.pos, 1.0f);
        output.uv = input.uv;
        return output;
    }
)";

// Pixel shader (sample overlay texture with alpha blending)
const char* g_pixelShaderSrc = R"(
    Texture2D overlayTexture : register(t0);
    SamplerState samplerState : register(s0);

    struct PS_INPUT {
        float4 pos : SV_POSITION;
        float2 uv : TEXCOORD0;
    };

    float4 main(PS_INPUT input) : SV_TARGET {
        return overlayTexture.Sample(samplerState, input.uv);
    }
)";

void InitializeOverlayResources() {
    // Create vertex shader
    ID3DBlob* vsBlob = nullptr;
    D3DCompile(g_vertexShaderSrc, strlen(g_vertexShaderSrc), nullptr, nullptr, nullptr, "main", "vs_5_0", 0, 0, &vsBlob, nullptr);
    g_pd3dDevice->CreateVertexShader(vsBlob->GetBufferPointer(), vsBlob->GetBufferSize(), nullptr, &g_vertexShader);
    vsBlob->Release();

    // Create pixel shader
    ID3DBlob* psBlob = nullptr;
    D3DCompile(g_pixelShaderSrc, strlen(g_pixelShaderSrc), nullptr, nullptr, nullptr, "main", "ps_5_0", 0, 0, &psBlob, nullptr);
    g_pd3dDevice->CreatePixelShader(psBlob->GetBufferPointer(), psBlob->GetBufferSize(), nullptr, &g_pixelShader);
    psBlob->Release();

    // Create blend state for alpha blending
    D3D11_BLEND_DESC blendDesc = {};
    blendDesc.RenderTarget[0].BlendEnable = TRUE;
    blendDesc.RenderTarget[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
    blendDesc.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
    blendDesc.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
    blendDesc.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_ONE;
    blendDesc.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_ZERO;
    blendDesc.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
    blendDesc.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL;

    g_pd3dDevice->CreateBlendState(&blendDesc, &g_blendState);

    // Create sampler state
    D3D11_SAMPLER_DESC samplerDesc = {};
    samplerDesc.Filter = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
    samplerDesc.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
    samplerDesc.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
    samplerDesc.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;

    g_pd3dDevice->CreateSamplerState(&samplerDesc, &g_samplerState);
}

void RenderOverlay(ID3D11DeviceContext* context) {
    if (!g_overlaySRV) return;  // No overlay texture yet

    // Save pipeline state
    ID3D11RasterizerState* oldRS = nullptr;
    ID3D11BlendState* oldBS = nullptr;
    float oldBlendFactor[4];
    UINT oldSampleMask;
    context->RSGetState(&oldRS);
    context->OMGetBlendState(&oldBS, oldBlendFactor, &oldSampleMask);

    // Set our blend state
    float blendFactor[4] = { 1.0f, 1.0f, 1.0f, 1.0f };
    context->OMSetBlendState(g_blendState, blendFactor, 0xFFFFFFFF);

    // Set shaders and resources
    context->VSSetShader(g_vertexShader, nullptr, 0);
    context->PSSetShader(g_pixelShader, nullptr, 0);
    context->PSSetShaderResources(0, 1, &g_overlaySRV);
    context->PSSetSamplers(0, 1, &g_samplerState);

    // Draw fullscreen quad (geometry generated in shader)
    context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
    context->Draw(4, 0);

    // Restore pipeline state
    context->OMSetBlendState(oldBS, oldBlendFactor, oldSampleMask);
    if (oldRS) oldRS->Release();
    if (oldBS) oldBS->Release();
}
```

### 6. Named Pipe Communication (`src/pipe.cpp`)

```cpp
#include <Windows.h>
#include <string>

HANDLE g_hPipe = INVALID_HANDLE_VALUE;

bool ConnectToPlatform() {
    g_hPipe = CreateFile(
        L"\\\\.\\pipe\\broken_latch",
        GENERIC_WRITE,
        0,
        nullptr,
        OPEN_EXISTING,
        0,
        nullptr
    );

    return g_hPipe != INVALID_HANDLE_VALUE;
}

void SendPipeMessage(const char* message) {
    if (g_hPipe == INVALID_HANDLE_VALUE) {
        if (!ConnectToPlatform()) return;
    }

    DWORD written;
    WriteFile(g_hPipe, message, strlen(message), &written, nullptr);
}

void CleanupPipe() {
    if (g_hPipe != INVALID_HANDLE_VALUE) {
        CloseHandle(g_hPipe);
        g_hPipe = INVALID_HANDLE_VALUE;
    }
}
```

### 7. Hook Header (`src/hook.h`)

```cpp
#pragma once
#include <Windows.h>
#include <d3d11.h>

// DirectX 11 hook
extern ID3D11Device* g_pd3dDevice;
extern ID3D11DeviceContext* g_pd3dContext;

bool InitializeDX11Hook();
bool InitializeDX12Hook();
void CleanupHooks();

// Rendering
void InitializeOverlayResources();
void RenderOverlay(ID3D11DeviceContext* context);

// Communication
bool ConnectToPlatform();
void SendPipeMessage(const char* message);
void CleanupPipe();
```

---

## Integration with Platform (Task 01, 02)

### From Rust Side (`src-tauri/src/hook/injector.rs`):

```rust
use std::process::Command;
use sysinfo::{ProcessExt, System, SystemExt};
use windows::Win32::System::Threading::*;
use windows::Win32::System::LibraryLoader::*;

pub fn inject_dll_into_game() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::new_all();
    system.refresh_all();

    // Find LeagueOfLegends.exe
    let league_process = system.processes().values()
        .find(|p| p.name() == "LeagueOfLegends.exe")
        .ok_or("League of Legends process not found")?;

    let pid = league_process.pid() as u32;

    // Open process
    let h_process = unsafe {
        OpenProcess(PROCESS_ALL_ACCESS, false, pid)?
    };

    // Get DLL path
    let dll_path = std::env::current_dir()?
        .join("broken_latch_hook.dll");

    // Allocate memory in target process
    let dll_path_str = dll_path.to_string_lossy();
    let dll_path_bytes = dll_path_str.as_bytes();

    let remote_mem = unsafe {
        VirtualAllocEx(
            h_process,
            None,
            dll_path_bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE
        )
    };

    // Write DLL path to remote process
    unsafe {
        WriteProcessMemory(
            h_process,
            remote_mem,
            dll_path_bytes.as_ptr() as *const _,
            dll_path_bytes.len(),
            None
        )?;
    }

    // Get LoadLibraryA address
    let kernel32 = unsafe { GetModuleHandleA(s!("kernel32.dll"))? };
    let load_library = unsafe { GetProcAddress(kernel32, s!("LoadLibraryA")) };

    // Create remote thread
    unsafe {
        CreateRemoteThread(
            h_process,
            None,
            0,
            Some(std::mem::transmute(load_library)),
            Some(remote_mem),
            0,
            None
        )?;
    }

    Ok(())
}
```

---

## Testing Requirements

### Manual Testing (No automated tests for DLL injection):

1. **Build DLL**:

```bash
cd overlay-hook
mkdir build && cd build
cmake ..
cmake --build . --config Release
```

2. **Test Injection**:

- Launch League of Legends
- Run platform
- Verify DLL is injected (check with Process Explorer)
- Verify named pipe message received: "DX11_HOOKED"

3. **Visual Test**:

- Platform should render overlay seamlessly on top of game
- No screen tearing or flicker
- Overlay should move with game window
- No FPS drop (measure with in-game FPS counter)

4. **Stability Test**:

- Play 5+ full games
- Verify no crashes
- Check for memory leaks (DLL memory usage should be stable)

---

## Acceptance Criteria

✅ **Complete when:**

1. DLL compiles without errors
2. DLL successfully injects into LeagueOfLegends.exe
3. DirectX 11 Present() hook is active
4. Overlay renders on game back buffer
5. No visible tearing or performance issues
6. Named pipe communication works (platform receives "DX11_HOOKED")
7. DLL cleans up properly on game exit
8. No anti-cheat flags triggered (test in multiple games)
9. Frame time impact <0.3ms (measured)
10. Fallback to transparent window works if hook fails

---

## Performance Requirements

- Hook initialization: <500ms
- Frame time added: <0.3ms per frame
- DLL RAM usage: <3MB
- No CPU spikes on injection

---

## Security & Anti-Cheat Considerations

**Safe:**

- ✅ Only hooks rendering functions
- ✅ Only reads/writes rendering surface
- ✅ Uses documented DirectX API
- ✅ No game memory access
- ✅ No network interception

**Risky (AVOID):**

- ❌ Reading game memory beyond rendering surface
- ❌ Hooking game logic functions
- ❌ Modifying game data structures

**Anti-Cheat Notes:**

- Riot Vanguard may flag kernel-level hooks
- This DLL is user-mode only
- Test thoroughly in custom games before ranked

---

## Dependencies for Next Tasks

- **Task 04** (Game Detection) will trigger DLL injection
- **Task 06** (Widget System) will render content that this DLL composites
- **Task 07** (App Loader) will have apps whose content gets rendered here

---

## Files to Create

### New Files:

- `overlay-hook/CMakeLists.txt`
- `overlay-hook/src/dllmain.cpp`
- `overlay-hook/src/dx11_hook.cpp`
- `overlay-hook/src/dx12_hook.cpp`
- `overlay-hook/src/render.cpp`
- `overlay-hook/src/pipe.cpp`
- `overlay-hook/src/hook.h`
- `src-tauri/src/hook/injector.rs` (platform side)
- `src-tauri/src/hook/pipe.rs` (platform side - named pipe server)

---

## Expected Time: 10-12 hours

## Difficulty: Very High (Requires DirectX + Windows API expertise)
