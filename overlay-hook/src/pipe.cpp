#include <Windows.h>
#include <string.h>

HANDLE g_hPipe = INVALID_HANDLE_VALUE;

bool ConnectToPlatform() {
    g_hPipe = CreateFileW(
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
    WriteFile(g_hPipe, message, static_cast<DWORD>(strlen(message)), &written, nullptr);
}

void CleanupPipe() {
    if (g_hPipe != INVALID_HANDLE_VALUE) {
        CloseHandle(g_hPipe);
        g_hPipe = INVALID_HANDLE_VALUE;
    }
}
