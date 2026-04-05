# Task 06: Backend - Hunter Focus Feature

**App: Hunter Mode**  
**Dependencies:** Task 01, 02, 03, 04  
**Estimated Complexity:** High  
**Priority:** P0 (Signature feature)

---

## Objective

Implement the backend route and service for the **Hunter Focus** feature — Hunter Mode's signature deep-dive analysis of a single enemy player. When a player locks onto a target (Alt+1-5), this service provides comprehensive insights: ability tendencies, positional patterns, recent performance, and counter tips.

---

## Context

Hunter Focus is triggered when the player selects one enemy to focus on during the game. The frontend calls `GET /hunter/:summonerName/:championId` and receives an extended profile with:

1. All basic profile data from Task 04 (win rate, KDA, mastery, etc.)
2. **Ability tendencies**: Skill max order, combo patterns
3. **Positional tendencies**: Lane behavior, roaming frequency, objective priority
4. **Counter tips**: How to beat this champion
5. **Recommended build**: What items to build against them

This is similar to Task 04 but goes deeper with champion-specific analysis.

---

## What You Need to Build

### 1. Route (`backend/routes/hunter.js`)

```javascript
const express = require("express");
const router = express.Router();
const { buildHunterProfile } = require("../services/hunterProfile");

/**
 * GET /hunter/:summonerName/:championId
 * Params:
 *   - summonerName: Enemy summoner name
 *   - championId: Champion they're playing
 * Query:
 *   - region: e.g. "na1"
 *
 * Returns: HunterProfile object (extended PlayerProfile)
 */
router.get("/:summonerName/:championId", async (req, res) => {
  try {
    const { summonerName, championId } = req.params;
    const { region } = req.query;

    if (!region) {
      return res.status(400).json({
        error: "Missing required query parameter: region",
      });
    }

    const profile = await buildHunterProfile(
      summonerName,
      parseInt(championId),
      region,
    );

    res.json({ profile });
  } catch (error) {
    console.error("[Hunter Route] Error:", error);
    res.status(500).json({
      error: "Failed to build hunter profile",
      message: error.message,
    });
  }
});

module.exports = router;
```

### 2. Hunter Profile Service (`backend/services/hunterProfile.js`)

