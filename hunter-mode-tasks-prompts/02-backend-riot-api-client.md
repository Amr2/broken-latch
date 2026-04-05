# Task 02: Backend - Riot API Client

**App: Hunter Mode**  
**Dependencies:** Task 01  
**Estimated Complexity:** High  
**Priority:** P0 (Core infrastructure)

---

## Objective

Build a robust Riot Games API client with automatic rate limiting, error handling, and retry logic. This service is the foundation for all data fetching in Hunter Mode — every backend task depends on it.

---

## Context

The Riot API has strict rate limits:

- **Application rate limit**: 20 requests per 1 second, 100 requests per 2 minutes
- **Method rate limits**: Vary by endpoint (e.g. Match-v5 has additional limits)

We must respect these limits to avoid getting our API key temporarily banned. This client implements:

1. **Token bucket rate limiter**: Ensures we never exceed 20/1s or 100/120s
2. **Automatic retry with exponential backoff**: Handles 429 (rate limit) and 503 (service unavailable) responses
3. **Request queue**: Batches multiple concurrent requests safely
4. **Regional routing**: Maps platform regions (na1, euw1) to regional endpoints (americas, europe)
5. **Error handling**: Graceful fallbacks for 404 (not found), 403 (forbidden key), 500 (server error)

All Riot API calls in future tasks go through this client.

---

## What You Need to Build

### 1. Rate Limiter Class (`backend/services/rateLimiter.js`)

```javascript
/**
 * Token Bucket Rate Limiter
 * Respects Riot API limits: 20/1s and 100/120s
 */
class RateLimiter {
  constructor() {
    this.shortWindow = {
      limit: 20,
      duration: 1000,
      tokens: 20,
      lastRefill: Date.now(),
    };
    this.longWindow = {
      limit: 100,
      duration: 120000,
      tokens: 100,
      lastRefill: Date.now(),
    };
    this.queue = [];
    this.processing = false;
  }

  /**
   * Refill tokens based on time elapsed
   */
  refillTokens(window) {
    const now = Date.now();
    const elapsed = now - window.lastRefill;
    const tokensToAdd = Math.floor((elapsed / window.duration) * window.limit);

    if (tokensToAdd > 0) {
      window.tokens = Math.min(window.limit, window.tokens + tokensToAdd);
      window.lastRefill = now;
    }
  }

  /**
   * Check if we can make a request
   */
  canMakeRequest() {
    this.refillTokens(this.shortWindow);
    this.refillTokens(this.longWindow);
    return this.shortWindow.tokens > 0 && this.longWindow.tokens > 0;
  }

  /**
   * Consume a token from both buckets
   */
  consumeToken() {
    this.shortWindow.tokens -= 1;
    this.longWindow.tokens -= 1;
  }

  /**
   * Wait until we can make a request
   */
  async waitForToken() {
    return new Promise((resolve) => {
      const check = () => {
        if (this.canMakeRequest()) {
          this.consumeToken();
          resolve();
        } else {
          // Check again in 50ms
          setTimeout(check, 50);
        }
      };
      check();
    });
  }

  /**
   * Throttle a request — waits for available token before executing
   */
  async throttle() {
    await this.waitForToken();
  }
}

module.exports = RateLimiter;
```

### 2. Riot API Client (`backend/services/riotApi.js`)

