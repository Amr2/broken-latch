import type { ISDK, PlatformInfo } from "./types";

/**
 * Platform info API — query platform version and installed apps.
 */
export class Platform {
  private sdk: ISDK;

  constructor(sdk: ISDK) {
    this.sdk = sdk;
  }

  /**
   * Get platform version and list of all installed apps.
   */
  public async getInfo(): Promise<PlatformInfo> {
    const response = await this.sdk.request("GET", "/platform/info");
    return response as unknown as PlatformInfo;
  }
}
