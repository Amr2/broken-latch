# Task 05: Backend - Win Condition Feature

**App: Hunter Mode**  
**Dependencies:** Task 01, 02, 03  
**Estimated Complexity:** Medium-High  
**Priority:** P0 (Core feature)

---

## Objective

Implement the backend route and service that analyzes team compositions and generates win condition recommendations. This powers the **Win Condition Panel** by evaluating both teams' champions, determining power spike timings, identifying key threats, and providing strategic coaching tips.

---

## Context

After champion select completes, Hunter Mode analyzes the draft to answer:

- What is each team's optimal win condition?
- When is each team strongest? (Early/Mid/Late game power ratings)
- Which enemy champions are the highest priority threats?
- What strategic tips should the player follow?

This analysis uses:

1. **Data Dragon champion data** for champion tags (Tank, Fighter, Mage, Assassin, Marksman, Support)
2. **Champion role weights** (hardcoded heuristics for how each role contributes to different strategies)
3. **Composition scoring** (sum of weights across 5 champions)

The frontend calls `GET /win-condition?allyChampions=...&enemyChampions=...` and receives a comprehensive report.

---

## What You Need to Build

### 1. Route (`backend/routes/wincondition.js`)

```javascript
const express = require("express");
const router = express.Router();
const { analyzeComposition } = require("../services/winConditionAnalyzer");

/**
 * GET /win-condition
 * Query params:
 *   - allyChampions: comma-separated champion IDs (e.g. "64,201,157,238,67")
 *   - enemyChampions: comma-separated champion IDs (e.g. "11,35,22,81,12")
 *
 * Returns: WinConditionReport object
 */
router.get("/", async (req, res) => {
  try {
    const { allyChampions, enemyChampions } = req.query;

    // Validate inputs
    if (!allyChampions || !enemyChampions) {
      return res.status(400).json({
        error: "Missing required parameters: allyChampions, enemyChampions",
      });
    }

    const ally = allyChampions.split(",").map((id) => parseInt(id.trim()));
    const enemy = enemyChampions.split(",").map((id) => parseInt(id.trim()));

    if (ally.length !== 5 || enemy.length !== 5) {
      return res.status(400).json({
        error: "Must provide exactly 5 champions per team",
      });
    }

    // Analyze composition
    const report = await analyzeComposition(ally, enemy);

    res.json({ report });
  } catch (error) {
    console.error("[Win Condition Route] Error:", error);
    res.status(500).json({
      error: "Failed to analyze composition",
      message: error.message,
    });
  }
});

module.exports = router;
```

### 2. Win Condition Analyzer Service (`backend/services/winConditionAnalyzer.js`)