```javascript
const axios = require("axios");
const RateLimiter = require("./rateLimiter");

const API_KEY = process.env.HUNTER_MODE_RIOT_API_KEY;

if (!API_KEY) {
  throw new Error(
    "HUNTER_MODE_RIOT_API_KEY not found in environment variables",
  );
}

// Regional routing map
const REGIONS = {
  na1: { platform: "na1", regional: "americas" },
  br1: { platform: "br1", regional: "americas" },
  la1: { platform: "la1", regional: "americas" },
  la2: { platform: "la2", regional: "americas" },
  euw1: { platform: "euw1", regional: "europe" },
  eun1: { platform: "eun1", regional: "europe" },
  tr1: { platform: "tr1", regional: "europe" },
  ru: { platform: "ru", regional: "europe" },
  kr: { platform: "kr", regional: "asia" },
  jp1: { platform: "jp1", regional: "asia" },
  oc1: { platform: "oc1", regional: "sea" },
  ph2: { platform: "ph2", regional: "sea" },
  sg2: { platform: "sg2", regional: "sea" },
  th2: { platform: "th2", regional: "sea" },
  tw2: { platform: "tw2", regional: "sea" },
  vn2: { platform: "vn2", regional: "sea" },
};

// Global rate limiter instance
const rateLimiter = new RateLimiter();

/**
 * Make a rate-limited request to Riot API with retry logic
 */
async function riotRequest(url, maxRetries = 3) {
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      // Wait for rate limit token
      await rateLimiter.throttle();

      // Make request
      const response = await axios.get(url, {
        headers: {
          "X-Riot-Token": API_KEY,
        },
        timeout: 5000,
      });

      return response.data;
    } catch (error) {
      const status = error.response?.status;

      // Handle rate limit exceeded (should rarely happen with our limiter)
      if (status === 429) {
        const retryAfter = parseInt(error.response.headers["retry-after"]) || 1;
        console.warn(
          `[Riot API] Rate limit hit, retrying after ${retryAfter}s`,
        );
        await sleep(retryAfter * 1000);
        continue;
      }

      // Handle service unavailable
      if (status === 503) {
        const backoff = Math.pow(2, attempt) * 1000; // Exponential backoff
        console.warn(
          `[Riot API] Service unavailable, retrying in ${backoff}ms`,
        );
        await sleep(backoff);
        continue;
      }

      // Handle not found (legitimate response, don't retry)
      if (status === 404) {
        throw new Error("Resource not found");
      }

      // Handle forbidden (bad API key)
      if (status === 403) {
        throw new Error("Invalid or expired API key");
      }

      // Other errors
      if (attempt === maxRetries - 1) {
        throw error; // Last attempt, throw the error
      }

      // Retry with backoff
      const backoff = Math.pow(2, attempt) * 1000;
      console.warn(`[Riot API] Request failed, retrying in ${backoff}ms`);
      await sleep(backoff);
    }
  }

  throw new Error("Max retries exceeded");
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ─── API Methods ─────────────────────────────────────────────────

/**
 * Get summoner by name (Summoner-v4)
 */
async function getSummonerByName(summonerName, region) {
  const { platform } = REGIONS[region];
  const url = `https://${platform}.api.riotgames.com/lol/summoner/v4/summoners/by-name/${encodeURIComponent(
    summonerName,
  )}`;
  return riotRequest(url);
}

/**
 * Get summoner by PUUID (Summoner-v4)
 */
async function getSummonerByPuuid(puuid, region) {
  const { platform } = REGIONS[region];
  const url = `https://${platform}.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/${puuid}`;
  return riotRequest(url);
}

/**
 * Get league entries for a summoner (League-v4)
 */
async function getLeagueEntries(summonerId, region) {
  const { platform } = REGIONS[region];
  const url = `https://${platform}.api.riotgames.com/lol/league/v4/entries/by-summoner/${summonerId}`;
  return riotRequest(url);
}

/**
 * Get champion masteries for a summoner (Champion Mastery-v4)
 */
async function getChampionMasteries(summonerId, region, count = 10) {
  const { platform } = REGIONS[region];
  const url = `https://${platform}.api.riotgames.com/lol/champion-mastery/v4/champion-masteries/by-summoner/${summonerId}/top?count=${count}`;
  return riotRequest(url);
}

/**
 * Get match IDs for a PUUID (Match-v5)
 */
async function getMatchIdsByPuuid(puuid, region, count = 20) {
  const { regional } = REGIONS[region];
  const url = `https://${regional}.api.riotgames.com/lol/match/v5/matches/by-puuid/${puuid}/ids?count=${count}`;
  return riotRequest(url);
}

/**
 * Get full match data by match ID (Match-v5)
 */
async function getMatch(matchId, region) {
  const { regional } = REGIONS[region];
  const url = `https://${regional}.api.riotgames.com/lol/match/v5/matches/${matchId}`;
  return riotRequest(url);
}

/**
 * Get Data Dragon champion data (not from Riot API, but useful for analysis)
 */
async function getChampionData(version = "13.20.1") {
  const url = `https://ddragon.leagueoflegends.com/cdn/${version}/data/en_US/champion.json`;
  // No rate limiting needed for Data Dragon
  const response = await axios.get(url);
  return response.data;
}

module.exports = {
  getSummonerByName,
  getSummonerByPuuid,
  getLeagueEntries,
  getChampionMasteries,
  getMatchIdsByPuuid,
  getMatch,
  getChampionData,
  REGIONS,
};
```

---

## Integration Points

### From Task 01 (Project Setup):

- Uses `HUNTER_MODE_RIOT_API_KEY` from .env file
- Backend server is running on port 45679

### For Task 03 (Caching):

- Caching layer will wrap these methods to avoid redundant API calls

### For Task 04-07 (All Backend Features):

- Every feature route imports and uses these methods

---

## Testing Requirements

### Unit Tests (`backend/services/rateLimiter.test.js`)

```javascript
const RateLimiter = require("./rateLimiter");

describe("RateLimiter", () => {
  test("allows requests within limits", async () => {
    const limiter = new RateLimiter();
    expect(limiter.canMakeRequest()).toBe(true);
  });

  test("blocks requests when tokens exhausted", async () => {
    const limiter = new RateLimiter();
    // Consume all short window tokens
    for (let i = 0; i < 20; i++) {
      limiter.consumeToken();
    }
    expect(limiter.canMakeRequest()).toBe(false);
  });

  test("refills tokens over time", async () => {
    const limiter = new RateLimiter();
    // Consume all tokens
    for (let i = 0; i < 20; i++) {
      limiter.consumeToken();
    }
    expect(limiter.canMakeRequest()).toBe(false);

    // Wait for refill (1 second)
    await new Promise((resolve) => setTimeout(resolve, 1100));
    limiter.refillTokens(limiter.shortWindow);
    expect(limiter.canMakeRequest()).toBe(true);
  });

  test("throttle waits for available token", async () => {
    const limiter = new RateLimiter();
    const start = Date.now();

    // Make 21 requests (exceeds short window limit of 20)
    const requests = [];
    for (let i = 0; i < 21; i++) {
      requests.push(limiter.throttle());
    }

    await Promise.all(requests);
    const elapsed = Date.now() - start;

    // Should have taken at least 50ms (waiting for refill)
    expect(elapsed).toBeGreaterThan(40);
  });
});
```

### Integration Tests (`backend/services/riotApi.test.js`)

```javascript
const riot = require("./riotApi");

