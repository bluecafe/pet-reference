//! `com.apple.quarantine` 확장 속성을 직접 읽고 지운다.
//!
//! macOS는 인터넷에서 받은 파일에 이 속성을 붙인다. Gatekeeper가 이걸 보고
//! 서명과 공증 상태를 확인하는데, 애드혹 서명뿐이면 "손상되었기 때문에 열 수
//! 없습니다"가 뜬다. 앱이 깨진 게 아니라 서명자를 확인할 수 없다는 뜻이다.
//!
//! 의존성 없이 libc를 직접 선언해 쓴다.

use std::ffi::CString;
use std::io;
use std::path::Path;

/// 격리 속성의 이름. 값이 아니라 이름이 붙어 있다는 것 자체가 표시다.
pub const QUARANTINE_ATTR: &str = "com.apple.quarantine";

#[cfg(target_os = "macos")]
mod sys {
    use std::os::raw::{c_char, c_int, c_void};

    pub const XATTR_NOFOLLOW: c_int = 0x0001;

    unsafe extern "C" {
        pub fn getxattr(
            path: *const c_char,
            name: *const c_char,
            value: *mut c_void,
            size: usize,
            position: u32,
            options: c_int,
        ) -> isize;

        pub fn removexattr(path: *const c_char, name: *const c_char, options: c_int) -> c_int;

        pub fn setxattr(
            path: *const c_char,
            name: *const c_char,
            value: *const c_void,
            size: usize,
            position: u32,
            options: c_int,
        ) -> c_int;
    }
}

fn to_cstring(path: &Path) -> io::Result<CString> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "경로에 널 바이트가 있다"))
}

/// 확장 속성 값을 읽는다. 없으면 `Ok(None)`이다.
#[cfg(target_os = "macos")]
pub fn read_attr(path: &Path, name: &str) -> io::Result<Option<Vec<u8>>> {
    let path_c = to_cstring(path)?;
    let name_c = CString::new(name)?;

    // 크기를 먼저 물어본다. 0을 넘기면 필요한 바이트 수를 돌려준다.
    let size = unsafe {
        sys::getxattr(
            path_c.as_ptr(),
            name_c.as_ptr(),
            std::ptr::null_mut(),
            0,
            0,
            sys::XATTR_NOFOLLOW,
        )
    };
    if size < 0 {
        // 속성이 없는 것은 오류가 아니다. 대부분의 파일에는 원래 없다.
        return Ok(None);
    }

    let mut buffer = vec![0u8; size as usize];
    let read = unsafe {
        sys::getxattr(
            path_c.as_ptr(),
            name_c.as_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            0,
            sys::XATTR_NOFOLLOW,
        )
    };
    if read < 0 {
        return Ok(None);
    }
    buffer.truncate(read as usize);
    Ok(Some(buffer))
}

