# Task 09: Frontend - Shared Components

**App: Hunter Mode**  
**Dependencies:** Task 01, 08  
**Estimated Complexity:** Medium  
**Priority:** P0 (Frontend foundation)

---

## Objective

Build all reusable React components used across multiple panels. These shared components provide consistent UI patterns, reduce code duplication, and ensure visual consistency throughout Hunter Mode.

---

## Context

Hunter Mode uses a cohesive design language across all panels. Rather than duplicating UI code, we create shared components for:

- Champion icons (from Data Dragon CDN)
- Rank badges (Bronze through Challenger)
- Win rate pills (color-coded percentage displays)
- KDA displays (Kills/Deaths/Assists formatting)
- Stat bars (horizontal progress bars with labels)
- Threat indicators (1-5 dot displays)
- Loading spinners

All components follow the TailwindCSS theme from Task 01 and accept proper TypeScript props.

---

## What You Need to Build

### 1. Champion Icon Component (`frontend/src/shared/ChampionIcon.tsx`)

```tsx
import { DATA_DRAGON_BASE, DATA_DRAGON_VERSION } from "../lib/constants";

interface ChampionIconProps {
  championId: number;
  size?: number; // Size in pixels
  className?: string;
}

export function ChampionIcon({
  championId,
  size = 32,
  className = "",
}: ChampionIconProps) {
  // Map champion ID to champion key (would ideally come from Data Dragon)
  // For now, hardcode common IDs or fetch from backend
  const url = `${DATA_DRAGON_BASE}/${DATA_DRAGON_VERSION}/img/champion/${championId}.png`;

  return (
    <img
      src={url}
      alt={`Champion ${championId}`}
      className={`rounded ${className}`}
      style={{ width: size, height: size }}
      onError={(e) => {
        // Fallback to placeholder if image fails to load
        e.currentTarget.src =
          "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'/%3E";
      }}
    />
  );
}
```

### 2. Rank Badge Component (`frontend/src/shared/RankBadge.tsx`)

```tsx
interface RankBadgeProps {
  rank: string; // e.g. "Gold II", "Platinum IV", "Unranked"
  size?: "sm" | "md" | "lg";
  className?: string;
}

export function RankBadge({
  rank,
  size = "md",
  className = "",
}: RankBadgeProps) {
  const [tier] = rank.split(" ");

  const colors: Record<string, string> = {
    Challenger: "from-yellow-400 to-yellow-600",
    Grandmaster: "from-red-400 to-red-600",
    Master: "from-purple-400 to-purple-600",
    Diamond: "from-blue-400 to-blue-600",
    Emerald: "from-emerald-400 to-emerald-600",
    Platinum: "from-cyan-400 to-cyan-600",
    Gold: "from-yellow-300 to-yellow-500",
    Silver: "from-gray-300 to-gray-500",
    Bronze: "from-orange-400 to-orange-600",
    Iron: "from-gray-600 to-gray-800",
    Unranked: "from-gray-500 to-gray-700",
  };

  const sizeClasses = {
    sm: "text-xs px-2 py-0.5",
    md: "text-sm px-3 py-1",
    lg: "text-base px-4 py-1.5",
  };

  const gradient = colors[tier] || colors.Unranked;

  return (
    <span
      className={`inline-block rounded-full bg-gradient-to-r ${gradient} font-semibold text-white ${sizeClasses[size]} ${className}`}
    >
      {rank}
    </span>
  );
}
```

### 3. Win Rate Pill Component (`frontend/src/shared/WinRatePill.tsx`)

```tsx
interface WinRatePillProps {
  winRate: number; // 0.0 to 1.0
  gamesPlayed?: number;
  size?: "sm" | "md";
  className?: string;
}

export function WinRatePill({
  winRate,
  gamesPlayed,
  size = "md",
  className = "",
}: WinRatePillProps) {
  const percentage = Math.round(winRate * 100);

  // Color based on win rate
  const color =
    percentage >= 55
      ? "bg-hm-win"
      : percentage >= 50
        ? "bg-hm-even"
        : "bg-hm-loss";

  const sizeClasses =
    size === "sm" ? "text-xs px-2 py-0.5" : "text-sm px-3 py-1";

  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full ${color} font-semibold text-white ${sizeClasses} ${className}`}
    >
      {percentage}%
      {gamesPlayed !== undefined && (
        <span className="text-xs opacity-70">({gamesPlayed}G)</span>
      )}
    </span>
  );
}
```

### 4. KDA Display Component (`frontend/src/shared/KdaDisplay.tsx`)

```tsx
interface KdaDisplayProps {
  kda: number;
  kills?: number;
  deaths?: number;
  assists?: number;
  size?: "sm" | "md";
  showBreakdown?: boolean;
  className?: string;
}

