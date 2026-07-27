// node --experimental-strip-types --test sheet.test.ts

import { test } from "node:test";
import assert from "node:assert/strict";
import { SAMPLE } from "./manifest.sample.ts";
import {
  computeFrames,
  findAliases,
  findLayoutMismatches,
  findUnusedRows,
  resolveFrames,
  validate,
  type SpriteManifest,
} from "./sheet.ts";

test("한 행에 들어가는 상태는 계산과 실제가 같다", () => {
  for (const state of ["idle", "jump", "wave"]) {
    assert.deepEqual(computeFrames(SAMPLE, state), resolveFrames(SAMPLE, state));
  }
});

test("행을 넘는 상태에서 계산이 깨진다", () => {
  // walk는 8프레임인데 한 행에 4개만 들어간다.
  const computed = computeFrames(SAMPLE, "walk")!;
  const actual = resolveFrames(SAMPLE, "walk")!;

  // 계산은 5번째 프레임을 첫 열로 되돌려 버린다 — 1번째와 같은 자리다.
  assert.deepEqual(computed[4], computed[0]);

  // 실제로는 시트의 다른 행에 있다.
  assert.notDeepEqual(actual[4], actual[0]);
  assert.equal(actual[0].y, 512);
  assert.equal(actual[4].y, 512 * 5);
});

test("어긋나는 상태를 찾아낸다", () => {
  assert.deepEqual(findLayoutMismatches(SAMPLE), ["walk"]);
});

test("같은 행을 가리키는 상태는 별칭이다", () => {
  assert.deepEqual(findAliases(SAMPLE), [["wave", "talking"]]);
  // 별칭은 좌표도 같다. 그림을 두 번 넣지 않는다.
  assert.deepEqual(resolveFrames(SAMPLE, "wave"), resolveFrames(SAMPLE, "talking"));
});

test("정상 매니페스트에는 오류가 없다", () => {
  assert.deepEqual(validate(SAMPLE), []);
});

test("선언한 프레임 수와 좌표 개수가 다르면 잡는다", () => {
  const broken: SpriteManifest = {
    ...SAMPLE,
    playback: { ...SAMPLE.playback, idle: { row: 0, frames: 6, fps: 3, loop: true } },
  };
  const errors = validate(broken);
  assert.equal(errors.length, 1);
  assert.deepEqual(errors[0], { kind: "frame-count", state: "idle", declared: 6, actual: 4 });
});

test("시트 밖을 가리키는 좌표를 잡는다", () => {
  const broken: SpriteManifest = {
    ...SAMPLE,
    layout: {
      ...SAMPLE.layout,
      jump: [{ x: 0, y: 512 * 99, w: 512, h: 512 }, ...SAMPLE.layout.jump.slice(1)],
    },
  };
  const errors = validate(broken);
  assert.ok(errors.some((e) => e.kind === "out-of-bounds" && e.state === "jump"));
});

test("셀 크기가 다른 프레임을 잡는다", () => {
  const broken: SpriteManifest = {
    ...SAMPLE,
    layout: {
      ...SAMPLE.layout,
      jump: [{ x: 0, y: 1024, w: 256, h: 512 }, ...SAMPLE.layout.jump.slice(1)],
    },
  };
  const errors = validate(broken);
  assert.ok(errors.some((e) => e.kind === "size-mismatch"));
});

test("좌표가 없는 상태를 잡는다", () => {
  const { walk: _removed, ...withoutWalk } = SAMPLE.layout;
  const broken: SpriteManifest = { ...SAMPLE, layout: withoutWalk };
  assert.ok(validate(broken).some((e) => e.kind === "missing-layout"));
});

test("아무도 안 쓰는 행을 찾는다", () => {
  // 표본은 여섯 행을 다 쓴다.
  assert.deepEqual(findUnusedRows(SAMPLE), [4]);
});
