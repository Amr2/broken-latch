# Task 04: Backend - Enemy Profiles Feature

**App: Hunter Mode**  
**Dependencies:** Task 01, 02, 03  
**Estimated Complexity:** High  
**Priority:** P0 (Core feature)

---

## Objective

Implement the backend route and service that powers the **Loading Screen Panel**. When 5 enemy summoner names + champion IDs are provided, this service fetches their match history from Riot API, analyzes their performance on those specific champions, and returns enriched `PlayerProfile` objects with insights like win rate, KDA, playstyle tags, and threat level.

---

## Context

This is Hunter Mode's flagship pre-game feature. During the loading screen (before minions spawn), the frontend calls `GET /enemies` with the enemy team roster. The backend must:

1. Call Riot Summoner API to get PUUIDs for each player
2. Call Match v5 API to get last 10 games per player on their champion
3. Call Champion Mastery API to get mastery level
4. Call League API to get their rank
5. Analyze match data to derive:
   - Win rate on champion
   - Average KDA, CS/min, vision score
   - Most common build path
   - Playstyle tag (Aggressive/Farm-focused/Roam-heavy)
   - Recent form (win streak, loss streak)
   - Threat level (1-5 based on rank + mastery + form)
6. Return all 5 profiles within 2 seconds

This data is cached aggressively (Task 03) to minimize API calls.

---

## What You Need to Build

### 1. Route (`backend/routes/enemies.js`)

```javascript
const express = require("express");
const router = express.Router();
const { buildPlayerProfiles } = require("../services/playerProfile");

/**
 * GET /enemies
 * Query params:
 *   - summonerNames: comma-separated (e.g. "Player1,Player2,Player3,Player4,Player5")
 *   - championIds: comma-separated (e.g. "64,201,157,238,67")
 *   - region: e.g. "na1"
 *
 * Returns: Array of 5 PlayerProfile objects
 */
router.get("/", async (req, res) => {
  try {
    const { summonerNames, championIds, region } = req.query;

    // Validate inputs
    if (!summonerNames || !championIds || !region) {
      return res.status(400).json({
        error:
          "Missing required parameters: summonerNames, championIds, region",
      });
    }

    const names = summonerNames.split(",").map((n) => n.trim());
    const champs = championIds.split(",").map((c) => parseInt(c.trim()));

    if (names.length !== 5 || champs.length !== 5) {
      return res.status(400).json({
        error: "Must provide exactly 5 summoner names and 5 champion IDs",
      });
    }

    // Build profiles for all 5 enemies in parallel
    const profiles = await buildPlayerProfiles(names, champs, region);

    res.json({ profiles });
  } catch (error) {
    console.error("[Enemies Route] Error:", error);
    res.status(500).json({
      error: "Failed to fetch enemy profiles",
      message: error.message,
    });
  }
});

module.exports = router;
```

### 2. Player Profile Service (`backend/services/playerProfile.js`)

This is the core analysis logic:

