//! 프레임 묶음을 검사해서 무엇이 걸리는지 보여준다.
//!
//! ```text
//! cargo run
//! ```

use asset_pipeline::{
    content_digest, inspect, opaque_pixels, remove_chroma, Frame, FrameIssue, Rgba,
    CHROMA_MAGENTA, DEFAULT_DENSITY_TOLERANCE_PERCENT, DEFAULT_SAFE_MARGIN,
};

const SIZE: usize = 128;

fn background() -> Frame {
    Frame::new(SIZE, SIZE, vec![CHROMA_MAGENTA; SIZE * SIZE])
}

/// 가운데에 사각형을 그린 프레임.
fn blob(side: usize, color: Rgba) -> Frame {
    let mut frame = background();
    let start = (SIZE - side) / 2;
    for y in start..start + side {
        for x in start..start + side {
            frame.pixels[y * SIZE + x] = color;
        }
    }
    frame
}

fn main() {
    let character = [0, 120, 255, 255];

    let raw = vec![
        ("정상", blob(60, character)),
        ("정상", blob(62, character)),
        ("배경만", background()),
        ("셀을 꽉 채움", Frame::new(SIZE, SIZE, vec![character; SIZE * SIZE])),
        ("배경이 덜 빠짐", blob(110, character)),
    ];

    println!("크로마키 제거 전후");
    println!("{}", "-".repeat(62));
    let mut frames = Vec::new();
    for (label, frame) in &raw {
        let before = opaque_pixels(frame);
        let cleaned = remove_chroma(frame, CHROMA_MAGENTA, 12);
        let after = opaque_pixels(&cleaned);
        println!(
            "  {:<16} {:>7} → {:>7} 픽셀   지문 {:016x}",
            label,
            before,
            after,
            content_digest(&cleaned)
        );
        frames.push(cleaned);
    }

    println!();
    println!("검사 결과 (여백 {DEFAULT_SAFE_MARGIN}px, 편차 {DEFAULT_DENSITY_TOLERANCE_PERCENT}%)");
    println!("{}", "-".repeat(62));

    let issues = inspect(&frames, DEFAULT_SAFE_MARGIN, DEFAULT_DENSITY_TOLERANCE_PERCENT);
    if issues.is_empty() {
        println!("  걸린 것 없음");
    }
    for issue in &issues {
        match issue {
            FrameIssue::Empty { index } => {
                println!("  {index}번  비었다 — 배경만 있었거나 전부 지워졌다");
            }
            FrameIssue::TouchesMargin { index } => {
                println!("  {index}번  여백에 닿는다 — 시트에서 옆 프레임과 붙어 보인다");
            }
            FrameIssue::SizeMismatch { index } => {
                println!("  {index}번  크기가 다르다");
            }
            FrameIssue::OutlierDensity { index, pixels, median } => {
                println!("  {index}번  픽셀 수가 튄다 ({pixels} vs 중앙값 {median})");
            }
        }
    }

    println!();
    println!("프레임 하나만 보면 정상인지 알 수 없다.");
    println!("픽셀 수가 많은 건지 적은 건지는 다른 프레임과 비교해야 안다.");
}
