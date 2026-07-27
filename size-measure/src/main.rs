//! 경로를 받아 무엇이 얼마나 차지하는지 보여준다.
//!
//! ```text
//! cargo run -- ~/Downloads/Some.app
//! cargo run                          # 인자가 없으면 현재 폴더
//! ```

use std::path::PathBuf;

use size_measure::{human, measure};

fn main() {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if !root.exists() {
        eprintln!("그런 경로가 없다: {}", root.display());
        std::process::exit(1);
    }

    let report = match measure(&root) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("측정 실패: {error}");
            std::process::exit(1);
        }
    };

    println!("{}", root.display());
    println!("{}", "=".repeat(64));
    println!("  파일        {}", report.entries.len());
    println!("  논리 합계    {:>10}   (ls 가 보여주는 값)", human(report.total_logical()));
    println!("  디스크 사용  {:>10}   (du 가 보여주는 값)", human(report.total_on_disk()));

    if report.total_on_disk() > report.total_logical() {
        let overhead = report.total_on_disk() - report.total_logical();
        println!("  블록 낭비    {:>10}   작은 파일이 많을수록 커진다", human(overhead));
    }
    if report.symlinks > 0 {
        println!("  심볼릭 링크  {:>10}   따라가지 않았다", report.symlinks);
    }
    if report.skipped_hardlinks > 0 {
        println!("  하드링크     {:>10}   실체가 같아 한 번만 셌다", report.skipped_hardlinks);
    }

    println!();
    println!("확장자별");
    println!("{}", "-".repeat(64));
    for (ext, bytes, count) in report.by_extension().into_iter().take(8) {
        let share = if report.total_on_disk() > 0 {
            bytes as f64 / report.total_on_disk() as f64 * 100.0
        } else {
            0.0
        };
        println!("  {:<12} {:>9}  {:>5.1}%  {}개", ext, human(bytes), share, count);
    }

    println!();
    println!("큰 파일");
    println!("{}", "-".repeat(64));
    let prefix = root.to_string_lossy().to_string();
    for entry in report.largest(10) {
        let shown = entry
            .path
            .to_string_lossy()
            .strip_prefix(&prefix)
            .map(|rest| rest.trim_start_matches('/').to_string())
            .unwrap_or_else(|| entry.path.to_string_lossy().to_string());
        println!("  {:>9}  {}", human(entry.on_disk), shown);
    }

    println!();
    println!("\"무겁다\"는 감이고 이 표는 숫자다. 줄이려면 뒤가 필요하다.");
}
