// 대국 중 캐릭터가 무슨 말을 할지 정하는 최소 예제.
//
// 두 가지가 이 파일의 요지다.
//
// 1. 생성 모델이 꺼져 있어도 대국 전체가 완결된다 — 여기 있는 대사만으로 충분하다
// 2. 같은 판을 다시 두면 같은 말을 한다 — Math.random()을 쓰지 않는다
//
// 의존성이 없다. Node 22 이상이면 바로 돌아간다.

/** 엔진이 좌표만이 아니라 "왜 거기에 뒀는지"를 돌려준다. 반응의 폭이 여기서 나온다. */
export type Tactic =
  | "win_five"
  | "block_five"
  | "open_four"
  | "double_threat"
  | "block_open_four"
  | "four"
  | "block_four"
  | "open_three"
  | "block_three"
  | "three"
  | "develop";

/** 발화 지점 전부: 대국 경계 이벤트 + 전술별 코멘트. */
export type BarkKey =
  | "greet"
  | "think"
  | "crisis"
  | "win"
  | "lose"
  | "draw"
  | "rematch"
  | Tactic;

export type BarkTable = Readonly<Record<BarkKey, readonly string[]>>;

// --- 발화 확률 차등 -----------------------------------------------------------
// 매 수마다 말하면 성가시고, 안 하면 심심하다. 그래서 등급을 뒀다.

/** 승부가 갈리는 순간과 대국 경계는 반드시 말한다. */
const ALWAYS_SPEAK: ReadonlySet<BarkKey> = new Set<BarkKey>([
  "greet", "win", "lose", "draw", "rematch", "crisis",
  "win_five", "block_five", "open_four", "double_threat", "block_open_four",
]);

/** 중간 티어 — 절반쯤 말한다. */
const MID_TIER: ReadonlySet<BarkKey> = new Set<BarkKey>([
  "four", "block_four", "open_three", "block_three", "three",
]);

const MID_TIER_PROBABILITY = 0.5;
/** 가장 흔하고 급하지 않은 수. 대체로 조용하다. */
const LOW_TIER_PROBABILITY = 0.2;

/**
 * mulberry32. 시드 하나로 결정되는 난수 생성기다.
 *
 * Math.random()을 쓰지 않는 이유는 재현 때문이다. "저기서 이상한 말을 했다"는
 * 제보를 받았을 때, 시드와 수순만으로 그대로 다시 볼 수 있어야 고칠 수 있다.
 */
function createRng(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) | 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** 문자열 키를 32비트 정수로 접는다. 같은 (시드, 수순)이라도 발화 지점이 다르면 다른 대사가 나오게. */
function hashKey(key: string): number {
  let hash = 0x811c9dc5;
  for (let i = 0; i < key.length; i += 1) {
    hash ^= key.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return hash >>> 0;
}

/**
 * 이번 발화 지점에서 말할지 정한다.
 *
 * 같은 (시드, 수순, 키)면 언제나 같은 결과다.
 */
export function shouldSpeak(key: BarkKey, seed: number, ply: number): boolean {
  if (ALWAYS_SPEAK.has(key)) return true;
  const probability = MID_TIER.has(key) ? MID_TIER_PROBABILITY : LOW_TIER_PROBABILITY;
  const rng = createRng((seed ^ hashKey(`speak:${key}`) ^ Math.imul(ply + 1, 0x9e3779b1)) >>> 0);
  return rng() < probability;
}

/**
 * 대사 하나를 고른다. 말하지 않기로 했거나 대사가 없으면 null.
 *
 * lastLine을 주면 바로 직전과 같은 대사를 피한다. 같은 말을 연달아 하면
 * 미리 써 둔 목록이라는 게 그대로 드러난다.
 */
export function pickBark(
  table: BarkTable,
  key: BarkKey,
  seed: number,
  ply: number,
  lastLine: string | null = null,
): string | null {
  const lines = table[key];
  if (!lines || lines.length === 0) return null;
  if (!shouldSpeak(key, seed, ply)) return null;

  const rng = createRng((seed ^ hashKey(key) ^ Math.imul(ply + 1, 0x9e3779b1)) >>> 0);
  const candidates = lines.length > 1 && lastLine !== null
    ? lines.filter((line) => line !== lastLine)
    : lines;
  const pool = candidates.length > 0 ? candidates : lines;
  return pool[Math.floor(rng() * pool.length) % pool.length];
}

/** 한국어 기본 대사판. 이것만으로 대국 시작부터 끝까지 완결된다. */
export const KO_BARKS: BarkTable = {
  greet: ["한 판 둘까?", "오늘은 내가 이길 것 같은데", "자, 시작하자"],
  think: ["음...", "어디 두지", "잠깐만"],
  crisis: ["어, 이거 위험한데", "막아야겠다", "그렇게 두면 곤란하지"],
  win: ["내가 이겼다!", "오늘은 내 날이네", "잘 뒀어, 그래도 내가 이겼지만"],
  lose: ["졌다...", "이번엔 네가 잘했어", "한 판 더 하자"],
  draw: ["비겼네", "판이 꽉 찼다", "무승부구나"],
  rematch: ["한 판 더?", "다시 할까", "이번엔 안 진다"],
  win_five: ["다섯 개!", "여기서 끝이다"],
  block_five: ["그건 못 두게 하지", "아슬아슬했다"],
  open_four: ["이제 막을 수 없어", "양쪽 다 열렸다"],
  double_threat: ["두 군데 동시에", "어느 쪽을 막을래?"],
  block_open_four: ["위험한 거 막았다", "여기 안 막으면 끝이었어"],
  four: ["넷", "하나 남았다"],
  block_four: ["막을게", "거기는 안 되지"],
  open_three: ["셋 열렸다", "슬슬 조여볼까"],
  block_three: ["미리 막아둔다", "커지기 전에"],
  three: ["셋", "조금씩"],
  develop: ["일단 여기", "천천히 가자", "이쯤에"],
};
