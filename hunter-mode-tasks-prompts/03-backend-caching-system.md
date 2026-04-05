# Task 03: Backend - Caching System

**App: Hunter Mode**  
**Dependencies:** Task 01, 02  
**Estimated Complexity:** Medium  
**Priority:** P0 (Performance critical)

---

## Objective

Implement a file-based caching layer using LowDB that wraps all Riot API calls. This dramatically reduces redundant API requests, improves response times, and prevents hitting rate limits during normal usage.

---

## Context

The Riot API is the bottleneck in Hunter Mode. Every summoner lookup, match fetch, and mastery query counts against our rate limit. Without caching:

- Loading 5 enemy profiles = ~50 API calls
- Each subsequent request for the same data wastes rate limit tokens
- Response times suffer from network latency

With caching:

- First request: ~50 API calls, stored in cache
- Repeat requests within TTL: 0 API calls, instant response from disk
- Cache hit rate target: >80%

Cache data is stored in `backend/data/cache.json` as a simple key-value store with TTL timestamps. LowDB handles file I/O automatically.

---

## What You Need to Build

### 1. Cache Service (`backend/services/cache.js`)

```javascript
const { Low } = require("lowdb");
const { JSONFile } = require("lowdb/node");
const path = require("path");

const CACHE_FILE = path.join(__dirname, "../data/cache.json");

// Default cache structure
const defaultData = { entries: {} };

// Initialize LowDB
const adapter = new JSONFile(CACHE_FILE);
const db = new Low(adapter, defaultData);

/**
 * Initialize the cache (call once on server startup)
 */
async function init() {
  await db.read();
  db.data ||= defaultData;
  await db.write();
  console.log("[Cache] Initialized at", CACHE_FILE);
}

/**
 * Get a value from cache
 * @param {string} key - Cache key
 * @returns {any|null} - Cached value or null if expired/missing
 */
async function get(key) {
  await db.read();
  const entry = db.data.entries[key];

  if (!entry) {
    return null; // Cache miss
  }

  // Check if expired
  if (Date.now() > entry.expiresAt) {
    // Remove expired entry
    delete db.data.entries[key];
    await db.write();
    return null;
  }

  console.log(`[Cache] HIT: ${key}`);
  return entry.value;
}

/**
 * Set a value in cache with TTL
 * @param {string} key - Cache key
 * @param {any} value - Value to cache
 * @param {number} ttlMinutes - Time to live in minutes
 */
async function set(key, value, ttlMinutes = 10) {
  await db.read();
  db.data.entries[key] = {
    value,
    expiresAt: Date.now() + ttlMinutes * 60 * 1000,
    createdAt: Date.now(),
  };
  await db.write();
  console.log(`[Cache] SET: ${key} (TTL: ${ttlMinutes}m)`);
}

/**
 * Delete a specific key
 */
async function del(key) {
  await db.read();
  delete db.data.entries[key];
  await db.write();
  console.log(`[Cache] DEL: ${key}`);
}

/**
 * Clear all cache entries
 */
async function clear() {
  await db.read();
  db.data.entries = {};
  await db.write();
  console.log("[Cache] Cleared all entries");
}

/**
 * Get cache statistics
 */
async function getStats() {
  await db.read();
  const entries = Object.values(db.data.entries);
  const now = Date.now();
  const valid = entries.filter((e) => e.expiresAt > now);
  const expired = entries.filter((e) => e.expiresAt <= now);

  return {
    totalEntries: entries.length,
    validEntries: valid.length,
    expiredEntries: expired.length,
    cacheFilePath: CACHE_FILE,
  };
}

/**
 * Clean up expired entries (call periodically)
 */
async function cleanup() {
  await db.read();
  const now = Date.now();
  const before = Object.keys(db.data.entries).length;

  for (const [key, entry] of Object.entries(db.data.entries)) {
    if (entry.expiresAt <= now) {
      delete db.data.entries[key];
    }
  }

  await db.write();
  const after = Object.keys(db.data.entries).length;
  const removed = before - after;

  if (removed > 0) {
    console.log(`[Cache] Cleaned up ${removed} expired entries`);
  }

  return { removed, remaining: after };
}

module.exports = {
  init,
  get,
  set,
  del,
  clear,
  getStats,
  cleanup,
};
```

### 2. Cached Riot API Wrapper (`backend/services/cachedRiotApi.js`)

This wraps the raw Riot API client from Task 02 with caching:

```javascript
const riot = require("./riotApi");
const cache = require("./cache");

/**
 * Cache TTL configuration (in minutes)
 */
const TTL = {
  summoner: 10, // Summoner data changes rarely
  league: 10, // Rank updates every few games
  mastery: 30, // Mastery grows slowly
  matchIds: 5, // Match list changes frequently
  match: 60, // Past matches never change
  championData: 1440, // Static data (1 day)
};

/**
 * Generic cache wrapper
 */
async function withCache(cacheKey, ttl, fetchFn) {
  // Try cache first
  const cached = await cache.get(cacheKey);
  if (cached !== null) {
    return cached;
  }

  // Cache miss - fetch from API
  const data = await fetchFn();

  // Store in cache
  await cache.set(cacheKey, data, ttl);

  return data;
}

/**
 * Get summoner by name (cached)
 */
async function getSummonerByName(summonerName, region) {
  const cacheKey = `summoner:${region}:${summonerName.toLowerCase()}`;
  return withCache(cacheKey, TTL.summoner, () =>
    riot.getSummonerByName(summonerName, region),
  );
}

/**
 * Get summoner by PUUID (cached)
 */
async function getSummonerByPuuid(puuid, region) {
  const cacheKey = `summoner-puuid:${region}:${puuid}`;
  return withCache(cacheKey, TTL.summoner, () =>
    riot.getSummonerByPuuid(puuid, region),
  );
}

/**
 * Get league entries (cached)
 */
async function getLeagueEntries(summonerId, region) {
  const cacheKey = `league:${region}:${summonerId}`;
  return withCache(cacheKey, TTL.league, () =>
    riot.getLeagueEntries(summonerId, region),
  );
}

/**
 * Get champion masteries (cached)
 */
async function getChampionMasteries(summonerId, region, count = 10) {
  const cacheKey = `mastery:${region}:${summonerId}:${count}`;
  return withCache(cacheKey, TTL.mastery, () =>
    riot.getChampionMasteries(summonerId, region, count),
  );
}

/**
 * Get match IDs (cached)
 */
async function getMatchIdsByPuuid(puuid, region, count = 20) {
  const cacheKey = `match-ids:${region}:${puuid}:${count}`;
  return withCache(cacheKey, TTL.matchIds, () =>
    riot.getMatchIdsByPuuid(puuid, region, count),
  );
}

/**
 * Get match data (cached with long TTL - matches never change)
 */
async function getMatch(matchId, region) {
  const cacheKey = `match:${matchId}`;
  return withCache(cacheKey, TTL.match, () => riot.getMatch(matchId, region));
}

/**
 * Get champion data from Data Dragon (cached for 1 day)
 */
async function getChampionData(version = "13.20.1") {
  const cacheKey = `champion-data:${version}`;
  return withCache(cacheKey, TTL.championData, () =>
    riot.getChampionData(version),
  );
}

module.exports = {
  getSummonerByName,
  getSummonerByPuuid,
  getLeagueEntries,
  getChampionMasteries,
  getMatchIdsByPuuid,
  getMatch,
  getChampionData,
};
```

### 3. Cache Cleanup Scheduler

Add to `backend/server.js`:

```javascript
const cache = require("./services/cache");

// Initialize cache on startup
(async () => {
  await cache.init();

  // Run cleanup every 10 minutes
  setInterval(
    async () => {
      await cache.cleanup();
    },
    10 * 60 * 1000,
  );
})();

// Cache stats endpoint (for debugging)
app.get("/cache/stats", async (req, res) => {
  const stats = await cache.getStats();
  res.json(stats);
});

// Cache clear endpoint (admin only - protect in production)
app.post("/cache/clear", async (req, res) => {
  await cache.clear();
  res.json({ success: true, message: "Cache cleared" });
});
```

---

## Integration Points

### From Task 02 (Riot API Client):

- Wraps all Riot API methods with caching layer
- Uses same method signatures

### For Task 04-07 (Backend Features):

- All features import from `cachedRiotApi` instead of `riotApi`
- Transparent caching — no code changes needed

---

## Testing Requirements

### Unit Tests (`backend/services/cache.test.js`)

```javascript
const cache = require("./cache");
const fs = require("fs");
const path = require("path");

const TEST_CACHE_FILE = path.join(__dirname, "../data/cache-test.json");

describe("Cache Service", () => {
  beforeEach(async () => {
    // Use test cache file
    await cache.init();
    await cache.clear();
  });

  test("set and get value", async () => {
    await cache.set("test-key", { data: "test-value" }, 10);
    const value = await cache.get("test-key");
    expect(value).toEqual({ data: "test-value" });
  });

  test("returns null for missing key", async () => {
    const value = await cache.get("non-existent-key");
    expect(value).toBeNull();
  });

  test("returns null for expired entry", async () => {
    // Set with 0.01 minute TTL (600ms)
    await cache.set("expiring-key", "value", 0.01);

    // Wait for expiration
    await new Promise((resolve) => setTimeout(resolve, 700));

    const value = await cache.get("expiring-key");
    expect(value).toBeNull();
  });

  test("cleanup removes expired entries", async () => {
    await cache.set("valid", "data", 10);
    await cache.set("expired", "data", 0.01);

    await new Promise((resolve) => setTimeout(resolve, 700));

    const result = await cache.cleanup();
    expect(result.removed).toBe(1);
    expect(result.remaining).toBe(1);
  });

  test("clear removes all entries", async () => {
    await cache.set("key1", "value1", 10);
    await cache.set("key2", "value2", 10);
    await cache.clear();

    const value1 = await cache.get("key1");
    const value2 = await cache.get("key2");

    expect(value1).toBeNull();
    expect(value2).toBeNull();
  });

  test("getStats returns correct counts", async () => {
    await cache.set("key1", "value1", 10);
    await cache.set("key2", "value2", 0.01);

    await new Promise((resolve) => setTimeout(resolve, 700));

    const stats = await cache.getStats();
    expect(stats.totalEntries).toBe(2);
    expect(stats.validEntries).toBe(1);
    expect(stats.expiredEntries).toBe(1);
  });
});
```

