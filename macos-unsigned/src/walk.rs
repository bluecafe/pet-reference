//! `.app`은 파일 하나가 아니라 디렉터리다. 그래서 격리 표시도 한 곳이 아니라
//! 안에 있는 파일마다 붙는다.
//!
//! `xattr -cr`의 `-r`이 재귀라는 뜻이고, 그게 필요한 이유가 이것이다.
//! 번들 최상위만 지우면 안에 있는 실행 파일에 표시가 남는다.

use std::io;
use std::path::{Path, PathBuf};

use crate::quarantine::{is_quarantined, remove_attr, QUARANTINE_ATTR};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BundleReport {
    /// 훑어본 항목 수.
    pub scanned: usize,
    /// 격리 표시가 붙어 있던 경로.
    pub quarantined: Vec<PathBuf>,
}

impl BundleReport {
    pub fn is_clean(&self) -> bool {
        self.quarantined.is_empty()
    }
}

/// 번들 안을 훑어 격리 표시가 붙은 경로를 모은다.
pub fn scan_bundle(root: &Path) -> io::Result<BundleReport> {
    let mut report = BundleReport::default();
    visit(root, &mut |path| {
        report.scanned += 1;
        if is_quarantined(path) {
            report.quarantined.push(path.to_path_buf());
        }
    })?;
    Ok(report)
}

/// 번들 안의 격리 표시를 전부 지운다. `xattr -cr`이 하는 일이다.
///
/// 지운 경로 수를 돌려준다. 없는 속성을 지우려다 나는 오류는 무시한다 —
/// 대부분의 파일에는 원래 없고, 그건 실패가 아니라 정상이다.
pub fn clear_bundle(root: &Path) -> io::Result<usize> {
    let mut cleared = 0;
    visit(root, &mut |path| {
        if is_quarantined(path) && remove_attr(path, QUARANTINE_ATTR).is_ok() {
            cleared += 1;
        }
    })?;
    Ok(cleared)
}

/// 디렉터리를 재귀적으로 훑는다. 심볼릭 링크는 따라가지 않는다 —
/// 번들 안에는 링크가 흔하고, 따라가면 같은 파일을 여러 번 보거나 밖으로 나간다.
fn visit(path: &Path, action: &mut impl FnMut(&Path)) -> io::Result<()> {
    action(path);

    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        visit(&entry.path(), action)?;
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::quarantine::write_attr;
    use std::fs;

    fn fake_bundle(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Contents/MacOS")).expect("번들 생성");
        fs::write(root.join("Contents/Info.plist"), b"<plist/>").expect("plist");
        fs::write(root.join("Contents/MacOS/app"), b"binary").expect("실행 파일");
        root
    }

    #[test]
    fn clearing_only_the_top_level_leaves_the_binary_marked() {
        let root = fake_bundle("macos-unsigned-example-nested.app");
        let binary = root.join("Contents/MacOS/app");

        // 다운로드된 번들을 흉내 낸다. 표시는 안쪽 파일에도 붙는다.
        write_attr(&root, QUARANTINE_ATTR, b"0081;68a1b2c3;Safari;").expect("루트");
        write_attr(&binary, QUARANTINE_ATTR, b"0081;68a1b2c3;Safari;").expect("실행 파일");

        // 최상위만 지우면 안쪽이 남는다. 앱은 여전히 안 열린다.
        remove_attr(&root, QUARANTINE_ATTR).expect("루트 지우기");
        let report = scan_bundle(&root).expect("검사");
        assert_eq!(report.quarantined, vec![binary.clone()]);
        assert!(!report.is_clean());

        // 재귀로 지워야 깨끗해진다.
        let cleared = clear_bundle(&root).expect("전체 지우기");
        assert_eq!(cleared, 1);
        assert!(scan_bundle(&root).expect("재검사").is_clean());

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_freshly_built_bundle_has_no_quarantine() {
        // 직접 빌드한 앱에는 표시가 없다. 그래서 만든 사람 기계에서는 잘 열리고,
        // 받은 사람 기계에서만 막힌다. 이 차이가 문제를 늦게 발견하게 만든다.
        let root = fake_bundle("macos-unsigned-example-fresh.app");
        let report = scan_bundle(&root).expect("검사");
        assert!(report.is_clean());
        assert!(report.scanned >= 4);
        fs::remove_dir_all(&root).ok();
    }
}
