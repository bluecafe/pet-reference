//! 잠금이 실제로 두 번째 접근을 막는지, 삭제 확인이 무엇을 통과시키는지 보여준다.
//!
//! ```text
//! cargo run
//! ```

use std::path::PathBuf;
use store_lock::{lock_path, require_delete_confirmation, ClearScope, StoreLock};

fn main() {
    let dir: PathBuf = std::env::temp_dir().join("store-lock-demo");
    let path = lock_path(&dir);

    println!("잠금 파일: {}\n", path.display());

    println!("1) 첫 번째가 잠금을 잡는다");
    let first = StoreLock::acquire(&path).expect("첫 번째 잠금");
    println!("   → 성공\n");

    println!("2) 두 번째가 기다리지 않고 시도한다");
    match StoreLock::try_acquire(&path) {
        Ok(Some(_)) => println!("   → 가져갔다 (이러면 안 된다)\n"),
        Ok(None) => println!("   → 빈손. 이미 잡혀 있다\n"),
        Err(error) => println!("   → 실패: {error:?}\n"),
    }

    println!("3) 첫 번째가 놓는다");
    drop(first);
    match StoreLock::try_acquire(&path) {
        Ok(Some(_)) => println!("   → 두 번째가 이제 잡는다\n"),
        Ok(None) => println!("   → 여전히 못 잡는다 (이러면 안 된다)\n"),
        Err(error) => println!("   → 실패: {error:?}\n"),
    }

    println!("삭제 확인");
    println!("{}", "-".repeat(38));
    for candidate in ["DELETE", "delete", "Delete", "DELETE ", "yes", "true", ""] {
        let verdict = match require_delete_confirmation(candidate) {
            Ok(()) => "통과",
            Err(_) => "거절",
        };
        println!("  {:<12} {}", format!("{candidate:?}"), verdict);
    }

    println!();
    println!("지울 범위: {:?}", [
        ClearScope::Conversation,
        ClearScope::Preferences,
        ClearScope::Relationship,
        ClearScope::Everything,
    ]);
    println!("전부 아니면 전무로 두면 사용자는 전부를 고를 수밖에 없다.");
}