### Integration Tests (`backend/services/cachedRiotApi.test.js`)

```javascript
const cachedRiot = require("./cachedRiotApi");
const cache = require("./cache");

const API_KEY = process.env.HUNTER_MODE_RIOT_API_KEY;
const describeIfApiKey = API_KEY ? describe : describe.skip;

describeIfApiKey("Cached Riot API", () => {
  beforeEach(async () => {
    await cache.init();
    await cache.clear();
  });

  test("caches summoner data on first request", async () => {
    const summoner1 = await cachedRiot.getSummonerByName("Doublelift", "na1");
    const summoner2 = await cachedRiot.getSummonerByName("Doublelift", "na1");

    // Should return same object (from cache)
    expect(summoner1).toEqual(summoner2);
  });

  test("cache hit returns data without API call", async () => {
    // First call - cache miss
    await cachedRiot.getSummonerByName("Doublelift", "na1");

    // Second call - should be instant (cache hit)
    const start = Date.now();
    await cachedRiot.getSummonerByName("Doublelift", "na1");
    const elapsed = Date.now() - start;

    // Cache hit should be <10ms (API calls are 100-500ms)
    expect(elapsed).toBeLessThan(50);
  });

  test("match data is cached with long TTL", async () => {
    const summoner = await cachedRiot.getSummonerByName("Doublelift", "na1");
    const matchIds = await cachedRiot.getMatchIdsByPuuid(
      summoner.puuid,
      "na1",
      1,
    );
    const match = await cachedRiot.getMatch(matchIds[0], "na1");

    expect(match).toHaveProperty("info");

    // Verify it's cached
    const cachedMatch = await cache.get(`match:${matchIds[0]}`);
    expect(cachedMatch).toEqual(match);
  });
});
```

### Manual Testing

```bash
# Start server
npm start

# First request (cache miss)
time curl "http://localhost:45679/enemies?summonerNames=Player1&championIds=64&region=na1"
# Should take ~1-2 seconds

# Second request (cache hit)
time curl "http://localhost:45679/enemies?summonerNames=Player1&championIds=64&region=na1"
# Should take <100ms

# Check cache stats
curl "http://localhost:45679/cache/stats"
# Should show entries > 0
```

---

## Acceptance Criteria

✅ **Complete when:**

1. LowDB cache initializes on server startup
2. All Riot API methods have cached versions
3. Cache keys are unique and collision-free
4. TTL correctly expires old entries
5. Cache hit returns data without API call
6. Cache miss fetches from API and stores result
7. Cleanup removes expired entries
8. Stats endpoint shows correct cache metrics
9. Clear endpoint empties cache
10. All unit tests pass
11. All integration tests pass
12. Manual test shows 10x+ speedup on cache hit

---

## Performance Requirements

- **Cache hit response time**: <10ms
- **Cache write time**: <20ms
- **Cleanup time**: <100ms for 1000 entries
- **File size**: Grows ~1KB per cached summoner, ~10KB per match

---

## Error Handling

### Scenarios to handle:

1. **Cache file corrupted**: Re-initialize with empty cache
2. **Disk full**: Log error, continue without caching
3. **Write failure**: Log error, return data without caching
4. **Invalid cache data**: Return null (cache miss)

---

## Cache Key Patterns

```
summoner:{region}:{summonerName}
summoner-puuid:{region}:{puuid}
league:{region}:{summonerId}
mastery:{region}:{summonerId}:{count}
match-ids:{region}:{puuid}:{count}
match:{matchId}
champion-data:{version}
```

---

## Files to Create/Modify

### New Files:

- `backend/services/cache.js`
- `backend/services/cachedRiotApi.js`
- `backend/services/cache.test.js`
- `backend/services/cachedRiotApi.test.js`

### Modified Files:

- `backend/server.js` (add cache initialization + endpoints)

---

## Expected Time: 4-5 hours

## Difficulty: Medium (Straightforward implementation with testing)