```javascript
const riot = require("./riotApi");

/**
 * Build profiles for multiple players in parallel
 */
async function buildPlayerProfiles(summonerNames, championIds, region) {
  const promises = summonerNames.map((name, index) =>
    buildPlayerProfile(name, championIds[index], region),
  );

  return Promise.all(promises);
}

/**
 * Build a single player profile
 */
async function buildPlayerProfile(
  summonerName,
  championId,
  region,
  matchCount = 10,
) {
  try {
    // 1. Get summoner data
    const summoner = await riot.getSummonerByName(summonerName, region);

    // 2. Get rank data
    const leagueEntries = await riot.getLeagueEntries(summoner.id, region);
    const soloQRank = leagueEntries.find(
      (e) => e.queueType === "RANKED_SOLO_5x5",
    );
    const rank = soloQRank ? `${soloQRank.tier} ${soloQRank.rank}` : "Unranked";

    // 3. Get champion mastery
    const masteries = await riot.getChampionMasteries(summoner.id, region);
    const championMastery = masteries.find(
      (m) => m.championId === championId,
    ) || {
      championLevel: 0,
      championPoints: 0,
    };

    // 4. Get match history (last 20 matches for filtering)
    const matchIds = await riot.getMatchIdsByPuuid(summoner.puuid, region, 20);

    // 5. Get full match data
    const matches = await Promise.all(
      matchIds.map((id) => riot.getMatch(id, region)),
    );

    // 6. Filter matches for this specific champion
    const championMatches = matches
      .filter((match) => {
        const participant = match.info.participants.find(
          (p) => p.puuid === summoner.puuid,
        );
        return participant && participant.championId === championId;
      })
      .slice(0, matchCount); // Take first N matches on this champion

    // 7. Analyze match data
    const analysis = analyzeMatches(
      championMatches,
      summoner.puuid,
      championId,
    );

    // 8. Build profile object
    return {
      summonerName,
      puuid: summoner.puuid,
      rank,
      championId,
      masteryLevel: championMastery.championLevel,
      masteryPoints: championMastery.championPoints,
      ...analysis,
    };
  } catch (error) {
    console.error(
      `[Player Profile] Error building profile for ${summonerName}:`,
      error.message,
    );

    // Return fallback profile on error
    return {
      summonerName,
      puuid: null,
      rank: "Unknown",
      championId,
      masteryLevel: 0,
      masteryPoints: 0,
      error: true,
      errorMessage: "Profile unavailable",
      winRate: 0,
      avgKda: 0,
      avgCsPerMin: 0,
      avgVisionScore: 0,
      avgGameDurationMins: 0,
      mostCommonBuild: [],
      mostCommonSummonerSpells: [0, 0],
      playstyleTag: "Unknown",
      recentForm: "Mixed",
      threatLevel: 1,
    };
  }
}

/**
 * Analyze match history to extract insights
 */
function analyzeMatches(matches, puuid, championId) {
  if (matches.length === 0) {
    return {
      gamesPlayed: 0,
      winRate: 0,
      avgKda: 0,
      avgCsPerMin: 0,
      avgVisionScore: 0,
      avgGameDurationMins: 0,
      mostCommonBuild: [],
      mostCommonSummonerSpells: [0, 0],
      playstyleTag: "Unknown",
      recentForm: "Mixed",
      threatLevel: 1,
    };
  }

  let wins = 0;
  let totalKills = 0;
  let totalDeaths = 0;
  let totalAssists = 0;
  let totalCs = 0;
  let totalVisionScore = 0;
  let totalGameDuration = 0;
  const builds = [];
  const summonerSpells = [];

  matches.forEach((match) => {
    const participant = match.info.participants.find((p) => p.puuid === puuid);
    if (!participant) return;

    // Basic stats
    if (participant.win) wins++;
    totalKills += participant.kills;
    totalDeaths += participant.deaths;
    totalAssists += participant.assists;
    totalCs +=
      participant.totalMinionsKilled + participant.neutralMinionsKilled;
    totalVisionScore += participant.visionScore;
    totalGameDuration += match.info.gameDuration / 60; // Convert to minutes

    // Build path (first 3 completed items)
    const build = [
      participant.item0,
      participant.item1,
      participant.item2,
    ].filter((item) => item > 0);
    builds.push(build.join(","));

    // Summoner spells
    summonerSpells.push(
      [participant.summoner1Id, participant.summoner2Id].sort().join(","),
    );
  });

  const gamesPlayed = matches.length;
  const winRate = wins / gamesPlayed;
  const avgKda =
    totalDeaths === 0
      ? totalKills + totalAssists
      : (totalKills + totalAssists) / totalDeaths;
  const avgCsPerMin = totalCs / totalGameDuration;
  const avgVisionScore = totalVisionScore / gamesPlayed;
  const avgGameDurationMins = totalGameDuration / gamesPlayed;

  // Most common build (mode of builds array)
  const mostCommonBuild = getMostCommon(builds)
    .split(",")
    .map((id) => parseInt(id))
    .filter((id) => id > 0);

  // Most common summoner spells
  const mostCommonSpells = getMostCommon(summonerSpells)
    .split(",")
    .map((id) => parseInt(id));

  // Playstyle tag (heuristic based on stats)
  const playstyleTag = derivePlaystyle(avgKda, avgCsPerMin, avgVisionScore);

  // Recent form (last 5 games)
  const recentForm = deriveRecentForm(matches.slice(0, 5), puuid);

  // Threat level (1-5 based on win rate, KDA, mastery)
  const threatLevel = deriveThreatLevel(winRate, avgKda, gamesPlayed);

  return {
    gamesPlayed,
    winRate,
    avgKda: parseFloat(avgKda.toFixed(2)),
    avgCsPerMin: parseFloat(avgCsPerMin.toFixed(1)),
    avgVisionScore: parseFloat(avgVisionScore.toFixed(1)),
    avgGameDurationMins: parseFloat(avgGameDurationMins.toFixed(1)),
    mostCommonBuild,
    mostCommonSummonerSpells: mostCommonSpells,
    playstyleTag,
    recentForm,
    threatLevel,
  };
}

/**
 * Get most common element in array
 */
function getMostCommon(arr) {
  const counts = {};
  arr.forEach((item) => {
    counts[item] = (counts[item] || 0) + 1;
  });

  let maxCount = 0;
  let mostCommon = arr[0];
  for (const [item, count] of Object.entries(counts)) {
    if (count > maxCount) {
      maxCount = count;
      mostCommon = item;
    }
  }

  return mostCommon;
}

/**
 * Derive playstyle from stats
 */
function derivePlaystyle(kda, csPerMin, visionScore) {
  // High KDA + low CS = aggressive fighter
  if (kda > 3.5 && csPerMin < 6) {
    return "Aggressive Early";
  }

  // High CS + low KDA = farm-focused
  if (csPerMin > 7 && kda < 2.5) {
    return "Farm Focused";
  }

  // High vision + moderate KDA = roam-heavy support
  if (visionScore > 50 && kda > 2.5) {
    return "Roam Heavy";
  }

  return "Balanced";
}

/**
 * Derive recent form from last 5 games
 */
function deriveRecentForm(recentMatches, puuid) {
  const results = recentMatches.map((match) => {
    const participant = match.info.participants.find((p) => p.puuid === puuid);
    return participant?.win ? "W" : "L";
  });

  const wins = results.filter((r) => r === "W").length;

  if (wins >= 4) return "Win Streak";
  if (wins <= 1) return "Loss Streak";
  return "Mixed";
}

/**
 * Calculate threat level (1-5)
 */
function deriveThreatLevel(winRate, kda, gamesPlayed) {
  // Not enough data
  if (gamesPlayed < 3) return 1;

  let score = 0;

  // Win rate contributes 0-2 points
  if (winRate >= 0.6) score += 2;
  else if (winRate >= 0.5) score += 1;

  // KDA contributes 0-2 points
  if (kda >= 4) score += 2;
  else if (kda >= 2.5) score += 1;

  // Experience contributes 0-1 point
  if (gamesPlayed >= 20) score += 1;

  // Map score (0-5) to threat level (1-5)
  return Math.max(1, Math.min(5, score));
}

module.exports = {
  buildPlayerProfile,
  buildPlayerProfiles,
};
```

