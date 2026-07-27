// node --experimental-strip-types demo.ts
//
// 한 판을 두 번 재생해서 같은 대사가 나오는지 보여주고,
// 티어별 실제 발화율을 센다.

import { KO_BARKS, pickBark, shouldSpeak, type BarkKey, type Tactic } from "./barks.ts";

/** 엔진이 돌려줬다고 가정한 수순. 실제로는 engine.ts가 만든다. */
const GAME: readonly { tactic: Tactic; crisis: boolean }[] = [
  { tactic: "develop", crisis: false },
  { tactic: "develop", crisis: false },
  { tactic: "three", crisis: false },
  { tactic: "block_three", crisis: false },
  { tactic: "open_three", crisis: false },
  { tactic: "block_open_four", crisis: true },
  { tactic: "four", crisis: false },
  { tactic: "double_threat", crisis: false },
  { tactic: "win_five", crisis: false },
];

function replay(seed: number): string[] {
  const lines: string[] = [];
  let last: string | null = null;

  const greet = pickBark(KO_BARKS, "greet", seed, 0, last);
  if (greet) { lines.push(`[인사]   ${greet}`); last = greet; }

  GAME.forEach((move, index) => {
    const ply = index + 1;
    const key: BarkKey = move.crisis ? "crisis" : move.tactic;
    const line = pickBark(KO_BARKS, key, seed, ply, last);
    if (line) {
      lines.push(`[${String(ply).padStart(2)}수] ${key.padEnd(16)} ${line}`);
      last = line;
    } else {
      lines.push(`[${String(ply).padStart(2)}수] ${key.padEnd(16)} (조용)`);
    }
  });

  const win = pickBark(KO_BARKS, "win", seed, GAME.length + 1, last);
  if (win) lines.push(`[승리]   ${win}`);
  return lines;
}

const SEED = 20260727;

console.log(`시드 ${SEED} — 첫 번째 재생`);
console.log("-".repeat(56));
const first = replay(SEED);
first.forEach((line) => console.log("  " + line));

const second = replay(SEED);
console.log();
console.log(
  JSON.stringify(first) === JSON.stringify(second)
    ? "두 번째 재생: 완전히 같다. 시드와 수순만으로 재현된다."
    : "두 번째 재생: 달랐다 (이러면 안 된다)",
);

console.log();
console.log("티어별 실제 발화율 (2,000수 표본)");
console.log("-".repeat(56));
const measure = (key: BarkKey) => {
  let spoken = 0;
  for (let ply = 0; ply < 2000; ply += 1) {
    if (shouldSpeak(key, ply * 31 + 7, ply)) spoken += 1;
  }
  return (spoken / 2000 * 100).toFixed(1);
};
for (const key of ["win_five", "crisis", "four", "open_three", "develop", "think"] as BarkKey[]) {
  console.log(`  ${key.padEnd(14)} ${measure(key).padStart(5)}%`);
}

console.log();
console.log("승부가 갈리는 순간에는 반드시 말하고, 흔한 수에는 대체로 조용하다.");
