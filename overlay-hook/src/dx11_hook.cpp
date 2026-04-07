#include <d3d11.h>
#include <dxgi.h>
#include "hook.h"
#include "MinHook.h"

// MinHook would normally be included here
// For now, we'll use manual hooking approach or placeholder
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

        // Render overlay on top of game frame (test texture created in InitializeOverlayResources)
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
    WNDCLASSEX wc = { sizeof(WNDCLASSEX), CS_CLASSDC, DefWindowProc, 0L, 0L, GetModuleHandle(nullptr), nullptr, nullptr, nullptr, nullptr, "DX", nullptr };
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

    // Hook using MinHook (placeholder - MinHook integration needed)
    MH_Initialize();
    MH_CreateHook(pPresentAddr, &hkPresent, reinterpret_cast<LPVOID*>(&oPresent));
    MH_CreateHook(pResizeBuffersAddr, &hkResizeBuffers, reinterpret_cast<LPVOID*>(&oResizeBuffers));
    MH_EnableHook(pPresentAddr);
    MH_EnableHook(pResizeBuffersAddr);

    // For now, return false until MinHook is integrated
    return true;
}
