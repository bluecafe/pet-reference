//! 서명 없이 배포한 macOS 앱이 왜 "손상되었다"고 나오는지, 그리고 그 표시가
//! 실제로 어디에 붙어 있는지 직접 보는 예제.
//!
//! `cargo test`로 확인하고 `cargo run`으로 임의 경로를 검사한다.

pub mod quarantine;
pub mod walk;

pub use quarantine::{
    is_quarantined, parse_quarantine, read_attr, remove_attr, write_attr, Quarantine,
    QUARANTINE_ATTR,
};
pub use walk::{clear_bundle, scan_bundle, BundleReport};
