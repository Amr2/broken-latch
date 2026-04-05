# Task 07: Backend - Post-Game Feature

**App: Hunter Mode**  
**Dependencies:** Task 01, 02, 03  
**Estimated Complexity:** Medium  
**Priority:** P0 (Core feature)

---

## Objective

Implement the backend route and service that generates post-game performance reviews. After a match ends, this service compares the player's performance to their historical averages and provides insights on strengths and areas for improvement.

---

## Context

When a game ends, Hunter Mode fetches the match result and analyzes the local player's performance across key metrics:

- KDA (Kills/Deaths/Assists)
- CS per minute
- Vision score
- Damage dealt
- Gold earned

Each metric is compared to:

1. Player's historical average on that champion
2. Average for that rank/champion combination (if available)

The service identifies the player's best stat this game and their biggest improvement area, providing actionable feedback.

---

## What You Need to Build

### 1. Route (`backend/routes/postgame.js`)

```javascript
const express = require("express");
const router = express.Router();
const { buildPostGameReview } = require("../services/postGameAnalyzer");

/**
 * GET /post-game/latest/:puuid
 * Params:
 *   - puuid: Local player's PUUID
 * Query:
 *   - region: e.g. "na1"
 *
 * Returns: PostGameReview object
 */
router.get("/latest/:puuid", async (req, res) => {
  try {
    const { puuid } = req.params;
    const { region } = req.query;

    if (!region) {
      return res.status(400).json({
        error: "Missing required query parameter: region",
      });
    }

    const review = await buildPostGameReview(puuid, region);

    res.json({ review });
  } catch (error) {
    console.error("[Post-Game Route] Error:", error);
    res.status(500).json({
      error: "Failed to generate post-game review",
      message: error.message,
    });
  }
});

/**
 * GET /post-game/match/:matchId/:puuid
 * Params:
 *   - matchId: Specific match ID
 *   - puuid: Local player's PUUID
 * Query:
 *   - region: e.g. "na1"
 *
 * Returns: PostGameReview object for specific match
 */
router.get("/match/:matchId/:puuid", async (req, res) => {
  try {
    const { matchId, puuid } = req.params;
    const { region } = req.query;

    if (!region) {
      return res.status(400).json({
        error: "Missing required query parameter: region",
      });
    }

    const review = await buildPostGameReview(puuid, region, matchId);

    res.json({ review });
  } catch (error) {
    console.error("[Post-Game Route] Error:", error);
    res.status(500).json({
      error: "Failed to generate post-game review",
      message: error.message,
    });
  }
});

module.exports = router;
```

### 2. Post-Game Analyzer Service (`backend/services/postGameAnalyzer.js`)