// These tests require a valid API key in .env
// Skip if API key is not set
const API_KEY = process.env.HUNTER_MODE_RIOT_API_KEY;
const describeIfApiKey = API_KEY ? describe : describe.skip;

describeIfApiKey("Riot API Client", () => {
  test("getSummonerByName returns valid summoner", async () => {
    const summoner = await riot.getSummonerByName("Doublelift", "na1");
    expect(summoner).toHaveProperty("puuid");
    expect(summoner).toHaveProperty("id");
    expect(summoner).toHaveProperty("name");
  });

  test("getLeagueEntries returns rank data", async () => {
    const summoner = await riot.getSummonerByName("Doublelift", "na1");
    const entries = await riot.getLeagueEntries(summoner.id, "na1");
    expect(Array.isArray(entries)).toBe(true);
  });

  test("getMatchIdsByPuuid returns match IDs", async () => {
    const summoner = await riot.getSummonerByName("Doublelift", "na1");
    const matchIds = await riot.getMatchIdsByPuuid(summoner.puuid, "na1", 5);
    expect(Array.isArray(matchIds)).toBe(true);
    expect(matchIds.length).toBeGreaterThan(0);
  });

  test("getMatch returns full match data", async () => {
    const summoner = await riot.getSummonerByName("Doublelift", "na1");
    const matchIds = await riot.getMatchIdsByPuuid(summoner.puuid, "na1", 1);
    const match = await riot.getMatch(matchIds[0], "na1");
    expect(match).toHaveProperty("info");
    expect(match.info).toHaveProperty("participants");
  });

  test("handles 404 for non-existent summoner", async () => {
    await expect(
      riot.getSummonerByName("ThisSummonerDoesNotExist123456789", "na1"),
    ).rejects.toThrow("Resource not found");
  });

  test("respects rate limits (20 requests should complete)", async () => {
    const summoner = await riot.getSummonerByName("Doublelift", "na1");

    const requests = [];
    for (let i = 0; i < 20; i++) {
      requests.push(riot.getLeagueEntries(summoner.id, "na1"));
    }

    const results = await Promise.all(requests);
    expect(results).toHaveLength(20);
  }, 30000); // 30 second timeout
});
```

### Manual Testing

```bash
# Start backend server
cd backend
npm start

# In another terminal, test with curl
curl "http://localhost:45679/test-riot-api"

# Expected: Should return summoner data without errors
```

Add a test endpoint to `backend/server.js`:

```javascript
// Test endpoint (remove in production)
app.get("/test-riot-api", async (req, res) => {
  const riot = require("./services/riotApi");
  try {
    const summoner = await riot.getSummonerByName("Doublelift", "na1");
    res.json({ success: true, summoner });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});
```

---

## Acceptance Criteria

✅ **Complete when:**

1. RateLimiter class correctly implements token bucket algorithm
2. RateLimiter respects both 20/1s and 100/120s limits
3. All Riot API methods are implemented (6 methods)
4. Regional routing works correctly for all regions
5. Rate limiting prevents exceeding API limits
6. 429 responses trigger automatic retry with backoff
7. 503 responses trigger exponential backoff
8. 404 responses throw "Resource not found" error
9. 403 responses throw "Invalid API key" error
10. All unit tests pass
11. All integration tests pass (with valid API key)
12. Manual test endpoint returns valid data

---

## Performance Requirements

- **Rate limit accuracy**: Never exceed 20/1s or 100/120s
- **Retry delay**: Exponential backoff starting at 1s
- **Request timeout**: 5 seconds
- **Throughput**: Can handle 100 requests in 2 minutes without errors

---

## Error Handling

### Error Types:

1. **429 Rate Limit**: Retry after delay from header
2. **503 Service Unavailable**: Exponential backoff (1s, 2s, 4s)
3. **404 Not Found**: Throw error immediately (don't retry)
4. **403 Forbidden**: Throw error (invalid API key)
5. **500 Server Error**: Retry with backoff
6. **Network timeout**: Retry with backoff
7. **Missing API key**: Throw on module load

---

## Files to Create/Modify

### New Files:

- `backend/services/rateLimiter.js`
- `backend/services/riotApi.js`
- `backend/services/rateLimiter.test.js`
- `backend/services/riotApi.test.js`

### Modified Files:

- `backend/server.js` (add test endpoint temporarily)

---

## Expected Time: 6-8 hours

## Difficulty: High (Complex rate limiting + error handling logic)