```javascript
const cachedRiot = require("./cachedRiotApi");

/**
 * Champion role weights for different strategies
 * Each role contributes different amounts to various win conditions
 */
const ROLE_WEIGHTS = {
  Tank: {
    earlyGame: 1,
    midGame: 2,
    lateGame: 3,
    teamfight: 3,
    poke: 0,
    siege: 1,
    splitpush: 1,
  },
  Fighter: {
    earlyGame: 3,
    midGame: 3,
    lateGame: 2,
    teamfight: 2,
    poke: 0,
    siege: 1,
    splitpush: 3,
  },
  Mage: {
    earlyGame: 1,
    midGame: 3,
    lateGame: 3,
    teamfight: 3,
    poke: 2,
    siege: 2,
    splitpush: 0,
  },
  Assassin: {
    earlyGame: 3,
    midGame: 3,
    lateGame: 2,
    teamfight: 1,
    poke: 0,
    siege: 0,
    splitpush: 2,
  },
  Marksman: {
    earlyGame: 1,
    midGame: 2,
    lateGame: 3,
    teamfight: 3,
    poke: 2,
    siege: 3,
    splitpush: 1,
  },
  Support: {
    earlyGame: 1,
    midGame: 2,
    lateGame: 2,
    teamfight: 2,
    poke: 1,
    siege: 1,
    splitpush: 0,
  },
};

/**
 * Analyze team composition
 */
async function analyzeComposition(allyChampionIds, enemyChampionIds) {
  // 1. Get champion data from Data Dragon
  const championData = await cachedRiot.getChampionData();
  const champions = championData.data;

  // 2. Map champion IDs to champion objects
  const getChampionById = (id) => {
    return Object.values(champions).find((c) => parseInt(c.key) === id);
  };

  const allyChampions = allyChampionIds.map(getChampionById).filter(Boolean);
  const enemyChampions = enemyChampionIds.map(getChampionById).filter(Boolean);

  // 3. Score both teams
  const allyScores = scoreTeam(allyChampions);
  const enemyScores = scoreTeam(enemyChampions);

  // 4. Derive win conditions
  const allyWinConditions = deriveWinConditions(allyScores, true);
  const enemyWinConditions = deriveWinConditions(enemyScores, false);

  // 5. Generate power timeline
  const allyPowerTimeline = {
    earlyGame: powerLevel(allyScores.earlyGame),
    midGame: powerLevel(allyScores.midGame),
    lateGame: powerLevel(allyScores.lateGame),
  };

  const enemyPowerTimeline = {
    earlyGame: powerLevel(enemyScores.earlyGame),
    midGame: powerLevel(enemyScores.midGame),
    lateGame: powerLevel(enemyScores.lateGame),
  };

  // 6. Identify key threats (top 2 enemy champions)
  const threats = identifyThreats(enemyChampions, enemyScores);

  // 7. Generate draft tips
  const draftTips = generateDraftTips(allyScores, enemyScores);

  // 8. Calculate favorable rating
  const allyTotal = Object.values(allyScores).reduce((a, b) => a + b, 0);
  const enemyTotal = Object.values(enemyScores).reduce((a, b) => a + b, 0);
  const diff = allyTotal - enemyTotal;

  const favorableRating =
    diff > 5 ? "Favorable" : diff < -5 ? "Unfavorable" : "Even";

  return {
    allyWinConditions,
    enemyWinConditions,
    allyPowerTimeline,
    enemyPowerTimeline,
    keyThreats: threats,
    draftTips,
    favorableRating,
  };
}

/**
 * Score a team based on champion roles
 */
function scoreTeam(champions) {
  const scores = {
    earlyGame: 0,
    midGame: 0,
    lateGame: 0,
    teamfight: 0,
    poke: 0,
    siege: 0,
    splitpush: 0,
  };

  champions.forEach((champion) => {
    if (!champion) return;

    // Get primary role tag
    const primaryTag = champion.tags[0];
    const weights = ROLE_WEIGHTS[primaryTag] || ROLE_WEIGHTS.Fighter;

    // Add weights to scores
    Object.keys(scores).forEach((key) => {
      scores[key] += weights[key];
    });
  });

  return scores;
}

/**
 * Convert numeric score to power level
 */
function powerLevel(score) {
  if (score >= 12) return "High";
  if (score >= 7) return "Medium";
  return "Low";
}

/**
 * Derive win conditions from team scores
 */
function deriveWinConditions(scores, isAlly) {
  const conditions = [];

  if (scores.teamfight >= 12) {
    conditions.push(
      "Force teamfights — your composition excels in 5v5 engagements",
    );
  }

  if (scores.poke >= 8) {
    conditions.push("Win through sustained poke before committing to fights");
  }

  if (scores.siege >= 10) {
    conditions.push(
      "Siege objectives — your ranged pressure makes towers vulnerable",
    );
  }

  if (scores.splitpush >= 10) {
    conditions.push(
      "Apply split pressure — force map responses with side-lane threats",
    );
  }

  if (scores.earlyGame >= 12) {
    conditions.push(
      "Snowball through early game — win before late-game scaling kicks in",
    );
  }

  if (scores.lateGame >= 12) {
    conditions.push(
      "Scale to late game — survive early and win extended fights",
    );
  }

  if (conditions.length === 0) {
    conditions.push("Well-rounded composition — adapt to game state");
  }

  return conditions.slice(0, 3); // Return top 3
}

/**
 * Identify top 2 threats from enemy team
 */
function identifyThreats(enemyChampions, enemyScores) {
  // Simple heuristic: prioritize champions with high carry potential
  const championScores = enemyChampions.map((champion) => {
    if (!champion) return { champion: null, score: 0 };

    const tag = champion.tags[0];
    let score = 0;

    // Assassins and Marksmen are high priority
    if (tag === "Assassin") score += 5;
    if (tag === "Marksman") score += 4;
    if (tag === "Mage") score += 3;
    if (tag === "Fighter") score += 2;

    return { champion, score };
  });

  // Sort by score and take top 2
  const threats = championScores
    .filter((t) => t.champion)
    .sort((a, b) => b.score - a.score)
    .slice(0, 2)
    .map((t) => ({
      championId: parseInt(t.champion.key),
      championName: t.champion.name,
    }));

  return threats;
}

/**
 * Generate draft tips based on composition matchup
 */
function generateDraftTips(allyScores, enemyScores) {
  const tips = [];

  if (enemyScores.teamfight > allyScores.teamfight + 5) {
    tips.push("Avoid grouped teamfights — split the map instead");
  }

  if (enemyScores.earlyGame > allyScores.earlyGame + 4) {
    tips.push("Play safe early — your team scales stronger");
  }

  if (allyScores.siege > enemyScores.siege + 4) {
    tips.push("Prioritize tower pressure over kills");
  }

  if (enemyScores.splitpush > allyScores.splitpush + 4) {
    tips.push("Watch side lanes — enemy has strong splitpush threat");
  }

  if (allyScores.poke > enemyScores.poke + 3) {
    tips.push("Abuse your poke range before engaging");
  }

  if (tips.length === 0) {
    tips.push("Play to your win conditions and adapt to enemy plays");
  }

  return tips.slice(0, 2); // Return top 2 tips
}

module.exports = {
  analyzeComposition,
};
```

