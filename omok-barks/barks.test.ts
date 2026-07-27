// node --experimental-strip-types --test barks.test.ts

import { test } from "node:test";
import assert from "node:assert/strict";
import { KO_BARKS, pickBark, shouldSpeak, type BarkKey } from "./barks.ts";

test("같은 (시드, 수순, 키)면 언제나 같은 대사가 나온다", () => {
  for (let ply = 0; ply < 20; ply += 1) {
    const first = pickBark(KO_BARKS, "develop", 12345, ply);
    const second = pickBark(KO_BARKS, "develop", 12345, ply);
    assert.equal(first, second, `ply ${ply}에서 결과가 갈렸다`);
  }
});

test("시드가 다르면 결과가 갈린다", () => {
  const a: (string | null)[] = [];
  const b: (string | null)[] = [];
  for (let ply = 0; ply < 30; ply += 1) {
    a.push(pickBark(KO_BARKS, "develop", 1, ply));
    b.push(pickBark(KO_BARKS, "develop", 999, ply));
  }
  assert.notDeepEqual(a, b, "시드를 바꿔도 같은 기보가 나왔다");
});

test("승부가 갈리는 순간에는 반드시 말한다", () => {
  const always: BarkKey[] = [
    "greet", "win", "lose", "draw", "rematch", "crisis",
    "win_five", "block_five", "open_four", "double_threat", "block_open_four",
  ];
  for (const key of always) {
    for (let ply = 0; ply < 50; ply += 1) {
      assert.equal(shouldSpeak(key, ply * 7 + 1, ply), true, `${key}가 침묵했다`);
    }
  }
});

test("흔한 수는 대체로 조용하고, 중간 티어는 그 사이다", () => {
  const rate = (key: BarkKey) => {
    let spoken = 0;
    const trials = 2000;
    for (let ply = 0; ply < trials; ply += 1) {
      if (shouldSpeak(key, ply * 31 + 7, ply)) spoken += 1;
    }
    return spoken / trials;
  };

  const develop = rate("develop");
  const four = rate("four");

  assert.ok(develop > 0.1 && develop < 0.3, `develop 발화율이 이상하다: ${develop}`);
  assert.ok(four > 0.4 && four < 0.6, `four 발화율이 이상하다: ${four}`);
  assert.ok(develop < four, "흔한 수가 더 자주 말했다");
});

test("직전과 같은 대사를 피한다", () => {
  // greet는 항상 말하므로 null이 나오지 않는다.
  const first = pickBark(KO_BARKS, "greet", 42, 0);
  assert.ok(first !== null);
  const second = pickBark(KO_BARKS, "greet", 42, 0, first);
  assert.notEqual(second, first, "직전과 같은 대사가 다시 나왔다");
});

test("모든 발화 지점에 대사가 준비돼 있다", () => {
  // LLM이 꺼져 있어도 대국이 완결된다는 뜻이다.
  const keys = Object.keys(KO_BARKS) as BarkKey[];
  for (const key of keys) {
    assert.ok(KO_BARKS[key].length > 0, `${key}에 대사가 없다`);
  }
  assert.equal(keys.length, 18, "발화 지점 수가 바뀌었다");
});