```javascript
const cachedRiot = require("./cachedRiotApi");

/**
 * Build post-game performance review
 * @param {string} puuid - Player's PUUID
 * @param {string} region - Region code
 * @param {string|null} matchId - Specific match ID (or null for latest)
 */
async function buildPostGameReview(puuid, region, matchId = null) {
  // 1. Get the target match
  let targetMatch;
  if (matchId) {
    targetMatch = await cachedRiot.getMatch(matchId, region);
  } else {
    // Get latest match
    const matchIds = await cachedRiot.getMatchIdsByPuuid(puuid, region, 1);
    if (matchIds.length === 0) {
      throw new Error("No recent matches found");
    }
    targetMatch = await cachedRiot.getMatch(matchIds[0], region);
  }

  // 2. Find player's participant data
  const participant = targetMatch.info.participants.find(
    (p) => p.puuid === puuid,
  );

  if (!participant) {
    throw new Error("Player not found in match");
  }

  // 3. Get historical matches on this champion (for comparison)
  const allMatchIds = await cachedRiot.getMatchIdsByPuuid(puuid, region, 30);
  const historicalMatches = [];

  for (const id of allMatchIds) {
    if (id === targetMatch.metadata.matchId) continue; // Skip current match
    if (historicalMatches.length >= 20) break;

    const match = await cachedRiot.getMatch(id, region);
    const p = match.info.participants.find((p) => p.puuid === puuid);

    if (p && p.championId === participant.championId) {
      historicalMatches.push(p);
    }
  }

  // 4. Calculate historical averages
  const historicalAvg = calculateAverages(historicalMatches);

  // 5. Calculate current game stats
  const currentStats = extractStats(participant, targetMatch.info.gameDuration);

  // 6. Compare and generate insights
  const comparison = compareStats(currentStats, historicalAvg);

  // 7. Identify best stat and improvement area
  const bestStat = findBestStat(comparison);
  const improvementArea = findImprovementArea(comparison);

  // 8. Build review object
  return {
    matchId: targetMatch.metadata.matchId,
    matchResult: participant.win ? "Victory" : "Defeat",
    championId: participant.championId,
    championName: participant.championName,
    gameDurationMinutes: Math.floor(targetMatch.info.gameDuration / 60),
    currentStats,
    historicalAvg,
    comparison,
    bestStat,
    improvementArea,
  };
}

/**
 * Extract key stats from participant
 */
function extractStats(participant, gameDuration) {
  const durationMinutes = gameDuration / 60;

  return {
    kda:
      participant.deaths === 0
        ? participant.kills + participant.assists
        : (participant.kills + participant.assists) / participant.deaths,
    kills: participant.kills,
    deaths: participant.deaths,
    assists: participant.assists,
    csPerMin:
      (participant.totalMinionsKilled + participant.neutralMinionsKilled) /
      durationMinutes,
    visionScore: participant.visionScore,
    damageDealt: participant.totalDamageDealtToChampions,
    goldEarned: participant.goldEarned,
  };
}

/**
 * Calculate averages from historical matches
 */
function calculateAverages(matches) {
  if (matches.length === 0) {
    return {
      kda: 0,
      csPerMin: 0,
      visionScore: 0,
      damageDealt: 0,
      goldEarned: 0,
    };
  }

  const totals = matches.reduce(
    (acc, p) => {
      const duration = p.challenges?.gameLength || 1800; // Default 30 min
      const durationMinutes = duration / 60;

      return {
        kda:
          acc.kda +
          (p.deaths === 0
            ? p.kills + p.assists
            : (p.kills + p.assists) / p.deaths),
        csPerMin:
          acc.csPerMin +
          (p.totalMinionsKilled + p.neutralMinionsKilled) / durationMinutes,
        visionScore: acc.visionScore + p.visionScore,
        damageDealt: acc.damageDealt + p.totalDamageDealtToChampions,
        goldEarned: acc.goldEarned + p.goldEarned,
      };
    },
    { kda: 0, csPerMin: 0, visionScore: 0, damageDealt: 0, goldEarned: 0 },
  );

  return {
    kda: parseFloat((totals.kda / matches.length).toFixed(2)),
    csPerMin: parseFloat((totals.csPerMin / matches.length).toFixed(1)),
    visionScore: parseFloat((totals.visionScore / matches.length).toFixed(1)),
    damageDealt: Math.floor(totals.damageDealt / matches.length),
    goldEarned: Math.floor(totals.goldEarned / matches.length),
  };
}

/**
 * Compare current stats to historical average
 */
function compareStats(current, historical) {
  return {
    kda: {
      current: parseFloat(current.kda.toFixed(2)),
      historical: historical.kda,
      delta: parseFloat((current.kda - historical.kda).toFixed(2)),
      deltaPercent:
        historical.kda === 0
          ? 0
          : parseFloat(
              (((current.kda - historical.kda) / historical.kda) * 100).toFixed(
                1,
              ),
            ),
    },
    csPerMin: {
      current: parseFloat(current.csPerMin.toFixed(1)),
      historical: historical.csPerMin,
      delta: parseFloat((current.csPerMin - historical.csPerMin).toFixed(1)),
      deltaPercent:
        historical.csPerMin === 0
          ? 0
          : parseFloat(
              (
                ((current.csPerMin - historical.csPerMin) /
                  historical.csPerMin) *
                100
              ).toFixed(1),
            ),
    },
    visionScore: {
      current: current.visionScore,
      historical: historical.visionScore,
      delta: current.visionScore - historical.visionScore,
      deltaPercent:
        historical.visionScore === 0
          ? 0
          : parseFloat(
              (
                ((current.visionScore - historical.visionScore) /
                  historical.visionScore) *
                100
              ).toFixed(1),
            ),
    },
    damageDealt: {
      current: current.damageDealt,
      historical: historical.damageDealt,
      delta: current.damageDealt - historical.damageDealt,
      deltaPercent:
        historical.damageDealt === 0
          ? 0
          : parseFloat(
              (
                ((current.damageDealt - historical.damageDealt) /
                  historical.damageDealt) *
                100
              ).toFixed(1),
            ),
    },
  };
}

/**
 * Find best performing stat
 */
function findBestStat(comparison) {
  const stats = [
    { name: "KDA", delta: comparison.kda.deltaPercent },
    { name: "CS/min", delta: comparison.csPerMin.deltaPercent },
    { name: "Vision Score", delta: comparison.visionScore.deltaPercent },
    { name: "Damage Dealt", delta: comparison.damageDealt.deltaPercent },
  ];

  const best = stats.reduce((max, stat) =>
    stat.delta > max.delta ? stat : max,
  );

  return {
    statName: best.name,
    improvement: `+${best.delta.toFixed(1)}%`,
  };
}

/**
 * Find area needing improvement
 */
function findImprovementArea(comparison) {
  const stats = [
    { name: "KDA", delta: comparison.kda.deltaPercent },
    { name: "CS/min", delta: comparison.csPerMin.deltaPercent },
    { name: "Vision Score", delta: comparison.visionScore.deltaPercent },
    { name: "Damage Dealt", delta: comparison.damageDealt.deltaPercent },
  ];

  const worst = stats.reduce((min, stat) =>
    stat.delta < min.delta ? stat : min,
  );

  // Only flag as improvement area if significantly below average
  if (worst.delta < -10) {
    return {
      statName: worst.name,
      deficit: `${worst.delta.toFixed(1)}%`,
    };
  }

  return null; // No major improvement area
}

module.exports = {
  buildPostGameReview,
};
```

