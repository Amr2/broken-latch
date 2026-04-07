#include <d3d11.h>
#include <d3dcompiler.h>
#include <string.h>
#include "hook.h"

// Vertex structure for screen-space quad
struct Vertex {
    float x, y;   // NDC position
    float u, v;   // UV
};

// Vertex shader: positions a quad in NDC from per-vertex data
const char* g_vertexShaderSrc = R"(
    struct VS_INPUT {
        float2 pos : POSITION;
        float2 uv  : TEXCOORD0;
    };
    struct PS_INPUT {
        float4 pos : SV_POSITION;
        float2 uv  : TEXCOORD0;
    };
    PS_INPUT main(VS_INPUT input) {
        PS_INPUT o;
        o.pos = float4(input.pos, 0.0f, 1.0f);
        o.uv  = input.uv;
        return o;
    }
)";

// Pixel shader: sample overlay texture
const char* g_pixelShaderSrc = R"(
    Texture2D overlayTexture : register(t0);
    SamplerState samplerState : register(s0);
    struct PS_INPUT {
        float4 pos : SV_POSITION;
        float2 uv  : TEXCOORD0;
    };
    float4 main(PS_INPUT input) : SV_TARGET {
        return overlayTexture.Sample(samplerState, input.uv);
    }
)";

static ID3D11Buffer*      g_vertexBuffer  = nullptr;
static ID3D11InputLayout* g_inputLayout   = nullptr;

