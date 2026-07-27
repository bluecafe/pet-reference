// 스프라이트 시트에서 "지금 그릴 프레임"의 좌표를 찾는다.
//
// 처음에는 규칙이 단순했다. 행 하나가 상태 하나고, 열이 프레임이다.
// 그러면 좌표는 곱셈으로 나온다.
//
//     x = frameIndex * cellWidth
//     y = row * cellHeight
//
// 이 방식이 언제 깨지는지가 이 예제의 요지다.
//
// 의존성이 없다. Node 22 이상이면 바로 돌아간다.

/** 셀 하나의 크기와 시트 전체 크기. */
export interface SheetGeometry {
  readonly sheetWidth: number;
  readonly sheetHeight: number;
  readonly cellWidth: number;
  readonly cellHeight: number;
}

/** 재생 정보. 어디에 있는지가 아니라 어떻게 재생할지를 담는다. */
export interface StatePlayback {
  readonly row: number;
  readonly frames: number;
  readonly fps: number;
  readonly loop: boolean;
}

/** 프레임 하나가 시트에서 차지하는 사각형. */
export interface FrameRect {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

export interface SpriteManifest {
  readonly geometry: SheetGeometry;
  readonly columns: number;
  /** 상태 이름 → 재생 정보. */
  readonly playback: Readonly<Record<string, StatePlayback>>;
  /** 상태 이름 → 실제 좌표. 계산으로 얻을 수 없는 배치를 담는다. */
  readonly layout: Readonly<Record<string, readonly FrameRect[]>>;
}

/**
 * 행·열 규칙으로 좌표를 계산한다.
 *
 * 한 행에 들어가는 프레임 수(`columns`)를 넘지 않을 때만 맞다.
 */
export function computeFrames(
  manifest: SpriteManifest,
  state: string,
): FrameRect[] | null {
  const playback = manifest.playback[state];
  if (!playback) return null;

  const { cellWidth, cellHeight } = manifest.geometry;
  return Array.from({ length: playback.frames }, (_, index) => ({
    x: (index % manifest.columns) * cellWidth,
    y: playback.row * cellHeight,
    w: cellWidth,
    h: cellHeight,
  }));
}

/**
 * 실제 좌표를 읽는다. 이쪽이 렌더링이 쓰는 값이다.
 *
 * 계산과 결과가 같은 상태가 대부분이지만, 다른 상태가 하나라도 있으면
 * 계산을 믿을 수 없다. 그래서 전부 명시 좌표를 쓴다.
 */
export function resolveFrames(
  manifest: SpriteManifest,
  state: string,
): readonly FrameRect[] | null {
  return manifest.layout[state] ?? null;
}

/** 계산과 실제가 어긋나는 상태를 찾는다. */
export function findLayoutMismatches(manifest: SpriteManifest): string[] {
  const mismatched: string[] = [];
  for (const state of Object.keys(manifest.playback)) {
    const computed = computeFrames(manifest, state);
    const actual = resolveFrames(manifest, state);
    if (!computed || !actual) continue;
    if (JSON.stringify(computed) !== JSON.stringify(actual)) {
      mismatched.push(state);
    }
  }
  return mismatched;
}

/** 두 상태가 같은 행을 가리키면 한쪽은 별칭이다. */
export function findAliases(manifest: SpriteManifest): [string, string][] {
  const byRow = new Map<number, string[]>();
  for (const [state, playback] of Object.entries(manifest.playback)) {
    const list = byRow.get(playback.row) ?? [];
    list.push(state);
    byRow.set(playback.row, list);
  }
  const aliases: [string, string][] = [];
  for (const states of byRow.values()) {
    for (let i = 1; i < states.length; i += 1) {
      aliases.push([states[0], states[i]]);
    }
  }
  return aliases;
}

export type ValidationError =
  | { readonly kind: "frame-count"; readonly state: string; readonly declared: number; readonly actual: number }
  | { readonly kind: "out-of-bounds"; readonly state: string; readonly frame: number }
  | { readonly kind: "missing-layout"; readonly state: string }
  | { readonly kind: "size-mismatch"; readonly state: string; readonly frame: number };

/**
 * 매니페스트가 시트와 맞는지 검사한다.
 *
 * 이 검사가 없으면 어긋난 좌표는 화면에서야 드러난다. 캐릭터가 반쯤 잘려
 * 보이거나 빈 칸이 그려지는데, 원인이 매니페스트라는 걸 알아채기 어렵다.
 */
export function validate(manifest: SpriteManifest): ValidationError[] {
  const errors: ValidationError[] = [];
  const { sheetWidth, sheetHeight, cellWidth, cellHeight } = manifest.geometry;

  for (const [state, playback] of Object.entries(manifest.playback)) {
    const frames = manifest.layout[state];
    if (!frames) {
      errors.push({ kind: "missing-layout", state });
      continue;
    }
    if (frames.length !== playback.frames) {
      errors.push({
        kind: "frame-count",
        state,
        declared: playback.frames,
        actual: frames.length,
      });
    }
    frames.forEach((rect, index) => {
      if (rect.w !== cellWidth || rect.h !== cellHeight) {
        errors.push({ kind: "size-mismatch", state, frame: index });
      }
      if (
        rect.x < 0 || rect.y < 0 ||
        rect.x + rect.w > sheetWidth ||
        rect.y + rect.h > sheetHeight
      ) {
        errors.push({ kind: "out-of-bounds", state, frame: index });
      }
    });
  }
  return errors;
}

/** 시트에서 아무도 쓰지 않는 행. 빈칸이 아니라 낭비다. */
export function findUnusedRows(manifest: SpriteManifest): number[] {
  const { sheetHeight, cellHeight } = manifest.geometry;
  const totalRows = Math.floor(sheetHeight / cellHeight);
  const used = new Set<number>();
  for (const frames of Object.values(manifest.layout)) {
    for (const rect of frames) {
      used.add(Math.floor(rect.y / cellHeight));
    }
  }
  return Array.from({ length: totalRows }, (_, i) => i).filter((row) => !used.has(row));
}