### 3. Register Route in Server (`backend/server.js`)

Add to existing server setup:

```javascript
const enemyRoutes = require("./routes/enemies");

// ... existing code ...

app.use("/enemies", enemyRoutes);
```

---

## Integration Points

### From Task 02 (Riot API Client):

- Uses `riot.getSummonerByName()`
- Uses `riot.getLeagueEntries()`
- Uses `riot.getChampionMasteries()`
- Uses `riot.getMatchIdsByPuuid()`
- Uses `riot.getMatch()`

### From Task 03 (Caching):

- All Riot API calls are automatically cached via the `riotGet()` wrapper
- Cache TTL: 10 minutes for summoner data, 5 minutes for matches

### For Task 10 (Frontend Loading Panel):

- Frontend calls `GET /enemies?summonerNames=...&championIds=...&region=...`
- Receives `{ profiles: PlayerProfile[] }` response
- Renders 5 `EnemyRow` components with the data

---

## Data Model

### PlayerProfile Object (returned by this service):

```typescript
interface PlayerProfile {
  summonerName: string;
  puuid: string | null;
  rank: string; // "Gold II", "Platinum IV", etc.
  championId: number;
  masteryLevel: number; // 0-7
  masteryPoints: number;
  gamesPlayed: number; // On this champion
  winRate: number; // 0.0 to 1.0
  avgKda: number; // e.g. 3.45
  avgCsPerMin: number; // e.g. 6.8
  avgVisionScore: number; // e.g. 28.5
  avgGameDurationMins: number; // e.g. 32.4
  mostCommonBuild: number[]; // Item IDs, e.g. [3078, 3031, 3087]
  mostCommonSummonerSpells: [number, number]; // Spell IDs, e.g. [4, 7]
  playstyleTag: string; // "Aggressive Early" | "Farm Focused" | "Roam Heavy" | "Balanced"
  recentForm: string; // "Win Streak" | "Loss Streak" | "Mixed"
  threatLevel: number; // 1-5
  error?: boolean; // True if profile fetch failed
  errorMessage?: string; // Error description
}
```

---

## Testing Requirements

### Unit Tests (`backend/services/playerProfile.test.js`)

