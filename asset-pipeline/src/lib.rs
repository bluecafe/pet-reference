//! 생성된 프레임을 시트에 넣기 전에 거치는 단계들.
//!
//! 캐릭터 프레임을 이미지 생성으로 만들면 배경이 투명하게 안 나온다. 단색으로
//! 깔고 나중에 빼야 하는데, 그 과정에서 프레임마다 결과가 달라진다. 눈으로
//! 수백 장을 확인할 수는 없으니 숫자로 본다.
//!
//! 의존성이 없다. PNG 디코딩은 범위 밖이라 픽셀 배열을 직접 받는다.

/// RGBA 픽셀 하나.
pub type Rgba = [u8; 4];

/// 한 프레임. 픽셀은 행 우선(row-major)으로 들어온다.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Rgba>,
}

impl Frame {
    pub fn new(width: usize, height: usize, pixels: Vec<Rgba>) -> Self {
        assert_eq!(pixels.len(), width * height, "픽셀 수가 크기와 맞지 않는다");
        Self { width, height, pixels }
    }

    pub fn at(&self, x: usize, y: usize) -> Rgba {
        self.pixels[y * self.width + x]
    }
}

/// 배경으로 깐 색. 마젠타를 쓰는 이유는 캐릭터에 거의 안 나오는 색이라서다.
/// 살구색이나 회색을 쓰면 피부와 옷이 같이 지워진다.
pub const CHROMA_MAGENTA: Rgba = [255, 0, 255, 255];

/// 색이 얼마나 비슷한지. 생성 이미지는 배경도 미세하게 번지므로 딱 맞지 않는다.
fn distance(a: Rgba, b: Rgba) -> u32 {
    let dr = a[0].abs_diff(b[0]) as u32;
    let dg = a[1].abs_diff(b[1]) as u32;
    let db = a[2].abs_diff(b[2]) as u32;
    dr * dr + dg * dg + db * db
}

/// 배경색에 가까운 픽셀을 투명하게 만든다.
///
/// `tolerance`가 0이면 정확히 같은 색만 지운다. 값을 키우면 번진 가장자리까지
/// 지워지지만, 너무 키우면 캐릭터의 붉은 계열까지 같이 지워진다.
pub fn remove_chroma(frame: &Frame, key: Rgba, tolerance: u32) -> Frame {
    let threshold = tolerance * tolerance;
    let pixels = frame
        .pixels
        .iter()
        .map(|&pixel| {
            if distance(pixel, key) <= threshold {
                [0, 0, 0, 0]
            } else {
                pixel
            }
        })
        .collect();
    Frame::new(frame.width, frame.height, pixels)
}

/// 투명하지 않은 픽셀 수. 프레임이 제대로 나왔는지 보는 가장 단순한 지표다.
pub fn opaque_pixels(frame: &Frame) -> usize {
    frame.pixels.iter().filter(|pixel| pixel[3] > 0).count()
}

/// 프레임 검사 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameIssue {
    /// 아무것도 안 남았다. 배경만 있었거나 크로마키가 전부 지웠다.
    Empty { index: usize },
    /// 캐릭터가 셀 가장자리에 닿는다. 시트에 넣으면 옆 프레임과 붙어 보인다.
    TouchesMargin { index: usize },
    /// 다른 프레임과 크기가 다르다. 시트는 같은 크기의 셀을 전제한다.
    SizeMismatch { index: usize },
    /// 불투명 픽셀 수가 다른 프레임과 크게 다르다. 배경이 덜 빠졌을 수 있다.
    OutlierDensity { index: usize, pixels: usize, median: usize },
}

/// 셀 가장자리에 두는 여백. 캐릭터가 여기 닿으면 잘려 보인다.
pub const DEFAULT_SAFE_MARGIN: usize = 24;

/// 불투명 픽셀 수가 중앙값에서 이만큼(%) 벗어나면 이상하다고 본다.
pub const DEFAULT_DENSITY_TOLERANCE_PERCENT: usize = 40;