```javascript
const cachedRiot = require("./cachedRiotApi");
const { buildPlayerProfile } = require("./playerProfile");

/**
 * Champion counter tips database
 * In a real app, this would come from a database or API
 */
const COUNTER_TIPS = {
  // Assassins
  238: [
    "Zed: Buy Zhonya's/Stopwatch to counter Death Mark",
    "Ward sides to spot his roams early",
    "He's weakest when his W (shadow) is on cooldown",
  ],
  91: [
    "Talon: Ward over walls to track his E mobility",
    "Build armor early to survive his burst",
    "He struggles vs grouped teams — stay together",
  ],
  // Marksmen
  222: [
    "Jinx: Shut her down early before she scales",
    "Dive her in teamfights — she has no escape",
    "Deny vision to prevent her long-range poke",
  ],
  // Add more champions as needed...
};

/**
 * Build comprehensive Hunter Focus profile
 */
async function buildHunterProfile(summonerName, championId, region) {
  // 1. Get base profile (reuses Task 04 logic)
  const baseProfile = await buildPlayerProfile(
    summonerName,
    championId,
    region,
    20,
  ); // Fetch 20 games for deeper analysis

  // 2. Get summoner and match data
  const summoner = await cachedRiot.getSummonerByName(summonerName, region);
  const matchIds = await cachedRiot.getMatchIdsByPuuid(
    summoner.puuid,
    region,
    30,
  );

  // 3. Get full match data for this champion
  const matches = await Promise.all(
    matchIds.map((id) => cachedRiot.getMatch(id, region)),
  );

  const championMatches = matches
    .filter((match) => {
      const participant = match.info.participants.find(
        (p) => p.puuid === summoner.puuid,
      );
      return participant && participant.championId === championId;
    })
    .slice(0, 20);

  // 4. Analyze ability tendencies
  const abilityTendencies = analyzeAbilities(championMatches, summoner.puuid);

  // 5. Analyze positional tendencies
  const positionalTendencies = analyzePositioning(
    championMatches,
    summoner.puuid,
  );

  // 6. Get counter tips
  const counterTips = COUNTER_TIPS[championId] || [
    "Focus this champion in teamfights",
    "Build defensive items if they're ahead",
    "Track their positioning and punish mistakes",
  ];

  // 7. Recommend counter build
  const recommendedCounterItems = getCounterBuild(championId);

  // 8. Combine into HunterProfile
  return {
    ...baseProfile,
    abilityTendencies,
    positionalTendencies,
    counterTips,
    recommendedCounterItems,
  };
}

/**
 * Analyze ability usage patterns
 */
function analyzeAbilities(matches, puuid) {
  if (matches.length === 0) {
    return {
      skillMaxOrder: [],
      preferredCombo: "Unknown",
      ultimateUsagePattern: "Unknown",
    };
  }

  // Extract skill levels at 15 minutes (proxy for skill max order)
  // Note: Riot API doesn't provide skill order directly, so we infer from common patterns
  // In a production app, you'd use timeline data or external databases

  // Placeholder logic (would need timeline API for real data)
  const skillMaxOrder = ["Q", "W", "E"]; // Most common order

  // Combo patterns (hardcoded by champion — would ideally come from gameplay data)
  const preferredCombo = "Q > E > W"; // Example

  // Ultimate usage (early vs late in fights)
  const ultimateUsagePattern = "Early engage"; // Example

  return {
    skillMaxOrder,
    preferredCombo,
    ultimateUsagePattern,
  };
}

/**
 * Analyze positional and macro tendencies
 */
function analyzePositioning(matches, puuid) {
  if (matches.length === 0) {
    return {
      roamingFrequency: "Unknown",
      objectivePriority: "Unknown",
      wardingPattern: "Unknown",
    };
  }

  // Calculate roaming frequency from kills/assists in other lanes
  // (simplified — would need timeline data for accuracy)
  let totalKillParticipation = 0;
  let totalVisionScore = 0;

  matches.forEach((match) => {
    const participant = match.info.participants.find((p) => p.puuid === puuid);
    if (participant) {
      totalKillParticipation += participant.kills + participant.assists;
      totalVisionScore += participant.visionScore;
    }
  });

  const avgKillParticipation = totalKillParticipation / matches.length;
  const avgVisionScore = totalVisionScore / matches.length;

  // Heuristics
  const roamingFrequency =
    avgKillParticipation > 15
      ? "High"
      : avgKillParticipation > 8
        ? "Medium"
        : "Low";

  const wardingPattern =
    avgVisionScore > 40
      ? "Vision-focused"
      : avgVisionScore > 20
        ? "Standard"
        : "Low vision";

  // Objective priority (would need timeline data — placeholder)
  const objectivePriority = "Balanced"; // Dragon/Baron/Towers

  return {
    roamingFrequency,
    objectivePriority,
    wardingPattern,
  };
}

/**
 * Get counter build recommendations
 * (Hardcoded by champion type — in production would use item effectiveness data)
 */
function getCounterBuild(championId) {
  const championData = {
    // Assassins (AD)
    238: [3157, 2420], // Zhonya's, Seeker's Armguard
    91: [3191, 3742], // Seeker's, Dead Man's Plate

    // Marksmen
    222: [3143, 3082], // Randuin's, Thornmail
    81: [3047, 3143], // Plated Steelcaps, Randuin's

    // Mages (AP)
    99: [3065, 3111], // Spirit Visage, Mercury Treads
    134: [3156, 3814], // Maw, Edge of Night
  };

  return championData[championId] || [3047, 3065]; // Default: Steelcaps + Spirit Visage
}

module.exports = {
  buildHunterProfile,
};
```

### 3. Register Route in Server (`backend/server.js`)

```javascript
const hunterRoutes = require("./routes/hunter");

// ... existing code ...

app.use("/hunter", hunterRoutes);
```

---

## Integration Points

### From Task 04 (Enemy Profiles):

- Reuses `buildPlayerProfile()` for base data
- Extends it with Hunter-specific insights

### From Task 02/03 (Riot API + Caching):

- Uses cached API calls for match data

### For Task 12 (Hunter Focus Panel):

- Frontend calls `GET /hunter/:summonerName/:championId?region=...`
- Receives `{ profile: HunterProfile }` response

---

## Data Model

### HunterProfile Object (extends PlayerProfile):

```typescript
interface HunterProfile extends PlayerProfile {
  // Ability analysis
  abilityTendencies: {
    skillMaxOrder: string[]; // e.g. ["Q", "W", "E"]
    preferredCombo: string; // e.g. "Q > E > W"
    ultimateUsagePattern: string; // e.g. "Early engage" | "Cleanup"
  };

  // Positioning analysis
  positionalTendencies: {
    roamingFrequency: "Low" | "Medium" | "High";
    objectivePriority: string; // e.g. "Dragon-focused" | "Balanced"
    wardingPattern: string; // e.g. "Vision-focused" | "Low vision"
  };

  // Counter strategy
  counterTips: string[]; // Array of 2-3 coaching tips
  recommendedCounterItems: number[]; // Item IDs to build against this champion
}
```