export function KdaDisplay({
  kda,
  kills,
  deaths,
  assists,
  size = "md",
  showBreakdown = false,
  className = "",
}: KdaDisplayProps) {
  const kdaColor =
    kda >= 4
      ? "text-hm-threat5"
      : kda >= 2.5
        ? "text-hm-threat3"
        : "text-hm-secondary";

  const sizeClasses = size === "sm" ? "text-xs" : "text-sm";

  if (
    showBreakdown &&
    kills !== undefined &&
    deaths !== undefined &&
    assists !== undefined
  ) {
    return (
      <div className={`flex items-center gap-1 ${sizeClasses} ${className}`}>
        <span className="text-hm-primary">{kills}</span>
        <span className="text-hm-muted">/</span>
        <span className="text-hm-loss">{deaths}</span>
        <span className="text-hm-muted">/</span>
        <span className="text-hm-primary">{assists}</span>
        <span className={`ml-2 font-semibold ${kdaColor}`}>
          ({kda.toFixed(2)} KDA)
        </span>
      </div>
    );
  }

  return (
    <span className={`font-semibold ${kdaColor} ${sizeClasses} ${className}`}>
      {kda.toFixed(2)} KDA
    </span>
  );
}
```

### 5. Stat Bar Component (`frontend/src/shared/StatBar.tsx`)

```tsx
interface StatBarProps {
  label: string;
  value: number;
  maxValue?: number;
  percentage?: number; // 0-100, overrides value/maxValue
  color?: string;
  showValue?: boolean;
  className?: string;
}

export function StatBar({
  label,
  value,
  maxValue = 100,
  percentage,
  color = "bg-hm-favorable",
  showValue = true,
  className = "",
}: StatBarProps) {
  const fillPercent = percentage ?? (value / maxValue) * 100;

  return (
    <div className={`flex flex-col gap-1 ${className}`}>
      <div className="flex items-center justify-between text-xs text-hm-secondary">
        <span>{label}</span>
        {showValue && <span>{value}</span>}
      </div>
      <div className="h-1.5 w-full rounded-full bg-hm-border overflow-hidden">
        <div
          className={`h-full ${color} transition-all duration-300`}
          style={{ width: `${Math.min(100, fillPercent)}%` }}
        />
      </div>
    </div>
  );
}
```

### 6. Threat Indicator Component (`frontend/src/shared/ThreatIndicator.tsx`)

```tsx
interface ThreatIndicatorProps {
  level: number; // 1-5
  size?: "sm" | "md" | "lg";
  showLabel?: boolean;
  className?: string;
}

export function ThreatIndicator({
  level,
  size = "md",
  showLabel = false,
  className = "",
}: ThreatIndicatorProps) {
  const dotSizes = {
    sm: "w-1.5 h-1.5",
    md: "w-2 h-2",
    lg: "w-2.5 h-2.5",
  };

  const colors = [
    "bg-hm-threat1",
    "bg-hm-threat2",
    "bg-hm-threat3",
    "bg-hm-threat4",
    "bg-hm-threat5",
  ];

  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {[1, 2, 3, 4, 5].map((i) => (
        <div
          key={i}
          className={`rounded-full ${dotSizes[size]} ${
            i <= level ? colors[level - 1] : "bg-hm-border"
          }`}
        />
      ))}
      {showLabel && (
        <span className="ml-1 text-xs text-hm-secondary">{level}/5</span>
      )}
    </div>
  );
}
```

### 7. Loading Spinner Component (`frontend/src/shared/LoadingSpinner.tsx`)

```tsx
interface LoadingSpinnerProps {
  size?: "sm" | "md" | "lg";
  className?: string;
}

export function LoadingSpinner({
  size = "md",
  className = "",
}: LoadingSpinnerProps) {
  const sizes = {
    sm: "w-4 h-4",
    md: "w-8 h-8",
    lg: "w-12 h-12",
  };

  return (
    <div className={`flex items-center justify-center ${className}`}>
      <div
        className={`animate-spin rounded-full border-2 border-hm-border border-t-hm-favorable ${sizes[size]}`}
      />
    </div>
  );
}
```

### 8. Item Icon Component (`frontend/src/shared/ItemIcon.tsx`)

```tsx
import { DATA_DRAGON_BASE, DATA_DRAGON_VERSION } from "../lib/constants";

interface ItemIconProps {
  itemId: number;
  size?: number;
  className?: string;
}

