//! 디렉터리 트리에서 무엇이 얼마나 차지하는지 잰다.
//!
//! "앱이 무겁다"는 감이고, "이 파일이 9.9M"은 숫자다. 줄이려면 뒤가 필요하다.
//!
//! 재는 일이 단순해 보이지만 세 군데서 갈린다.
//!
//! 1. 논리 크기와 디스크 사용량이 다르다
//! 2. 심볼릭 링크를 따라가면 같은 것을 여러 번 센다
//! 3. 하드링크는 실체가 하나인데 여러 경로에서 보인다
//!
//! 의존성이 없다.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// 한 항목의 크기.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    /// 파일이 담고 있는 바이트 수. `ls -l`이 보여주는 값이다.
    pub logical: u64,
    /// 디스크가 실제로 내준 크기. `du`가 보여주는 값이다.
    pub on_disk: u64,
}

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub entries: Vec<Entry>,
    /// 하드링크라서 두 번 세지 않은 항목 수.
    pub skipped_hardlinks: usize,
    /// 따라가지 않은 심볼릭 링크 수.
    pub symlinks: usize,
}

impl Report {
    pub fn total_logical(&self) -> u64 {
        self.entries.iter().map(|entry| entry.logical).sum()
    }

    pub fn total_on_disk(&self) -> u64 {
        self.entries.iter().map(|entry| entry.on_disk).sum()
    }

    /// 큰 것부터 n개. 전체 목록은 사람이 못 읽는다.
    pub fn largest(&self, n: usize) -> Vec<&Entry> {
        let mut sorted: Vec<&Entry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.on_disk.cmp(&a.on_disk).then(a.path.cmp(&b.path)));
        sorted.into_iter().take(n).collect()
    }

    /// 확장자별 합계. 무엇을 줄여야 하는지는 대개 여기서 나온다.
    pub fn by_extension(&self) -> Vec<(String, u64, usize)> {
        let mut totals: HashMap<String, (u64, usize)> = HashMap::new();
        for entry in &self.entries {
            let key = entry
                .path
                .extension()
                .map(|ext| ext.to_string_lossy().to_lowercase())
                .unwrap_or_else(|| "(없음)".to_string());
            let slot = totals.entry(key).or_insert((0, 0));
            slot.0 += entry.on_disk;
            slot.1 += 1;
        }
        let mut rows: Vec<(String, u64, usize)> = totals
            .into_iter()
            .map(|(ext, (bytes, count))| (ext, bytes, count))
            .collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        rows
    }
}

/// 트리를 훑어 파일 크기를 모은다.
///
/// 심볼릭 링크는 따라가지 않는다. `.app` 번들의 `Frameworks` 아래에는 버전
/// 폴더를 가리키는 링크가 여러 개 있어서, 따라가면 같은 것을 몇 번씩 센다.
/// 링크가 밖을 가리키면 번들 밖의 크기까지 합쳐진다.
pub fn measure(root: &Path) -> io::Result<Report> {
    let mut report = Report::default();
    let mut seen_inodes: HashSet<(u64, u64)> = HashSet::new();
    walk(root, &mut report, &mut seen_inodes)?;
    Ok(report)
}

fn walk(path: &Path, report: &mut Report, seen: &mut HashSet<(u64, u64)>) -> io::Result<()> {
    // symlink_metadata는 링크 자체를 본다. metadata는 링크가 가리키는 대상을 본다.
    let metadata = std::fs::symlink_metadata(path)?;

    if metadata.file_type().is_symlink() {
        report.symlinks += 1;
        return Ok(());
    }

    if metadata.is_dir() {
        for child in std::fs::read_dir(path)? {
            walk(&child?.path(), report, seen)?;
        }
        return Ok(());
    }

    #[cfg(unix)]
    {
        // 하드링크는 실체가 하나인데 여러 경로에서 보인다. 같은 (기기, inode)를
        // 두 번 세면 실제보다 큰 값이 나온다.
        if metadata.nlink() > 1 && !seen.insert((metadata.dev(), metadata.ino())) {
            report.skipped_hardlinks += 1;
            return Ok(());
        }
    }

    report.entries.push(Entry {
        path: path.to_path_buf(),
        logical: metadata.len(),
        on_disk: disk_usage(&metadata),
    });
    Ok(())
}

