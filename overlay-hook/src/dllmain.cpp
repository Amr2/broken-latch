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
            CleanupPipe();
            break;
    }

    return TRUE;
}
