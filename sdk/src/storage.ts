import type { ISDK } from "./types";

/**
 * Storage API — persistent key-value store scoped to your app.
 */
export class Storage {
  private sdk: ISDK;

  constructor(sdk: ISDK) {
    this.sdk = sdk;
  }

  /**
   * Store a value. Automatically scoped to the current app.
   */
  public async set(key: string, value: unknown): Promise<void> {
    await this.sdk.request("POST", `/storage/${this.sdk.getAppId()}/${key}`, {
      value,
    });
  }

  /**
   * Retrieve a stored value. Returns null if not found.
   */
  public async get<T = unknown>(key: string): Promise<T | null> {
    try {
      const response = await this.sdk.request(
        "GET",
        `/storage/${this.sdk.getAppId()}/${key}`,
      );
      return response["value"] as T;
    } catch {
      return null;
    }
  }

  /**
   * Delete a stored value.
   */
  public async delete(key: string): Promise<void> {
    await this.sdk.request(
      "DELETE",
      `/storage/${this.sdk.getAppId()}/${key}`,
    );
  }

  /**
   * Clear all storage for this app.
   */
  public async clear(): Promise<void> {
    await this.sdk.request("DELETE", `/storage/${this.sdk.getAppId()}`);
  }
}
