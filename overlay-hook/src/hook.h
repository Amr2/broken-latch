#pragma once
#include <Windows.h>
#include <d3d11.h>

// DirectX 11 hook
extern ID3D11Device* g_pd3dDevice;
extern ID3D11DeviceContext* g_pd3dContext;
extern ID3D11RenderTargetView* g_mainRenderTargetView;

// DirectX 11 resources
extern ID3D11Texture2D* g_overlayTexture;
extern ID3D11ShaderResourceView* g_overlaySRV;
extern ID3D11VertexShader* g_vertexShader;
extern ID3D11PixelShader* g_pixelShader;
extern ID3D11SamplerState* g_samplerState;
extern ID3D11BlendState* g_blendState;
extern ID3D11Buffer* g_vertexBuffer;
extern ID3D11InputLayout* g_inputLayout;

// Hook initialization
bool InitializeDX11Hook();
bool InitializeDX12Hook();
void CleanupHooks();

// Rendering
void InitializeOverlayResources();
void RenderOverlay(ID3D11DeviceContext* context);
void CleanupRenderResources();

// Communication
bool ConnectToPlatform();
void SendPipeMessage(const char* message);
void CleanupPipe();

// Global module handle
extern HMODULE g_hModule;