/// 확장 속성을 붙인다. 예제에서 격리 상태를 흉내 낼 때 쓴다.
#[cfg(target_os = "macos")]
pub fn write_attr(path: &Path, name: &str, value: &[u8]) -> io::Result<()> {
    let path_c = to_cstring(path)?;
    let name_c = CString::new(name)?;
    let result = unsafe {
        sys::setxattr(
            path_c.as_ptr(),
            name_c.as_ptr(),
            value.as_ptr().cast(),
            value.len(),
            0,
            sys::XATTR_NOFOLLOW,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// 확장 속성을 지운다. `xattr -d`가 하는 일이다.
#[cfg(target_os = "macos")]
pub fn remove_attr(path: &Path, name: &str) -> io::Result<()> {
    let path_c = to_cstring(path)?;
    let name_c = CString::new(name)?;
    let result =
        unsafe { sys::removexattr(path_c.as_ptr(), name_c.as_ptr(), sys::XATTR_NOFOLLOW) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn read_attr(_path: &Path, _name: &str) -> io::Result<Option<Vec<u8>>> {
    Ok(None)
}

#[cfg(not(target_os = "macos"))]
pub fn write_attr(_path: &Path, _name: &str, _value: &[u8]) -> io::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn remove_attr(_path: &Path, _name: &str) -> io::Result<()> {
    Ok(())
}

/// 격리 속성의 값. 세미콜론으로 구분된 네 칸이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quarantine {
    /// 플래그. 16진수 문자열로 들어온다.
    pub flags: String,
    /// 붙은 시각. 16진수 유닉스 타임스탬프다.
    pub timestamp_hex: String,
    /// 받아온 프로그램 이름. Safari, Chrome, curl 등.
    pub agent: String,
    /// 이벤트 식별자. 없을 수도 있다.
    pub event_id: String,
}

/// 격리 속성 값을 뜯는다.
///
/// 형식이 문서화돼 있지 않아서 실제 값을 보고 맞췄다. 칸이 모자라면 빈
/// 문자열로 채운다 — 여기서 실패로 처리하면 macOS 버전이 형식을 바꿀 때
/// 도구가 통째로 멈춘다.
pub fn parse_quarantine(value: &[u8]) -> Option<Quarantine> {
    let text = std::str::from_utf8(value).ok()?;
    let mut parts = text.split(';');
    Some(Quarantine {
        flags: parts.next()?.to_string(),
        timestamp_hex: parts.next().unwrap_or_default().to_string(),
        agent: parts.next().unwrap_or_default().to_string(),
        event_id: parts.next().unwrap_or_default().to_string(),
    })
}

/// 이 경로에 격리 표시가 붙어 있는지.
pub fn is_quarantined(path: &Path) -> bool {
    matches!(read_attr(path, QUARANTINE_ATTR), Ok(Some(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_quarantine_value() {
        // Safari로 받은 파일에서 볼 수 있는 형태다.
        let value = b"0081;68a1b2c3;Safari;1A2B3C4D-5E6F-7A8B-9C0D-1E2F3A4B5C6D";
        let parsed = parse_quarantine(value).expect("파싱");
        assert_eq!(parsed.flags, "0081");
        assert_eq!(parsed.agent, "Safari");
        assert_eq!(parsed.timestamp_hex, "68a1b2c3");
    }

    #[test]
    fn tolerates_missing_fields() {
        // 칸이 모자라도 멈추지 않는다. 형식이 바뀌어도 도구는 계속 돈다.
        let parsed = parse_quarantine(b"0081;68a1b2c3").expect("파싱");
        assert_eq!(parsed.agent, "");
        assert_eq!(parsed.event_id, "");
    }

    #[test]
    fn rejects_non_utf8() {
        assert!(parse_quarantine(&[0xff, 0xfe]).is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn attribute_round_trip() {
        use std::fs;

        let path = std::env::temp_dir().join("macos-unsigned-example-roundtrip.txt");
        fs::write(&path, b"payload").expect("파일 생성");

        // 갓 만든 파일에는 격리가 없다. 인터넷에서 온 게 아니기 때문이다.
        assert!(!is_quarantined(&path));

        // 다운로드된 상태를 흉내 낸다.
        write_attr(&path, QUARANTINE_ATTR, b"0081;68a1b2c3;Safari;").expect("속성 붙이기");
        assert!(is_quarantined(&path));

        let raw = read_attr(&path, QUARANTINE_ATTR).expect("읽기").expect("값");
        assert_eq!(parse_quarantine(&raw).expect("파싱").agent, "Safari");

        // xattr -cr이 하는 일이다.
        remove_attr(&path, QUARANTINE_ATTR).expect("속성 지우기");
        assert!(!is_quarantined(&path));

        fs::remove_file(&path).ok();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn removing_a_missing_attribute_is_an_error_not_a_crash() {
        use std::fs;

        let path = std::env::temp_dir().join("macos-unsigned-example-missing.txt");
        fs::write(&path, b"payload").expect("파일 생성");

        // 없는 속성을 지우면 실패한다. 도구는 이걸 "이미 깨끗함"으로 다뤄야 한다.
        assert!(remove_attr(&path, QUARANTINE_ATTR).is_err());
        assert!(!is_quarantined(&path));

        fs::remove_file(&path).ok();
    }
}