---

## Testing Requirements

### Unit Tests (`backend/services/hunterProfile.test.js`)

```javascript
const { buildHunterProfile } = require("./hunterProfile");

const API_KEY = process.env.HUNTER_MODE_RIOT_API_KEY;
const describeIfApiKey = API_KEY ? describe : describe.skip;

describeIfApiKey("Hunter Profile Service", () => {
  test("builds complete hunter profile", async () => {
    const profile = await buildHunterProfile("Doublelift", 222, "na1");

    expect(profile).toHaveProperty("summonerName");
    expect(profile).toHaveProperty("winRate");
    expect(profile).toHaveProperty("abilityTendencies");
    expect(profile).toHaveProperty("positionalTendencies");
    expect(profile).toHaveProperty("counterTips");
    expect(profile).toHaveProperty("recommendedCounterItems");
  });

  test("ability tendencies have correct structure", async () => {
    const profile = await buildHunterProfile("Doublelift", 222, "na1");

    expect(profile.abilityTendencies).toHaveProperty("skillMaxOrder");
    expect(profile.abilityTendencies).toHaveProperty("preferredCombo");
    expect(profile.abilityTendencies).toHaveProperty("ultimateUsagePattern");
  });

  test("positional tendencies have correct values", async () => {
    const profile = await buildHunterProfile("Doublelift", 222, "na1");

    expect(["Low", "Medium", "High"]).toContain(
      profile.positionalTendencies.roamingFrequency,
    );
  });

  test("counter tips are provided", async () => {
    const profile = await buildHunterProfile("Doublelift", 222, "na1");

    expect(Array.isArray(profile.counterTips)).toBe(true);
    expect(profile.counterTips.length).toBeGreaterThan(0);
  });

  test("recommended items are valid", async () => {
    const profile = await buildHunterProfile("Doublelift", 222, "na1");

    expect(Array.isArray(profile.recommendedCounterItems)).toBe(true);
    expect(profile.recommendedCounterItems.length).toBeGreaterThan(0);
  });
});
```

### Integration Tests (`backend/routes/hunter.test.js`)

```javascript
const request = require("supertest");
const app = require("../server");

describe("GET /hunter/:summonerName/:championId", () => {
  test("returns hunter profile for valid input", async () => {
    const response = await request(app)
      .get("/hunter/Doublelift/222")
      .query({ region: "na1" });

    expect(response.status).toBe(200);
    expect(response.body.profile).toHaveProperty("abilityTendencies");
    expect(response.body.profile).toHaveProperty("positionalTendencies");
    expect(response.body.profile).toHaveProperty("counterTips");
  });

  test("returns 400 for missing region", async () => {
    const response = await request(app).get("/hunter/Doublelift/222");

    expect(response.status).toBe(400);
  });
});
```

### Manual Testing

```bash
curl "http://localhost:45679/hunter/Doublelift/222?region=na1"

# Expected response time: <2 seconds
# Expected response: Full HunterProfile with all fields
```

---

## Acceptance Criteria

✅ **Complete when:**

1. Route `/hunter/:summonerName/:championId` accepts parameters
2. Service extends base profile with hunter-specific data
3. Ability tendencies are analyzed from match data
4. Positional tendencies show roaming/vision/objective patterns
5. Counter tips are provided for common champions
6. Recommended counter items are included
7. All unit tests pass
8. All integration tests pass
9. Manual test returns valid extended profile
10. Response time is <2 seconds

---

## Performance Requirements

- **Response time**: <2 seconds (with cache), <4s (cold cache)
- **Match depth**: Analyzes up to 20 games on champion
- **Memory**: <10MB during analysis

---

## Future Enhancements

- Use Riot Timeline API for accurate ability order data
- Dynamic counter tips from community database
- Machine learning for positional tendency prediction
- Champion-specific analysis modules

---

## Files to Create/Modify

### New Files:

- `backend/routes/hunter.js`
- `backend/services/hunterProfile.js`
- `backend/services/hunterProfile.test.js`
- `backend/routes/hunter.test.js`

### Modified Files:

- `backend/server.js` (register route)

---

## Expected Time: 8-10 hours

## Difficulty: High (Complex multi-source data analysis)
