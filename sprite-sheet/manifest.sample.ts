// 실제 앱 매니페스트를 줄인 것. 구조와 함정은 그대로 남겼다.
//
// 원본은 상태 36개에 시트가 2048×18432였다. 여기서는 다섯 개만 뒀지만
// walk가 시트 위에서 끊어져 있는 것과 talking이 wave의 별칭인 것은 실제 그대로다.

import type { SpriteManifest } from "./sheet.ts";

const CELL = 512;
const COLUMNS = 4;
/** 원본과 같은 비율로 줄였다. 마지막 행이 walk의 뒷부분이다. */
const ROWS = 6;

export const SAMPLE: SpriteManifest = {
  geometry: {
    sheetWidth: CELL * COLUMNS,
    sheetHeight: CELL * ROWS,
    cellWidth: CELL,
    cellHeight: CELL,
  },
  columns: COLUMNS,

  playback: {
    idle: { row: 0, frames: 4, fps: 3, loop: true },
    // 8프레임이다. 한 행에 4개만 들어가므로 행 하나로는 부족하다.
    walk: { row: 1, frames: 8, fps: 8, loop: true },
    jump: { row: 2, frames: 4, fps: 6, loop: false },
    wave: { row: 3, frames: 4, fps: 3, loop: true },
    // wave와 같은 행이다. 그림이 같고 부르는 이름만 다르다.
    talking: { row: 3, frames: 4, fps: 3, loop: true },
  },

  layout: {
    idle: row(0),
    // 앞 4프레임은 행 1, 뒤 4프레임은 맨 끝 행이다.
    // 시트를 다시 만들지 않고 남는 자리에 이어 붙인 결과다.
    walk: [...row(1), ...row(ROWS - 1)],
    jump: row(2),
    wave: row(3),
    talking: row(3),
  },
};

function row(index: number) {
  return Array.from({ length: COLUMNS }, (_, column) => ({
    x: column * CELL,
    y: index * CELL,
    w: CELL,
    h: CELL,
  }));
}
