//! 로컬 음성 인식(Whisper 등)이 돌려준 전사 결과를 받아들이기 전에 거른다.
//!
//! 거르는 이유가 세 가지라 단계도 세 가지다.
//!
//! 1. 조용한 구간에서 모델이 만들어 내는 자막 표기(`[음악]`)를 버린다
//! 2. 길이 상한을 넘는 결과를 거절한다
//! 3. 전사 결과가 프롬프트로 들어갈 때 경계를 깨뜨리는 문자열을 거절한다
//!
//! 의존성이 없다. `cargo test`로 바로 돌아간다.

/// 전사 결과를 받아들이지 않는 이유.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// 아무 말도 들리지 않았다. 대괄호 환청도 여기로 묶인다.
    Empty,
    /// 길이 상한을 넘었다.
    TooLarge,
    /// 프롬프트 경계를 깨뜨릴 수 있는 문자열이 들어 있다.
    Unsafe,
}

/// 바이트 상한. UTF-8 기준이다.
pub const MAX_TRANSCRIPT_BYTES: usize = 4 * 1024;
/// 글자 상한. 한글은 한 글자가 3바이트라 바이트만으로는 언어별 편차가 크다.
pub const MAX_TRANSCRIPT_SCALARS: usize = 1_000;
/// 이 길이를 넘으면 자막 표기로 보지 않는다.
const BRACKET_LABEL_MAX_SCALARS: usize = 32;

/// 전사 결과를 검증한다. 통과하면 앞뒤 공백을 지운 문자열을 돌려준다.
pub fn accept_transcript(raw: &str) -> Result<String, RejectReason> {
    let transcript = raw.trim();

    if transcript.is_empty() || is_bracket_only_transcript(transcript) {
        return Err(RejectReason::Empty);
    }

    if transcript.len() > MAX_TRANSCRIPT_BYTES
        || transcript.chars().count() > MAX_TRANSCRIPT_SCALARS
    {
        return Err(RejectReason::TooLarge);
    }

    if has_unsafe_markup(transcript) {
        return Err(RejectReason::Unsafe);
    }

    Ok(transcript.to_string())
}

/// 대괄호로만 감싸인 짧은 결과인지 본다.
///
/// Whisper는 학습 데이터에 자막이 많이 들어 있어서, 소리가 없는 구간에
/// `[음악]` `[박수]` `[BLANK_AUDIO]` 같은 표기를 만들어 낸다. 실제 발화가 아니다.
///
/// 숫자가 들어 있으면 거르지 않는다. 자막 표기는 라벨이라 숫자가 없고,
/// `[00:12]`처럼 숫자가 들어간 것은 진짜 내용일 가능성이 있기 때문이다.
/// 진짜 발화를 버리는 쪽이 환청을 한 번 통과시키는 쪽보다 나쁘다.
pub fn is_bracket_only_transcript(transcript: &str) -> bool {
    let trimmed = transcript.trim();
    if trimmed.chars().count() > BRACKET_LABEL_MAX_SCALARS {
        return false;
    }
    let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let inner = inner.trim();
    !inner.is_empty()
        && inner
            .chars()
            .all(|character| !matches!(character, '[' | ']') && !character.is_ascii_digit())
}

/// 전사 결과가 그대로 프롬프트에 들어간다면, 음성이 곧 입력 경로가 된다.
/// 사고 태그와 코드블록 경계를 위조하는 문자열을 막는다.
fn has_unsafe_markup(transcript: &str) -> bool {
    let lowercase = transcript.to_ascii_lowercase();
    transcript.chars().any(|character| character <= '\u{001f}')
        || lowercase.contains("<think")
        || transcript.contains("```")
        || transcript.contains("~~~")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_labels_are_treated_as_silence() {
        for hallucination in ["[음악]", "[박수]", "[Music]", "[BLANK_AUDIO]", " [웃음] "] {
            assert_eq!(
                accept_transcript(hallucination),
                Err(RejectReason::Empty),
                "통과하면 안 된다: {hallucination}"
            );
        }
    }

    #[test]
    fn brackets_with_digits_or_extra_text_pass_through() {
        for spoken in ["[00:12]", "[3초 침묵]", "[음악] 안녕하세요", "안녕하세요"] {
            assert!(
                accept_transcript(spoken).is_ok(),
                "거르면 안 된다: {spoken}"
            );
        }
    }

    #[test]
    fn empty_brackets_are_not_labels() {
        // 안이 비어 있으면 자막 표기로 보지 않는다. 알려진 빈틈이다 —
        // 실제로 이런 결과를 본 적이 없어 지금은 막지 않는다.
        assert!(accept_transcript("[]").is_ok());
    }

    #[test]
    fn scalar_limit_catches_hangul_before_the_byte_limit() {
        // 한글 1,000자는 3,000바이트라 바이트 상한에는 못 미치지만 글자 상한에 걸린다.
        assert!(accept_transcript(&"한".repeat(MAX_TRANSCRIPT_SCALARS)).is_ok());
        assert_eq!(
            accept_transcript(&"한".repeat(MAX_TRANSCRIPT_SCALARS + 1)),
            Err(RejectReason::TooLarge)
        );
        assert_eq!(
            accept_transcript(&"x".repeat(MAX_TRANSCRIPT_BYTES + 1)),
            Err(RejectReason::TooLarge)
        );
    }

    #[test]
    fn markup_that_breaks_prompt_boundaries_is_rejected() {
        for injection in [
            "control\u{0007}",
            "<THINK>private</THINK>",
            "```fenced```",
            "~~~fenced~~~",
        ] {
            assert_eq!(
                accept_transcript(injection),
                Err(RejectReason::Unsafe),
                "통과하면 안 된다: {injection}"
            );
        }
    }
}