/// 디스크가 실제로 내준 크기.
///
/// 파일 시스템은 블록 단위로 자리를 준다. 1바이트짜리 파일도 블록 하나를
/// 차지하므로, 작은 파일이 많으면 논리 크기 합계보다 실제가 훨씬 크다.
#[cfg(unix)]
fn disk_usage(metadata: &std::fs::Metadata) -> u64 {
    // blocks()는 512바이트 단위다. 파일 시스템의 블록 크기와 다르다.
    metadata.blocks() * 512
}

#[cfg(not(unix))]
fn disk_usage(metadata: &std::fs::Metadata) -> u64 {
    metadata.len()
}

/// 사람이 읽는 크기. 소수점 한 자리까지만 — 그 이하는 판단에 영향을 주지 않는다.
pub fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes}{}", UNITS[0])
    } else {
        format!("{value:.1}{}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_tree(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Contents/Resources")).expect("트리 생성");
        root
    }

    #[test]
    fn sums_files_across_directories() {
        let root = temp_tree("size-measure-example-sum");
        fs::write(root.join("Contents/binary"), vec![0u8; 4096]).expect("파일");
        fs::write(root.join("Contents/Resources/image.png"), vec![0u8; 8192]).expect("파일");

        let report = measure(&root).expect("측정");
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.total_logical(), 4096 + 8192);

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn tiny_files_cost_more_than_their_contents() {
        let root = temp_tree("size-measure-example-blocks");
        // 1바이트 파일 열 개. 논리 합계는 10바이트다.
        for i in 0..10 {
            fs::write(root.join(format!("Contents/tiny{i}")), b"x").expect("파일");
        }

        let report = measure(&root).expect("측정");
        assert_eq!(report.total_logical(), 10);
        // 디스크는 블록 단위로 준다. 훨씬 크게 나온다.
        assert!(
            report.total_on_disk() > report.total_logical(),
            "on_disk={} logical={}",
            report.total_on_disk(),
            report.total_logical()
        );

        fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_followed() {
        let root = temp_tree("size-measure-example-symlink");
        let real = root.join("Contents/big.bin");
        fs::write(&real, vec![0u8; 16384]).expect("파일");
        std::os::unix::fs::symlink(&real, root.join("Contents/alias.bin")).expect("링크");

        let report = measure(&root).expect("측정");
        // 따라갔다면 32768이 나온다.
        assert_eq!(report.total_logical(), 16384);
        assert_eq!(report.symlinks, 1);
        assert_eq!(report.entries.len(), 1);

        fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn hard_links_are_counted_once() {
        let root = temp_tree("size-measure-example-hardlink");
        let real = root.join("Contents/lib.dylib");
        fs::write(&real, vec![0u8; 16384]).expect("파일");
        fs::hard_link(&real, root.join("Contents/Resources/lib.dylib")).expect("하드링크");

        let report = measure(&root).expect("측정");
        // 실체가 하나다. 두 번 세면 32768이 나온다.
        assert_eq!(report.total_logical(), 16384);
        assert_eq!(report.skipped_hardlinks, 1);

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn groups_by_extension() {
        let root = temp_tree("size-measure-example-ext");
        fs::write(root.join("Contents/a.png"), vec![0u8; 8192]).expect("파일");
        fs::write(root.join("Contents/b.png"), vec![0u8; 4096]).expect("파일");
        fs::write(root.join("Contents/c.json"), vec![0u8; 1024]).expect("파일");
        fs::write(root.join("Contents/binary"), vec![0u8; 2048]).expect("파일");

        let report = measure(&root).expect("측정");
        let rows = report.by_extension();

        assert_eq!(rows[0].0, "png");
        assert_eq!(rows[0].2, 2, "png 파일이 두 개여야 한다");
        assert!(rows.iter().any(|(ext, _, _)| ext == "(없음)"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn largest_is_sorted_and_capped() {
        let root = temp_tree("size-measure-example-largest");
        for (name, size) in [("small", 1024), ("big", 65536), ("mid", 8192)] {
            fs::write(root.join(format!("Contents/{name}")), vec![0u8; size]).expect("파일");
        }

        let report = measure(&root).expect("측정");
        let top = report.largest(2);
        assert_eq!(top.len(), 2);
        assert!(top[0].path.ends_with("big"));
        assert!(top[1].path.ends_with("mid"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn human_readable_sizes() {
        assert_eq!(human(512), "512B");
        assert_eq!(human(1024), "1.0K");
        assert_eq!(human(1024 * 1024), "1.0M");
        assert_eq!(human(10_313_160), "9.8M");
    }
}
