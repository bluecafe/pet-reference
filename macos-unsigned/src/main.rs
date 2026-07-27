//! 경로 하나를 받아 격리 표시가 어디에 붙어 있는지 보여준다.
//!
//! ```text
//! cargo run                        # 다운로드된 번들을 흉내 내서 시연
//! cargo run -- ~/Downloads/Some.app  # 실제 경로 검사
//! ```
//!
//! 지우지는 않는다. 남의 파일을 말없이 고치지 않는다.

use std::path::PathBuf;

use macos_unsigned::{
    parse_quarantine, read_attr, scan_bundle, write_attr, QUARANTINE_ATTR,
};

fn main() {
    match std::env::args().nth(1) {
        Some(path) => inspect(PathBuf::from(path)),
        None => demo(),
    }
}

fn inspect(path: PathBuf) {
    if !path.exists() {
        eprintln!("그런 경로가 없다: {}", path.display());
        std::process::exit(1);
    }

    let report = match scan_bundle(&path) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("검사 실패: {error}");
            std::process::exit(1);
        }
    };

    println!("{}", path.display());
    println!("  훑어본 항목  {}", report.scanned);
    println!("  격리된 항목  {}", report.quarantined.len());

    for item in report.quarantined.iter().take(10) {
        let detail = read_attr(item, QUARANTINE_ATTR)
            .ok()
            .flatten()
            .and_then(|raw| parse_quarantine(&raw))
            .map(|q| format!("{} (플래그 {})", q.agent, q.flags))
            .unwrap_or_else(|| "값을 읽지 못함".to_string());
        println!("    {}  ← {}", item.display(), detail);
    }
    if report.quarantined.len() > 10 {
        println!("    ... 그 외 {}개", report.quarantined.len() - 10);
    }

    println!();
    if report.is_clean() {
        println!("격리 표시가 없다. 이대로 열린다.");
    } else {
        println!("이 상태면 \"손상되었기 때문에 열 수 없습니다\"가 뜬다.");
        println!("받은 사람이 지우려면: xattr -cr {}", path.display());
    }
}

fn demo() {
    use std::fs;

    let root = std::env::temp_dir().join("macos-unsigned-demo.app");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Contents/MacOS")).expect("번들 생성");
    fs::write(root.join("Contents/Info.plist"), b"<plist/>").expect("plist");
    let binary = root.join("Contents/MacOS/app");
    fs::write(&binary, b"binary").expect("실행 파일");

    println!("가짜 번들을 만들었다: {}\n", root.display());

    println!("1) 갓 빌드한 상태");
    println!("   격리된 항목 {}", scan_bundle(&root).unwrap().quarantined.len());
    println!("   → 만든 사람 기계에서는 잘 열린다\n");

    println!("2) 다운로드된 상태를 흉내 낸다");
    for path in [&root, &binary] {
        write_attr(path, QUARANTINE_ATTR, b"0081;68a1b2c3;Safari;").expect("속성");
    }
    let report = scan_bundle(&root).unwrap();
    println!("   격리된 항목 {}", report.quarantined.len());
    println!("   → 받은 사람 기계에서 \"손상되었습니다\"가 뜬다\n");

    println!("3) 최상위만 지운다 (xattr -d, 재귀 아님)");
    macos_unsigned::remove_attr(&root, QUARANTINE_ATTR).expect("지우기");
    let report = scan_bundle(&root).unwrap();
    println!("   격리된 항목 {}", report.quarantined.len());
    for item in &report.quarantined {
        println!("     남음: {}", item.display());
    }
    println!("   → 안쪽 실행 파일에 남아 있어 여전히 안 열린다\n");

    println!("4) 재귀로 지운다 (xattr -cr)");
    let cleared = macos_unsigned::clear_bundle(&root).expect("전체 지우기");
    println!("   지운 항목 {cleared}");
    println!("   격리된 항목 {}", scan_bundle(&root).unwrap().quarantined.len());
    println!("   → 이제 열린다");

    fs::remove_dir_all(&root).ok();

    println!();
    println!("-r 이 재귀라는 뜻이고, 그게 필요한 이유가 3번이다.");
    println!(".app은 파일 하나가 아니라 디렉터리다.");
}