export function ItemIcon({ itemId, size = 24, className = "" }: ItemIconProps) {
  const url = `${DATA_DRAGON_BASE}/${DATA_DRAGON_VERSION}/img/item/${itemId}.png`;

  return (
    <img
      src={url}
      alt={`Item ${itemId}`}
      className={`rounded ${className}`}
      style={{ width: size, height: size }}
      onError={(e) => {
        e.currentTarget.src =
          "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'/%3E";
      }}
    />
  );
}
```

### 9. Index Export (`frontend/src/shared/index.ts`)

```typescript
export { ChampionIcon } from "./ChampionIcon";
export { RankBadge } from "./RankBadge";
export { WinRatePill } from "./WinRatePill";
export { KdaDisplay } from "./KdaDisplay";
export { StatBar } from "./StatBar";
export { ThreatIndicator } from "./ThreatIndicator";
export { LoadingSpinner } from "./LoadingSpinner";
export { ItemIcon } from "./ItemIcon";
```

---

## Integration Points

### From Task 01 (Project Setup):

- Uses TailwindCSS theme colors (hm-\*)
- Uses Data Dragon constants

### For All Panel Tasks (10-13):

- All panels import from `shared/` folder
- Consistent UI across all features

---

## Testing Requirements

### Visual Testing

Create a test page (`frontend/src/shared/ComponentShowcase.tsx`):

```tsx
export function ComponentShowcase() {
  return (
    <div className="p-8 bg-hm-bg min-h-screen text-hm-primary space-y-8">
      <section>
        <h2 className="text-lg mb-4">Champion Icons</h2>
        <div className="flex gap-2">
          <ChampionIcon championId={64} size={32} />
          <ChampionIcon championId={238} size={48} />
          <ChampionIcon championId={222} size={64} />
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">Rank Badges</h2>
        <div className="flex gap-2 flex-wrap">
          <RankBadge rank="Iron IV" />
          <RankBadge rank="Bronze II" />
          <RankBadge rank="Silver I" />
          <RankBadge rank="Gold III" />
          <RankBadge rank="Platinum II" />
          <RankBadge rank="Diamond I" />
          <RankBadge rank="Master" />
          <RankBadge rank="Challenger" />
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">Win Rate Pills</h2>
        <div className="flex gap-2">
          <WinRatePill winRate={0.35} gamesPlayed={20} />
          <WinRatePill winRate={0.5} gamesPlayed={50} />
          <WinRatePill winRate={0.65} gamesPlayed={30} />
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">KDA Displays</h2>
        <div className="space-y-2">
          <KdaDisplay
            kda={1.5}
            kills={3}
            deaths={5}
            assists={4}
            showBreakdown
          />
          <KdaDisplay
            kda={3.2}
            kills={8}
            deaths={2}
            assists={10}
            showBreakdown
          />
          <KdaDisplay
            kda={5.7}
            kills={15}
            deaths={1}
            assists={12}
            showBreakdown
          />
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">Stat Bars</h2>
        <div className="space-y-2 w-64">
          <StatBar label="CS/min" value={7.2} maxValue={10} />
          <StatBar label="Vision Score" value={45} maxValue={80} />
          <StatBar label="Win Rate" percentage={65} />
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">Threat Indicators</h2>
        <div className="space-y-2">
          {[1, 2, 3, 4, 5].map((level) => (
            <ThreatIndicator key={level} level={level} showLabel />
          ))}
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">Loading Spinner</h2>
        <div className="flex gap-4">
          <LoadingSpinner size="sm" />
          <LoadingSpinner size="md" />
          <LoadingSpinner size="lg" />
        </div>
      </section>

      <section>
        <h2 className="text-lg mb-4">Item Icons</h2>
        <div className="flex gap-2">
          <ItemIcon itemId={3078} size={24} />
          <ItemIcon itemId={3031} size={32} />
          <ItemIcon itemId={3087} size={40} />
        </div>
      </section>
    </div>
  );
}
```

---

## Acceptance Criteria

✅ **Complete when:**

1. All 8 shared components are implemented
2. All components have proper TypeScript props
3. All components use TailwindCSS classes
4. Champion/Item icons load from Data Dragon
5. Rank badges show correct tier colors
6. Win rate pills color-code correctly (green/gray/red)
7. KDA displays format correctly
8. Stat bars animate fill percentage
9. Threat indicators show 1-5 dots
10. Loading spinner animates
11. ComponentShowcase renders all variants
12. No TypeScript errors
13. No console warnings

---

## Performance Requirements

- **Render time**: <16ms per component (60fps)
- **Image load time**: <500ms (from CDN)
- **Animation smoothness**: 60fps for transitions

---

## Files to Create

### New Files:

- `frontend/src/shared/ChampionIcon.tsx`
- `frontend/src/shared/RankBadge.tsx`
- `frontend/src/shared/WinRatePill.tsx`
- `frontend/src/shared/KdaDisplay.tsx`
- `frontend/src/shared/StatBar.tsx`
- `frontend/src/shared/ThreatIndicator.tsx`
- `frontend/src/shared/LoadingSpinner.tsx`
- `frontend/src/shared/ItemIcon.tsx`
- `frontend/src/shared/index.ts`
- `frontend/src/shared/ComponentShowcase.tsx` (for testing)

---

## Expected Time: 6-8 hours

## Difficulty: Medium (Straightforward React components with styling)