/// 프레임들을 한꺼번에 검사한다.
///
/// 프레임 하나만 보면 정상인지 알 수 없다. 불투명 픽셀이 3만 개라는 게
/// 많은 건지 적은 건지는 **다른 프레임과 비교해야** 안다. 그래서 묶어서 본다.
pub fn inspect(frames: &[Frame], safe_margin: usize, tolerance_percent: usize) -> Vec<FrameIssue> {
    let mut issues = Vec::new();
    if frames.is_empty() {
        return issues;
    }

    let (width, height) = (frames[0].width, frames[0].height);
    let mut densities: Vec<usize> = Vec::with_capacity(frames.len());

    for (index, frame) in frames.iter().enumerate() {
        if frame.width != width || frame.height != height {
            issues.push(FrameIssue::SizeMismatch { index });
        }
        let count = opaque_pixels(frame);
        densities.push(count);

        if count == 0 {
            issues.push(FrameIssue::Empty { index });
            continue;
        }
        if touches_margin(frame, safe_margin) {
            issues.push(FrameIssue::TouchesMargin { index });
        }
    }

    let median = median_of(&densities);
    if median > 0 {
        for (index, &count) in densities.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let diff = count.abs_diff(median) * 100 / median;
            if diff > tolerance_percent {
                issues.push(FrameIssue::OutlierDensity { index, pixels: count, median });
            }
        }
    }

    issues
}

/// 여백 안쪽에 불투명 픽셀이 있는지 본다.
fn touches_margin(frame: &Frame, margin: usize) -> bool {
    if margin == 0 || frame.width <= margin * 2 || frame.height <= margin * 2 {
        return false;
    }
    for y in 0..frame.height {
        let in_vertical_margin = y < margin || y >= frame.height - margin;
        for x in 0..frame.width {
            let in_horizontal_margin = x < margin || x >= frame.width - margin;
            if (in_vertical_margin || in_horizontal_margin) && frame.at(x, y)[3] > 0 {
                return true;
            }
        }
    }
    false
}