### 3. Register Route in Server (`backend/server.js`)

```javascript
const winConditionRoutes = require("./routes/wincondition");

// ... existing code ...

app.use("/win-condition", winConditionRoutes);
```

---

## Integration Points

### From Task 02 (Riot API Client):

- Uses `cachedRiot.getChampionData()` for champion tags

### From Task 03 (Caching):

- Champion data is cached for 24 hours (static data)

### For Task 11 (Win Condition Panel):

- Frontend calls `GET /win-condition?allyChampions=...&enemyChampions=...`
- Receives `{ report: WinConditionReport }` response

---

## Data Model

### WinConditionReport Object:

```typescript
interface WinConditionReport {
  allyWinConditions: string[]; // Array of 1-3 win condition strings
  enemyWinConditions: string[]; // Array of 1-3 win condition strings
  allyPowerTimeline: {
    earlyGame: "Low" | "Medium" | "High";
    midGame: "Low" | "Medium" | "High";
    lateGame: "Low" | "Medium" | "High";
  };
  enemyPowerTimeline: {
    earlyGame: "Low" | "Medium" | "High";
    midGame: "Low" | "Medium" | "High";
    lateGame: "Low" | "Medium" | "High";
  };
  keyThreats: Array<{
    championId: number;
    championName: string;
  }>;
  draftTips: string[]; // Array of 1-2 strategic tips
  favorableRating: "Favorable" | "Even" | "Unfavorable";
}
```

---

## Testing Requirements

### Unit Tests (`backend/services/winConditionAnalyzer.test.js`)

```javascript
const { analyzeComposition } = require("./winConditionAnalyzer");

describe("Win Condition Analyzer", () => {
  test("analyzes teamfight composition correctly", async () => {
    // Team with 4 tanks + 1 mage (strong teamfight)
    const ally = [54, 111, 201, 516, 25]; // Malphite, Nautilus, Braum, Ornn, Morgana
    const enemy = [64, 238, 157, 222, 43]; // Lee, Zed, Yasuo, Jinx, Karma

    const report = await analyzeComposition(ally, enemy);

    expect(report.allyWinConditions).toContain(
      expect.stringMatching(/teamfight/i),
    );
    expect(report.allyPowerTimeline.lateGame).toBe("High");
  });

  test("identifies early game composition", async () => {
    // Assassin-heavy early game team
    const ally = [64, 238, 91, 23, 555]; // Lee, Zed, Talon, Tryndamere, Pyke
    const enemy = [54, 25, 115, 222, 40]; // Malphite, Morgana, Ziggs, Jinx, Janna

    const report = await analyzeComposition(ally, enemy);

    expect(report.allyPowerTimeline.earlyGame).toBe("High");
    expect(report.enemyPowerTimeline.lateGame).toBe("High");
  });

  test("identifies key threats correctly", async () => {
    const ally = [86, 11, 117, 222, 40]; // Garen, Yi, Lulu, Jinx, Janna
    const enemy = [238, 157, 64, 81, 412]; // Zed, Yasuo, Lee, Ezreal, Thresh

    const report = await analyzeComposition(ally, enemy);

    expect(report.keyThreats).toHaveLength(2);
    expect(report.keyThreats[0]).toHaveProperty("championId");
    expect(report.keyThreats[0]).toHaveProperty("championName");
  });

  test("generates appropriate draft tips", async () => {
    const ally = [86, 11, 117, 18, 16]; // Weak early, strong late
    const enemy = [64, 238, 91, 119, 555]; // Strong early, weak late

    const report = await analyzeComposition(ally, enemy);

    expect(report.draftTips).toContain(
      expect.stringMatching(/safe early|scales stronger/i),
    );
  });

  test("calculates favorable rating correctly", async () => {
    // Balanced team vs weak team
    const ally = [64, 238, 157, 222, 412]; // Strong meta picks
    const enemy = [11, 86, 33, 21, 16]; // Weak early picks

    const report = await analyzeComposition(ally, enemy);

    expect(report.favorableRating).toBe("Favorable");
  });
});
```