```javascript
const { analyzeMatches, derivePlaystyle, deriveThreatLevel } = require('./playerProfile');

describe('Player Profile Analysis', () => {
    test('analyzeMatches calculates correct win rate', () => {
        const mockMatches = [
            { info: { participants: [{ puuid: 'test', win: true, kills: 5, deaths: 2, assists: 8, ... }] } },
            { info: { participants: [{ puuid: 'test', win: false, kills: 2, deaths: 5, assists: 3, ... }] } },
            { info: { participants: [{ puuid: 'test', win: true, kills: 10, deaths: 1, assists: 12, ... }] } }
        ];

        const result = analyzeMatches(mockMatches, 'test', 64);
        expect(result.winRate).toBeCloseTo(0.67, 2);
        expect(result.gamesPlayed).toBe(3);
    });

    test('derivePlaystyle identifies aggressive players', () => {
        const style = derivePlaystyle(4.5, 5.2, 30);
        expect(style).toBe('Aggressive Early');
    });

    test('deriveThreatLevel assigns correct threat', () => {
        const level = deriveThreatLevel(0.65, 4.2, 25);
        expect(level).toBe(5); // High win rate + high KDA + experience
    });

    test('handles empty match history gracefully', () => {
        const result = analyzeMatches([], 'test', 64);
        expect(result.gamesPlayed).toBe(0);
        expect(result.winRate).toBe(0);
        expect(result.threatLevel).toBe(1);
    });
});
```

### Integration Tests (`backend/routes/enemies.test.js`)

```javascript
const request = require("supertest");
const app = require("../server"); // Your Express app

describe("GET /enemies", () => {
  test("returns 5 profiles for valid input", async () => {
    const response = await request(app).get("/enemies").query({
      summonerNames: "Player1,Player2,Player3,Player4,Player5",
      championIds: "64,201,157,238,67",
      region: "na1",
    });

    expect(response.status).toBe(200);
    expect(response.body.profiles).toHaveLength(5);
    expect(response.body.profiles[0]).toHaveProperty("summonerName");
    expect(response.body.profiles[0]).toHaveProperty("winRate");
    expect(response.body.profiles[0]).toHaveProperty("threatLevel");
  });

  test("returns 400 for invalid input", async () => {
    const response = await request(app).get("/enemies").query({
      summonerNames: "Player1,Player2", // Only 2, needs 5
      championIds: "64,201",
      region: "na1",
    });

    expect(response.status).toBe(400);
  });

  test("handles Riot API errors gracefully", async () => {
    // Mock Riot API to return error
    // Verify fallback profiles are returned
  });
});
```

### Manual Testing with Postman/cURL

```bash
# Test with real summoner names (replace with actual names in your region)
curl "http://localhost:45679/enemies?summonerNames=Doublelift,Bjergsen,Sneaky,Meteos,Aphromoo&championIds=222,4,81,64,412&region=na1"

# Expected response time: <2 seconds
# Expected response: { "profiles": [ ... 5 PlayerProfile objects ... ] }
```

---

## Acceptance Criteria

✅ **Complete when:**

1. Route `/enemies` accepts query params and validates input
2. Service fetches data from Riot API for all 5 players in parallel
3. Match history is filtered correctly by champion
4. Win rate, KDA, CS/min, vision score calculations are accurate
5. Most common build and summoner spells are identified correctly
6. Playstyle tag is derived using heuristics
7. Recent form is determined from last 5 games
8. Threat level (1-5) is calculated based on win rate, KDA, experience
9. Errors are handled gracefully (returns fallback profile)
10. Response time is <2 seconds for 5 profiles
11. All unit tests pass
12. Integration test passes
13. Manual test with real summoner names works

---

## Performance Requirements

- **Response time**: <2 seconds for 5 profiles (with warm cache: <500ms)
- **Riot API calls**: ~50 calls total (10 per player) - respect rate limits via Task 02
- **Memory**: <20MB additional during processing
- **Cache hit rate**: >80% for repeated matches within 5 minutes

---

## Error Handling

### Scenarios to handle:

1. **Summoner not found**: Return fallback profile with `error: true`
2. **No ranked games**: Set rank to "Unranked"
3. **No games on champion**: Set gamesPlayed: 0, all stats to 0
4. **Riot API rate limit**: Wait and retry (handled in Task 02)
5. **Riot API down**: Return cached data if available, else fallback profile

---

## Dependencies for Next Tasks

- **Task 05** (Win Condition) needs enemy championIds from this data
- **Task 06** (Hunter Focus) reuses this service for deep-dive on 1 player
- **Task 10** (Loading Panel) displays this data in UI

---

## Files to Create/Modify

### New Files:

- `backend/routes/enemies.js`
- `backend/services/playerProfile.js`
- `backend/services/playerProfile.test.js`
- `backend/routes/enemies.test.js`

### Modified Files:

- `backend/server.js` (register route)

---

## Expected Time: 8-10 hours

## Difficulty: High (Complex data analysis + API orchestration)
