// node --test hit-test.test.js

import { test } from "node:test";
import assert from "node:assert/strict";
import {
  Owner,
  acceptsMouse,
  isDrag,
  isInsideCanvas,
  resolveOwner,
  shouldIgnoreCursor,
  toCanvasPixel,
} from "./ui/hit-test.js";

const rect = { left: 0, top: 0, width: 640, height: 640 };

test("창 크기와 캔버스 해상도가 다르면 비율로 환산한다", () => {
  // 640px 창에 320px 캔버스. 화면 좌표의 절반이 캔버스 좌표다.
  assert.deepEqual(toCanvasPixel(320, 320, rect, 320, 320), { x: 160, y: 160 });
  assert.deepEqual(toCanvasPixel(0, 0, rect, 320, 320), { x: 0, y: 0 });

  // 환산을 빼먹으면 이 값이 320이 되어 캔버스 밖을 가리킨다.
  const wrong = { x: 320, y: 320 };
  assert.equal(isInsideCanvas(wrong, 320, 320), false);
});

test("캔버스 밖 좌표를 걸러낸다", () => {
  assert.equal(isInsideCanvas({ x: 0, y: 0 }, 320, 320), true);
  assert.equal(isInsideCanvas({ x: 319, y: 319 }, 320, 320), true);
  assert.equal(isInsideCanvas({ x: -1, y: 0 }, 320, 320), false);
  assert.equal(isInsideCanvas({ x: 320, y: 0 }, 320, 320), false);
});

test("마우스를 못 받는 요소를 걸러낸다", () => {
  assert.equal(acceptsMouse({ display: "block", visibility: "visible", pointerEvents: "auto" }), true);
  assert.equal(acceptsMouse({ display: "none", visibility: "visible", pointerEvents: "auto" }), false);
  assert.equal(acceptsMouse({ display: "block", visibility: "hidden", pointerEvents: "auto" }), false);
  assert.equal(acceptsMouse({ display: "block", visibility: "visible", pointerEvents: "none" }), false);
  assert.equal(acceptsMouse(null), false);
});

test("UI가 알파값보다 먼저다", () => {
  const button = { display: "block", visibility: "visible", pointerEvents: "auto" };
  // 버튼 자리는 캔버스 픽셀이 아니라 알파가 0이지만, 클릭은 버튼이 받아야 한다.
  assert.equal(resolveOwner([button], 0), Owner.UI);
});

test("장식 레이어가 화면을 덮어도 창 전체가 클릭을 먹지 않는다", () => {
  const decoration = { display: "block", visibility: "visible", pointerEvents: "none" };
  assert.equal(resolveOwner([decoration], 0), Owner.PASS_THROUGH);
});

test("알파값이 통과 여부를 가른다", () => {
  assert.equal(resolveOwner([], 255), Owner.CHARACTER);
  assert.equal(resolveOwner([], 1), Owner.CHARACTER);
  assert.equal(resolveOwner([], 0), Owner.PASS_THROUGH);
});

test("알파를 못 읽으면 통과시키지 않는다", () => {
  // getImageData는 캔버스 상태에 따라 실패한다. 실패를 통과로 처리하면
  // 캐릭터를 눌렀는데 뒤에 있는 창이 반응한다.
  assert.equal(resolveOwner([], null), Owner.CHARACTER);
});

test("판정 결과가 창 설정으로 이어진다", () => {
  assert.equal(shouldIgnoreCursor(Owner.PASS_THROUGH), true);
  assert.equal(shouldIgnoreCursor(Owner.CHARACTER), false);
  assert.equal(shouldIgnoreCursor(Owner.UI), false);
});

test("클릭과 드래그를 거리로 가른다", () => {
  assert.equal(isDrag(100, 100, 101, 101), false); // 손떨림
  assert.equal(isDrag(100, 100, 100, 100), false);
  assert.equal(isDrag(100, 100, 140, 100), true); // 옮기려는 동작
});