fn median_of(values: &[usize]) -> usize {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

/// 프레임 내용의 지문. 같은 내용이면 같은 값이 나온다.
///
/// 실제 파이프라인은 SHA-256을 쓴다. 여기서는 의존성을 안 넣으려고 FNV-1a를
/// 썼다. **충돌 저항이 필요한 곳에는 쓰면 안 된다** — 이 값의 목적은
/// "빌드 사이에 내용이 바뀌었는가"를 보는 것뿐이다.
pub fn content_digest(frame: &Frame) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x1000_0000_01b3;

    let mut hash = OFFSET;
    let mut feed = |byte: u8| {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    };

    for value in [frame.width, frame.height] {
        for byte in value.to_le_bytes() {
            feed(byte);
        }
    }
    for pixel in &frame.pixels {
        for &byte in pixel {
            feed(byte);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: usize, height: usize, color: Rgba) -> Frame {
        Frame::new(width, height, vec![color; width * height])
    }

    /// 가운데에 사각형 하나를 그린 프레임. 나머지는 배경색이다.
    fn with_blob(size: usize, blob: usize, color: Rgba) -> Frame {
        let mut frame = solid(size, size, CHROMA_MAGENTA);
        let start = (size - blob) / 2;
        for y in start..start + blob {
            for x in start..start + blob {
                frame.pixels[y * size + x] = color;
            }
        }
        frame
    }

    #[test]
    fn chroma_removal_clears_only_the_background() {
        let frame = with_blob(64, 20, [10, 200, 30, 255]);
        let cleaned = remove_chroma(&frame, CHROMA_MAGENTA, 0);
        assert_eq!(opaque_pixels(&cleaned), 20 * 20);
    }

    #[test]
    fn tolerance_catches_bleeding_edges() {
        // 생성 이미지는 배경이 미세하게 번진다. 정확히 같은 색만 지우면 남는다.
        let mut frame = solid(8, 8, CHROMA_MAGENTA);
        frame.pixels[0] = [250, 5, 250, 255];

        assert_eq!(opaque_pixels(&remove_chroma(&frame, CHROMA_MAGENTA, 0)), 1);
        assert_eq!(opaque_pixels(&remove_chroma(&frame, CHROMA_MAGENTA, 16)), 0);
    }

    #[test]
    fn too_much_tolerance_eats_the_character() {
        // 붉은 계열은 마젠타와 가깝다. 관용값을 키우면 같이 지워진다.
        let frame = with_blob(32, 10, [220, 30, 200, 255]);
        assert!(opaque_pixels(&remove_chroma(&frame, CHROMA_MAGENTA, 20)) > 0);
        assert_eq!(opaque_pixels(&remove_chroma(&frame, CHROMA_MAGENTA, 90)), 0);
    }

    #[test]
    fn empty_frames_are_reported() {
        let blank = remove_chroma(&solid(32, 32, CHROMA_MAGENTA), CHROMA_MAGENTA, 0);
        let good = remove_chroma(&with_blob(32, 10, [0, 0, 255, 255]), CHROMA_MAGENTA, 0);

        let issues = inspect(&[good, blank], DEFAULT_SAFE_MARGIN, 100);
        assert!(issues.contains(&FrameIssue::Empty { index: 1 }));
    }

    #[test]
    fn characters_touching_the_edge_are_reported() {
        // 셀을 꽉 채우면 시트에서 옆 프레임과 붙어 보인다.
        let full = solid(64, 64, [0, 0, 255, 255]);
        let issues = inspect(&[full], 8, 100);
        assert!(issues.contains(&FrameIssue::TouchesMargin { index: 0 }));
    }

    #[test]
    fn density_outliers_are_reported() {
        let base = |blob| remove_chroma(&with_blob(64, blob, [0, 0, 255, 255]), CHROMA_MAGENTA, 0);
        // 셋은 비슷하고 하나만 배경이 덜 빠져 훨씬 크다.
        let frames = vec![base(20), base(21), base(20), base(40)];

        let issues = inspect(&frames, DEFAULT_SAFE_MARGIN, DEFAULT_DENSITY_TOLERANCE_PERCENT);
        assert!(issues
            .iter()
            .any(|issue| matches!(issue, FrameIssue::OutlierDensity { index: 3, .. })));
    }

    #[test]
    fn similar_frames_are_not_flagged() {
        // 실제 리포트의 값이다. 걷는 동작 네 장의 픽셀 수는 이 정도로 붙어 있다.
        let counts = [24643usize, 24587, 24626, 24682];
        let median = median_of(&counts);
        for count in counts {
            let diff = count.abs_diff(median) * 100 / median;
            assert!(diff <= DEFAULT_DENSITY_TOLERANCE_PERCENT, "{count}이 튄다");
        }
    }

    #[test]
    fn size_mismatch_is_reported() {
        let a = solid(32, 32, [0, 0, 255, 255]);
        let b = solid(64, 32, [0, 0, 255, 255]);
        let issues = inspect(&[a, b], 0, 100);
        assert!(issues.contains(&FrameIssue::SizeMismatch { index: 1 }));
    }

    #[test]
    fn digest_changes_when_content_changes() {
        let a = with_blob(16, 4, [1, 2, 3, 255]);
        let b = with_blob(16, 4, [1, 2, 3, 255]);
        let c = with_blob(16, 4, [1, 2, 4, 255]);

        assert_eq!(content_digest(&a), content_digest(&b));
        assert_ne!(content_digest(&a), content_digest(&c));
    }

    #[test]
    fn digest_separates_frames_with_the_same_pixels_but_different_shape() {
        // 픽셀 수가 같아도 크기가 다르면 다른 프레임이다.
        let wide = Frame::new(4, 2, vec![[9, 9, 9, 255]; 8]);
        let tall = Frame::new(2, 4, vec![[9, 9, 9, 255]; 8]);
        assert_ne!(content_digest(&wide), content_digest(&tall));
    }
}
