// node --experimental-strip-types demo.ts
//
// 계산 방식과 실제 좌표가 어디서 갈라지는지 보여준다.

import { SAMPLE } from "./manifest.sample.ts";
import {
  computeFrames,
  findAliases,
  findLayoutMismatches,
  findUnusedRows,
  resolveFrames,
  validate,
} from "./sheet.ts";

const g = SAMPLE.geometry;
console.log(`시트 ${g.sheetWidth}×${g.sheetHeight}, 셀 ${g.cellWidth}×${g.cellHeight}, ${SAMPLE.columns}열`);
console.log(`상태 ${Object.keys(SAMPLE.playback).length}개\n`);

console.log("계산 vs 실제");
console.log("-".repeat(58));
for (const state of Object.keys(SAMPLE.playback)) {
  const computed = computeFrames(SAMPLE, state)!;
  const actual = resolveFrames(SAMPLE, state)!;
  const same = JSON.stringify(computed) === JSON.stringify(actual);
  console.log(`  ${state.padEnd(10)} ${String(actual.length).padStart(2)}프레임  ${same ? "같음" : "다름  ← 계산으로 못 찾는다"}`);
}

console.log("\nwalk 프레임이 실제로 있는 자리");
console.log("-".repeat(58));
resolveFrames(SAMPLE, "walk")!.forEach((rect, index) => {
  const row = rect.y / g.cellHeight;
  const marker = index === 4 ? "  ← 여기서 행이 건너뛴다" : "";
  console.log(`  ${String(index).padStart(2)}번  행 ${row}  (x=${rect.x}, y=${rect.y})${marker}`);
});

console.log();
console.log("어긋나는 상태:", findLayoutMismatches(SAMPLE).join(", ") || "없음");
console.log("별칭:", findAliases(SAMPLE).map(([a, b]) => `${b} → ${a}`).join(", ") || "없음");
console.log("안 쓰는 행:", findUnusedRows(SAMPLE).join(", ") || "없음");
console.log("검증 오류:", validate(SAMPLE).length, "건");

console.log();
console.log("한 상태라도 계산과 어긋나면 계산을 믿을 수 없다.");
console.log("그래서 전부 명시 좌표를 쓴다.");
