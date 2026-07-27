//! 로컬 파일에 둔 기억을 안전하게 다루는 최소 예제.
//!
//! 두 가지를 보여준다.
//!
//! 1. **잠금** — 앱이 두 번 떠도 기록이 섞이지 않게 한다
//! 2. **삭제 확인** — 되돌릴 수 없는 동작은 정확한 문자열을 요구한다
//!
//! 의존성이 없다. `cargo test`로 바로 돌아간다.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

/// 저장소를 쓰지 못한 이유.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// 잠금을 걸 수 없어 저장을 포기했다. 안전하게 못 쓸 바에는 안 쓴다.
    StorageUnavailable,
    /// 삭제 확인 문자열이 정확하지 않다.
    ConfirmationRequired,
}

/// 지울 범위. 전부 아니면 전무로 두면 사용자는 전부를 고를 수밖에 없다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearScope {
    Conversation,
    Preferences,
    Relationship,
    Everything,
}

/// 되돌릴 수 없는 동작이라 불리언이 아니라 정확한 문자열을 받는다.
///
/// `clear(scope, true)` 같은 형태였다면 다른 플래그를 복사하다 실수로 `true`가
/// 들어갈 수 있고, 호출하는 쪽 코드만 봐서는 그 `true`가 무슨 뜻인지 안 보인다.
/// `"DELETE"`는 우연히 만들어지지 않는다.
pub fn require_delete_confirmation(confirmation: &str) -> Result<(), StoreError> {
    if confirmation == "DELETE" {
        Ok(())
    } else {
        Err(StoreError::ConfirmationRequired)
    }
}

/// 잠금을 쥐고 있는 동안만 살아 있는 값. 떨어지면 잠금이 풀린다.
pub struct StoreLock {
    file: File,
}

impl StoreLock {
    /// 잠금을 건다. 이미 다른 쪽이 쥐고 있으면 풀릴 때까지 기다린다.
    pub fn acquire(path: &Path) -> Result<Self, StoreError> {
        let file = open_lock_file(path)?;

        #[cfg(unix)]
        {
            lock_exclusive(&file)?;
            Ok(Self { file })
        }

        // 잠금을 걸 수단이 없는 환경에서는 저장 자체를 거절한다.
        // 잠금 없이 그냥 쓰는 경로를 두지 않는 것이 이 예제의 요지다.
        #[cfg(not(unix))]
        {
            let _ = file;
            Err(StoreError::StorageUnavailable)
        }
    }

    /// 잠금을 시도하고, 이미 잡혀 있으면 기다리지 않고 `None`을 돌려준다.
    /// UI에서 부르는 작업이 무한정 기다리면 창이 멈춘다.
    pub fn try_acquire(path: &Path) -> Result<Option<Self>, StoreError> {
        let file = open_lock_file(path)?;

        #[cfg(unix)]
        {
            if try_lock_exclusive(&file) {
                Ok(Some(Self { file }))
            } else {
                Ok(None)
            }
        }

        #[cfg(not(unix))]
        {
            let _ = file;
            Err(StoreError::StorageUnavailable)
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        // 푸는 실패는 멈춘다고 나아지지 않는다. 돌려줄 곳도 없고, 프로세스가
        // 끝나면 OS가 어차피 정리한다. 거는 쪽만 엄격하면 된다.
        #[cfg(unix)]
        unlock_best_effort(&self.file);
    }
}

fn open_lock_file(path: &Path) -> Result<File, StoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(to_storage_error)?;
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);

    // 대화 기록이 들어가는 자리라 기본 권한에 맡기지 않는다.
    // 기본값은 시스템 설정에 따라 달라지고, 달라지면 다른 계정이 읽을 수 있다.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    options.open(path).map_err(to_storage_error)
}

fn to_storage_error(_error: io::Error) -> StoreError {
    StoreError::StorageUnavailable
}

#[cfg(unix)]
mod unix {
    use super::{File, StoreError};
    use std::os::fd::AsRawFd;

    const LOCK_EX: std::os::raw::c_int = 2;
    const LOCK_NB: std::os::raw::c_int = 4;
    const LOCK_UN: std::os::raw::c_int = 8;

    unsafe extern "C" {
        fn flock(fd: std::os::raw::c_int, operation: std::os::raw::c_int) -> std::os::raw::c_int;
    }

    pub(super) fn lock_exclusive(file: &File) -> Result<(), StoreError> {
        if unsafe { flock(file.as_raw_fd(), LOCK_EX) } == 0 {
            Ok(())
        } else {
            // 경고를 남기고 진행하지 않는다. 기억이 섞이는 것보다
            // 이번 저장을 건너뛰는 편이 낫다.
            Err(StoreError::StorageUnavailable)
        }
    }

    pub(super) fn try_lock_exclusive(file: &File) -> bool {
        unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) == 0 }
    }

    pub(super) fn unlock_best_effort(file: &File) {
        let _ = unsafe { flock(file.as_raw_fd(), LOCK_UN) };
    }
}

#[cfg(unix)]
use unix::{lock_exclusive, try_lock_exclusive, unlock_best_effort};

/// 예제에서 쓰는 잠금 파일 경로.
pub fn lock_path(dir: &Path) -> PathBuf {
    dir.join("memory.lock")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("store-lock-example-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn delete_requires_the_exact_word() {
        assert_eq!(require_delete_confirmation("DELETE"), Ok(()));

        for wrong in ["delete", "Delete", "DELETE ", " DELETE", "yes", "true", ""] {
            assert_eq!(
                require_delete_confirmation(wrong),
                Err(StoreError::ConfirmationRequired),
                "통과하면 안 된다: {wrong:?}"
            );
        }
    }

    #[test]
    fn a_second_holder_cannot_take_the_lock() {
        let dir = temp_dir("second-holder");
        let path = lock_path(&dir);

        let first = StoreLock::acquire(&path).expect("first holder");

        // 이미 잡혀 있으므로 기다리지 않는 쪽은 빈손으로 돌아온다.
        let second = StoreLock::try_acquire(&path).expect("no error");
        assert!(second.is_none(), "두 번째가 잠금을 가져갔다");

        drop(first);

        // 풀리면 다시 잡힌다.
        let third = StoreLock::try_acquire(&path).expect("no error");
        assert!(third.is_some(), "풀린 잠금을 다시 못 잡았다");
    }

    #[cfg(unix)]
    #[test]
    fn lock_file_is_private_to_the_owner() {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_dir("permissions");
        let path = lock_path(&dir);
        let _lock = StoreLock::acquire(&path).expect("lock");

        let mode = std::fs::metadata(&path).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "소유자 외에 열 수 있는 권한이 남았다");
    }
}