void InitializeOverlayResources() {
    if (!g_pd3dDevice) return;

    // ---- Vertex shader ----
    ID3DBlob* vsBlob    = nullptr;
    ID3DBlob* errorBlob = nullptr;
    HRESULT hr = D3DCompile(g_vertexShaderSrc, strlen(g_vertexShaderSrc),
                            nullptr, nullptr, nullptr, "main", "vs_5_0", 0, 0,
                            &vsBlob, &errorBlob);
    if (errorBlob) errorBlob->Release();
    if (FAILED(hr) || !vsBlob) return;

    g_pd3dDevice->CreateVertexShader(vsBlob->GetBufferPointer(),
                                     vsBlob->GetBufferSize(), nullptr, &g_vertexShader);

    // ---- Input layout ----
    D3D11_INPUT_ELEMENT_DESC layout[] = {
        { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT,   0, 0,  D3D11_INPUT_PER_VERTEX_DATA, 0 },
        { "TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT,   0, 8,  D3D11_INPUT_PER_VERTEX_DATA, 0 },
    };
    g_pd3dDevice->CreateInputLayout(layout, 2,
                                    vsBlob->GetBufferPointer(), vsBlob->GetBufferSize(),
                                    &g_inputLayout);
    vsBlob->Release();

    // ---- Pixel shader ----
    ID3DBlob* psBlob = nullptr;
    hr = D3DCompile(g_pixelShaderSrc, strlen(g_pixelShaderSrc),
                    nullptr, nullptr, nullptr, "main", "ps_5_0", 0, 0,
                    &psBlob, &errorBlob);
    if (errorBlob) errorBlob->Release();
    if (SUCCEEDED(hr) && psBlob) {
        g_pd3dDevice->CreatePixelShader(psBlob->GetBufferPointer(),
                                        psBlob->GetBufferSize(), nullptr, &g_pixelShader);
        psBlob->Release();
    }

    // ---- Blend state (alpha blending) ----
    D3D11_BLEND_DESC bd = {};
    bd.RenderTarget[0].BlendEnable           = TRUE;
    bd.RenderTarget[0].SrcBlend              = D3D11_BLEND_SRC_ALPHA;
    bd.RenderTarget[0].DestBlend             = D3D11_BLEND_INV_SRC_ALPHA;
    bd.RenderTarget[0].BlendOp               = D3D11_BLEND_OP_ADD;
    bd.RenderTarget[0].SrcBlendAlpha         = D3D11_BLEND_ONE;
    bd.RenderTarget[0].DestBlendAlpha        = D3D11_BLEND_ZERO;
    bd.RenderTarget[0].BlendOpAlpha          = D3D11_BLEND_OP_ADD;
    bd.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL;
    g_pd3dDevice->CreateBlendState(&bd, &g_blendState);

    // ---- Sampler ----
    D3D11_SAMPLER_DESC sd = {};
    sd.Filter   = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
    sd.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
    sd.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
    sd.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
    g_pd3dDevice->CreateSamplerState(&sd, &g_samplerState);

    // ---- TEST: create a 200x100 solid red-with-alpha texture ----
    // This proves the hook + render pipeline is working before real content is wired up
    const int W = 200, H = 100;
    UINT pixels[W * H];
    for (int i = 0; i < W * H; i++)
        pixels[i] = 0xCC0000FF; // RGBA: red=FF, green=00, blue=00, alpha=CC (~80%)

    D3D11_TEXTURE2D_DESC td = {};
    td.Width            = W;
    td.Height           = H;
    td.MipLevels        = 1;
    td.ArraySize        = 1;
    td.Format           = DXGI_FORMAT_R8G8B8A8_UNORM;
    td.SampleDesc.Count = 1;
    td.Usage            = D3D11_USAGE_DEFAULT;
    td.BindFlags        = D3D11_BIND_SHADER_RESOURCE;

    D3D11_SUBRESOURCE_DATA initData = {};
    initData.pSysMem     = pixels;
    initData.SysMemPitch = W * 4;

    if (SUCCEEDED(g_pd3dDevice->CreateTexture2D(&td, &initData, &g_overlayTexture))) {
        D3D11_SHADER_RESOURCE_VIEW_DESC srvd = {};
        srvd.Format              = DXGI_FORMAT_R8G8B8A8_UNORM;
        srvd.ViewDimension       = D3D11_SRV_DIMENSION_TEXTURE2D;
        srvd.Texture2D.MipLevels = 1;
        g_pd3dDevice->CreateShaderResourceView(g_overlayTexture, &srvd, &g_overlaySRV);
    }

    // ---- Vertex buffer: quad in top-left corner (NDC: x=-1..+something, y=1..down) ----
    // Screen-space quad at top-left, 200x100 pixels.
    // We'll use normalized coordinates: (-1, 1) = top-left of screen.
    // At 1920x1080: 200px wide = 200/960=0.208 NDC; 100px tall = 100/540=0.185 NDC
    // Use approximate fixed values — close enough for a test.
    float x0 = -1.0f, y0 = 1.0f;
    float x1 = -1.0f + 2.0f * (200.0f / 1920.0f);
    float y1 =  1.0f - 2.0f * (100.0f / 1080.0f);

    Vertex verts[4] = {
        { x0, y0,  0.0f, 0.0f },
        { x1, y0,  1.0f, 0.0f },
        { x0, y1,  0.0f, 1.0f },
        { x1, y1,  1.0f, 1.0f },
    };

    D3D11_BUFFER_DESC vbd = {};
    vbd.Usage          = D3D11_USAGE_DEFAULT;
    vbd.ByteWidth      = sizeof(verts);
    vbd.BindFlags      = D3D11_BIND_VERTEX_BUFFER;

    D3D11_SUBRESOURCE_DATA vd = {};
    vd.pSysMem = verts;
    g_pd3dDevice->CreateBuffer(&vbd, &vd, &g_vertexBuffer);
}

void RenderOverlay(ID3D11DeviceContext* ctx) {
    if (!g_overlaySRV || !g_vertexBuffer) return;

    // Save state
    ID3D11BlendState*       oldBS  = nullptr;
    ID3D11RasterizerState*  oldRS  = nullptr;
    float                   oldBF[4];
    UINT                    oldSM;
    ctx->OMGetBlendState(&oldBS, oldBF, &oldSM);
    ctx->RSGetState(&oldRS);

    // Set pipeline
    float bf[4] = { 1,1,1,1 };
    ctx->OMSetBlendState(g_blendState, bf, 0xFFFFFFFF);

    UINT stride = sizeof(Vertex), offset = 0;
    ctx->IASetVertexBuffers(0, 1, &g_vertexBuffer, &stride, &offset);
    ctx->IASetInputLayout(g_inputLayout);
    ctx->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);

    ctx->VSSetShader(g_vertexShader, nullptr, 0);
    ctx->PSSetShader(g_pixelShader, nullptr, 0);
    ctx->PSSetShaderResources(0, 1, &g_overlaySRV);
    ctx->PSSetSamplers(0, 1, &g_samplerState);

    ctx->Draw(4, 0);

    // Restore state
    ctx->OMSetBlendState(oldBS, oldBF, oldSM);
    ctx->RSSetState(oldRS);
    if (oldBS) oldBS->Release();
    if (oldRS) oldRS->Release();
}

void CleanupRenderResources() {
    if (g_vertexBuffer)  { g_vertexBuffer->Release();  g_vertexBuffer  = nullptr; }
    if (g_inputLayout)   { g_inputLayout->Release();   g_inputLayout   = nullptr; }
    if (g_vertexShader)  { g_vertexShader->Release();  g_vertexShader  = nullptr; }
    if (g_pixelShader)   { g_pixelShader->Release();   g_pixelShader   = nullptr; }
    if (g_blendState)    { g_blendState->Release();    g_blendState    = nullptr; }
    if (g_samplerState)  { g_samplerState->Release();  g_samplerState  = nullptr; }
    if (g_overlaySRV)    { g_overlaySRV->Release();    g_overlaySRV    = nullptr; }
    if (g_overlayTexture){ g_overlayTexture->Release(); g_overlayTexture = nullptr; }
}
