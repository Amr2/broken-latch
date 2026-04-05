#include <d3d11.h>
#include <d3dcompiler.h>
#include <string.h>
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
    if (!g_pd3dDevice) return;

    // Create vertex shader
    ID3DBlob* vsBlob = nullptr;
    ID3DBlob* errorBlob = nullptr;
    
    HRESULT hr = D3DCompile(
        g_vertexShaderSrc, 
        strlen(g_vertexShaderSrc), 
        nullptr, 
        nullptr, 
        nullptr, 
        "main", 
        "vs_5_0", 
        0, 
        0, 
        &vsBlob, 
        &errorBlob
    );
    
    if (SUCCEEDED(hr)) {
        g_pd3dDevice->CreateVertexShader(
            vsBlob->GetBufferPointer(), 
            vsBlob->GetBufferSize(), 
            nullptr, 
            &g_vertexShader
        );
        vsBlob->Release();
    }
    if (errorBlob) errorBlob->Release();

    // Create pixel shader
    ID3DBlob* psBlob = nullptr;
    hr = D3DCompile(
        g_pixelShaderSrc, 
        strlen(g_pixelShaderSrc), 
        nullptr, 
        nullptr, 
        nullptr, 
        "main", 
        "ps_5_0", 
        0, 
        0, 
        &psBlob, 
        &errorBlob
    );
    
    if (SUCCEEDED(hr)) {
        g_pd3dDevice->CreatePixelShader(
            psBlob->GetBufferPointer(), 
            psBlob->GetBufferSize(), 
            nullptr, 
            &g_pixelShader
        );
        psBlob->Release();
    }
    if (errorBlob) errorBlob->Release();

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

void CleanupRenderResources() {
    if (g_vertexShader) { g_vertexShader->Release(); g_vertexShader = nullptr; }
    if (g_pixelShader) { g_pixelShader->Release(); g_pixelShader = nullptr; }
    if (g_blendState) { g_blendState->Release(); g_blendState = nullptr; }
    if (g_samplerState) { g_samplerState->Release(); g_samplerState = nullptr; }
    if (g_overlaySRV) { g_overlaySRV->Release(); g_overlaySRV = nullptr; }
    if (g_overlayTexture) { g_overlayTexture->Release(); g_overlayTexture = nullptr; }
}