### 3. Register Route in Server (`backend/server.js`)

```javascript
const postGameRoutes = require("./routes/postgame");

// ... existing code ...

app.use("/post-game", postGameRoutes);
```

---

## Integration Points

### From Task 02/03 (Riot API + Caching):

- Uses cached match data
- Fetches historical matches for comparison

### For Task 13 (Post-Game Panel):

- Frontend calls `GET /post-game/latest/:puuid?region=...`
- Receives `{ review: PostGameReview }` response

---

## Data Model

### PostGameReview Object:

```typescript
interface PostGameReview {
  matchId: string;
  matchResult: "Victory" | "Defeat";
  championId: number;
  championName: string;
  gameDurationMinutes: number;

  currentStats: {
    kda: number;
    kills: number;
    deaths: number;
    assists: number;
    csPerMin: number;
    visionScore: number;
    damageDealt: number;
    goldEarned: number;
  };

  historicalAvg: {
    kda: number;
    csPerMin: number;
    visionScore: number;
    damageDealt: number;
    goldEarned: number;
  };

  comparison: {
    [key: string]: {
      current: number;
      historical: number;
      delta: number;
      deltaPercent: number;
    };
  };

  bestStat: {
    statName: string;
    improvement: string; // e.g. "+15.3%"
  };

  improvementArea: {
    statName: string;
    deficit: string; // e.g. "-12.5%"
  } | null;
}
```

---

## Testing Requirements

### Unit Tests (`backend/services/postGameAnalyzer.test.js`)

```javascript
const { buildPostGameReview } = require("./postGameAnalyzer");

const API_KEY = process.env.HUNTER_MODE_RIOT_API_KEY;
const describeIfApiKey = API_KEY ? describe : describe.skip;

describeIfApiKey("Post-Game Analyzer", () => {
  test("builds review for latest match", async () => {
    const review = await buildPostGameReview("test-puuid", "na1");

    expect(review).toHaveProperty("matchId");
    expect(review).toHaveProperty("matchResult");
    expect(review).toHaveProperty("currentStats");
    expect(review).toHaveProperty("historicalAvg");
    expect(review).toHaveProperty("comparison");
    expect(review).toHaveProperty("bestStat");
  });

  test("comparison shows deltas correctly", async () => {
    const review = await buildPostGameReview("test-puuid", "na1");

    expect(review.comparison.kda).toHaveProperty("current");
    expect(review.comparison.kda).toHaveProperty("historical");
    expect(review.comparison.kda).toHaveProperty("delta");
    expect(review.comparison.kda).toHaveProperty("deltaPercent");
  });

  test("identifies best stat", async () => {
    const review = await buildPostGameReview("test-puuid", "na1");

    expect(review.bestStat).toHaveProperty("statName");
    expect(review.bestStat).toHaveProperty("improvement");
  });
});
```

### Integration Tests (`backend/routes/postgame.test.js`)

```javascript
const request = require("supertest");
const app = require("../server");

describe("GET /post-game/latest/:puuid", () => {
  test("returns review for valid PUUID", async () => {
    const response = await request(app)
      .get("/post-game/latest/test-puuid")
      .query({ region: "na1" });

    expect(response.status).toBe(200);
    expect(response.body.review).toHaveProperty("matchId");
    expect(response.body.review).toHaveProperty("comparison");
  });

  test("returns 400 for missing region", async () => {
    const response = await request(app).get("/post-game/latest/test-puuid");

    expect(response.status).toBe(400);
  });
});
```

### Manual Testing

```bash
# Get latest match review for a player
curl "http://localhost:45679/post-game/latest/YOUR_PUUID?region=na1"

# Expected response time: <2 seconds
# Expected response: Full PostGameReview object
```

---

## Acceptance Criteria

✅ **Complete when:**

1. Route `/post-game/latest/:puuid` fetches latest match
2. Route `/post-game/match/:matchId/:puuid` fetches specific match
3. Service extracts stats from match data
4. Historical averages calculated from last 20 games
5. Comparison shows current vs historical with deltas
6. Best stat identified correctly
7. Improvement area identified (if exists)
8. All unit tests pass
9. All integration tests pass
10. Manual test returns valid review

---

## Performance Requirements

- **Response time**: <2 seconds (with cache), <4s (cold)
- **Match history depth**: Analyzes up to 20 historical games
- **Memory**: <5MB during analysis

---

## Files to Create/Modify

### New Files:

- `backend/routes/postgame.js`
- `backend/services/postGameAnalyzer.js`
- `backend/services/postGameAnalyzer.test.js`
- `backend/routes/postgame.test.js`

### Modified Files:

- `backend/server.js` (register route)

---

## Expected Time: 5-6 hours

## Difficulty: Medium (Stat aggregation and comparison logic)