### Integration Tests (`backend/routes/wincondition.test.js`)

```javascript
const request = require("supertest");
const app = require("../server");

describe("GET /win-condition", () => {
  test("returns win condition report for valid input", async () => {
    const response = await request(app).get("/win-condition").query({
      allyChampions: "64,238,157,222,412",
      enemyChampions: "54,111,201,516,25",
    });

    expect(response.status).toBe(200);
    expect(response.body.report).toHaveProperty("allyWinConditions");
    expect(response.body.report).toHaveProperty("enemyWinConditions");
    expect(response.body.report).toHaveProperty("allyPowerTimeline");
    expect(response.body.report).toHaveProperty("keyThreats");
    expect(response.body.report).toHaveProperty("draftTips");
    expect(response.body.report).toHaveProperty("favorableRating");
  });

  test("returns 400 for invalid input", async () => {
    const response = await request(app).get("/win-condition").query({
      allyChampions: "64,238", // Only 2 champions
      enemyChampions: "54,111",
    });

    expect(response.status).toBe(400);
  });

  test("power timeline has correct values", async () => {
    const response = await request(app).get("/win-condition").query({
      allyChampions: "64,238,157,222,412",
      enemyChampions: "54,111,201,516,25",
    });

    const { allyPowerTimeline, enemyPowerTimeline } = response.body.report;

    expect(["Low", "Medium", "High"]).toContain(allyPowerTimeline.earlyGame);
    expect(["Low", "Medium", "High"]).toContain(allyPowerTimeline.midGame);
    expect(["Low", "Medium", "High"]).toContain(allyPowerTimeline.lateGame);
    expect(["Low", "Medium", "High"]).toContain(enemyPowerTimeline.earlyGame);
  });
});
```

### Manual Testing

```bash
# Test with real champion IDs
curl "http://localhost:45679/win-condition?allyChampions=64,238,157,222,412&enemyChampions=54,111,201,516,25"

# Expected response time: <500ms (with warm cache)
# Expected response: Full WinConditionReport object
```

---

## Acceptance Criteria

✅ **Complete when:**

1. Route `/win-condition` accepts and validates inputs
2. Service fetches champion data from Data Dragon
3. Team scoring correctly sums role weights
4. Win conditions are derived from scores
5. Power timeline shows Low/Medium/High for all phases
6. Key threats identifies top 2 enemy champions
7. Draft tips are contextual and helpful
8. Favorable rating is calculated correctly
9. All unit tests pass
10. All integration tests pass
11. Manual test returns valid report
12. Response time is <500ms with cache

---

## Performance Requirements

- **Response time**: <500ms (with warm cache), <2s (cold cache)
- **Memory**: <5MB for champion data
- **Cache hit rate**: 100% after first request (champion data is static)

---

## Files to Create/Modify

### New Files:

- `backend/routes/wincondition.js`
- `backend/services/winConditionAnalyzer.js`
- `backend/services/winConditionAnalyzer.test.js`
- `backend/routes/wincondition.test.js`

### Modified Files:

- `backend/server.js` (register route)

---

## Expected Time: 6-8 hours

## Difficulty: Medium-High (Complex composition analysis logic)
