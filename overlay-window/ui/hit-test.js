// 커서 아래에 무엇이 있는지 판정한다. 이 파일에는 DOM도 Tauri도 없다.
//
// 창은 사각형인데 캐릭터는 사각형이 아니다. 그래서 "이 좌표에서 클릭을
// 받을 것인가"를 매번 정해야 하고, 그 판정이 이 모듈의 전부다.
//
// node --test hit-test.test.js 로 검증한다.

/** 클릭을 누가 받는지. */
export const Owner = {
  /** 캔버스 위에 얹힌 HTML 요소(말풍선, 버튼)가 받는다. */
  UI: "ui",
  /** 캔버스에 그려진 캐릭터가 받는다. */
  CHARACTER: "character",
  /** 아무것도 없다. 뒤에 있는 창으로 통과시킨다. */
  PASS_THROUGH: "pass-through",
};

/**
 * 화면 좌표를 캔버스 픽셀 좌표로 바꾼다.
 *
 * 창 크기와 캔버스 해상도가 다를 수 있어 비율로 환산해야 한다.
 * 이걸 빼먹으면 판정 위치가 캐릭터와 어긋난다 — 고해상도 화면에서
 * 캐릭터 오른쪽 아래를 눌러야 반응하는 증상이 이것이다.
 */
export function toCanvasPixel(clientX, clientY, rect, canvasWidth, canvasHeight) {
  return {
    x: Math.floor(((clientX - rect.left) / rect.width) * canvasWidth),
    y: Math.floor(((clientY - rect.top) / rect.height) * canvasHeight),
  };
}

/**
 * 그 좌표가 캔버스 안에 있는지. 밖이면 픽셀을 읽을 수 없다.
 */
export function isInsideCanvas(pixel, canvasWidth, canvasHeight) {
  return pixel.x >= 0 && pixel.y >= 0 && pixel.x < canvasWidth && pixel.y < canvasHeight;
}

/**
 * 마우스를 받을 수 있는 요소인지 본다.
 *
 * 걸러야 하는 것이 세 가지다. 특히 마지막이 중요한데, 장식용으로 얹어둔
 * 레이어가 화면 전체를 덮고 있으면 그것 때문에 창 전체가 클릭을 먹는다.
 */
export function acceptsMouse(style) {
  if (!style) return false;
  if (style.display === "none") return false;
  if (style.visibility === "hidden") return false;
  if (style.pointerEvents === "none") return false;
  return true;
}

/**
 * 클릭을 누가 받을지 정한다.
 *
 * UI가 먼저다. 말풍선이나 버튼은 캔버스 픽셀이 아니라서 알파값이 0으로
 * 나오는데, 알파값만 보면 버튼이 보이는데도 클릭이 뒤로 빠져나간다.
 *
 * @param elementStyles 그 지점에 겹쳐 있는 요소들의 스타일 (앞에 있는 것부터)
 * @param alpha         같은 지점의 캔버스 알파값 (0~255). 읽지 못했으면 null
 */
export function resolveOwner(elementStyles, alpha) {
  if (elementStyles.some(acceptsMouse)) {
    return Owner.UI;
  }
  // 알파를 못 읽었으면 통과시키지 않는다. 판정에 실패했다고 클릭을 흘려보내면
  // 캐릭터를 눌렀는데 뒤에 있는 창이 반응한다. 모를 때는 받아 두는 쪽이 낫다.
  if (alpha === null) {
    return Owner.CHARACTER;
  }
  return alpha > 0 ? Owner.CHARACTER : Owner.PASS_THROUGH;
}

/**
 * 판정 결과를 창 설정으로 옮긴다.
 * true면 이 창은 마우스 이벤트를 무시하고 뒤로 흘려보낸다.
 */
export function shouldIgnoreCursor(owner) {
  return owner === Owner.PASS_THROUGH;
}

/**
 * 눌렀다 뗄 때까지 움직인 거리로 클릭과 드래그를 가른다.
 *
 * 너무 작으면 손떨림 때문에 클릭이 드래그로 오인되고, 너무 크면 옮기려는
 * 동작이 클릭으로 먹혀 창이 안 움직인다. 데스크톱 UI에서 흔히 쓰는 범위가
 * 3~5픽셀이라 그 안에서 정했다.
 */
export const DRAG_THRESHOLD_PX = 4;

export function isDrag(startX, startY, endX, endY, threshold = DRAG_THRESHOLD_PX) {
  const dx = endX - startX;
  const dy = endY - startY;
  return Math.hypot(dx, dy) > threshold;
}
