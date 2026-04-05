#include <Windows.h>
#include "hook.h"

void CleanupHooks() {
    // Cleanup DirectX 11 resources
    CleanupRenderResources();
    
    // Release DirectX 11 context and device
    if (g_pd3dContext) {
        g_pd3dContext->Release();
        g_pd3dContext = nullptr;
    }
    
    if (g_pd3dDevice) {
        g_pd3dDevice->Release();
        g_pd3dDevice = nullptr;
    }

    // Disable hooks (MinHook integration needed)
    // MH_DisableHook(MH_ALL_HOOKS);
    // MH_Uninitialize();
}
