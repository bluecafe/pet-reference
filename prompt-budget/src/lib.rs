//! 컨텍스트가 좁은 로컬 모델에 넣을 프롬프트를 예산에 맞춰 자른다.
//!
//! 핵심은 자르는 기술이 아니라 **버리는 순서**다. 넣을 것은 여섯 가지인데
//! 예산은 하나뿐이라, 무엇을 먼저 포기할지가 곧 캐릭터의 품질이 된다.
//!
//! ```text
//! 1  최근 대화      오래된 것부터
//! 2  예시           최신 것부터
//! 3  선호           최신 것부터
//! 4  요약           잘라 맞추고, 안 되면 비운다
//! 5  페르소나        잘라 맞추고, 안 되면 비운다
//!    사용자의 현재 입력은 자르지 않는다
//! ```
//!
//! 의존성이 없다. `cargo test`로 바로 돌아간다.

/// 프롬프트에 넣을 재료.
#[derive(Clone, Debug, Default)]
pub struct PromptInput {
    /// 캐릭터의 성격과 말투.
    pub persona: String,
    /// 지난 대화를 줄인 것.
    pub summary: String,
    /// 이렇게 말하라는 예시. 앞쪽이 더 중요하다.
    pub examples: Vec<String>,
    /// 사용자가 밝힌 선호. 앞쪽이 더 중요하다.
    pub preferences: Vec<String>,
    /// 최근 주고받은 대화. 앞쪽이 오래된 것이다.
    pub recent_turns: Vec<String>,
    /// 지금 사용자가 한 말. 이것만은 자르지 않는다.
    pub current_user_text: String,
}

/// 프롬프트를 만들지 못한 이유.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildError {
    /// 사용자의 현재 입력이 상한을 넘었다. 자르지 않고 거절한다.
    UserTextTooLarge,
    /// 버릴 수 있는 것을 다 버렸는데도 예산에 안 들어간다.
    BudgetExceeded,
}

/// 무엇을 얼마나 버렸는지 함께 돌려준다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltPrompt {
    pub text: String,
    pub removed_turns: usize,
    pub removed_examples: usize,
    pub removed_preferences: usize,
    pub persona_bytes: usize,
    pub summary_bytes: usize,
}

pub const MAX_PROMPT_BYTES: usize = 32 * 1024;
pub const MAX_PERSONA_BYTES: usize = 16 * 1024;
pub const MAX_SUMMARY_BYTES: usize = 4 * 1024;
pub const MAX_CURRENT_USER_BYTES: usize = 4 * 1024;
pub const MAX_CURRENT_USER_SCALARS: usize = 1_000;
/// 목록은 들어오는 순간 최근 이만큼만 남긴다. 예산 계산은 그 다음이다.
pub const MAX_CONTEXT_ITEMS: usize = 8;

pub fn build_prompt(input: PromptInput) -> Result<BuiltPrompt, BuildError> {
    build_prompt_with_budget(input, MAX_PROMPT_BYTES)
}

pub fn build_prompt_with_budget(
    input: PromptInput,
    budget: usize,
) -> Result<BuiltPrompt, BuildError> {
    // 사용자의 현재 입력은 줄이지 않는다. 질문을 잘라서 넣으면 모델은 잘린 줄
    // 모르고 답하고, 사용자는 자기가 하지 않은 질문의 답을 받는다.
    if input.current_user_text.len() > MAX_CURRENT_USER_BYTES
        || input.current_user_text.chars().count() > MAX_CURRENT_USER_SCALARS
    {
        return Err(BuildError::UserTextTooLarge);
    }

    let mut persona = truncate_utf8_bytes(&input.persona, MAX_PERSONA_BYTES);
    let mut summary = truncate_utf8_bytes(&input.summary, MAX_SUMMARY_BYTES);

    let example_start = input.examples.len().saturating_sub(MAX_CONTEXT_ITEMS);
    let preference_start = input.preferences.len().saturating_sub(MAX_CONTEXT_ITEMS);
    let turn_start = input.recent_turns.len().saturating_sub(MAX_CONTEXT_ITEMS);

    let mut examples = input.examples[example_start..].to_vec();
    let mut preferences = input.preferences[preference_start..].to_vec();
    let mut turns = input.recent_turns[turn_start..].to_vec();

    let mut removed_examples = example_start;
    let mut removed_preferences = preference_start;
    let mut removed_turns = turn_start;

    loop {
        let text = render(
            &persona,
            &examples,
            &summary,
            &preferences,
            &turns,
            &input.current_user_text,
        );

        if text.len() <= budget {
            return Ok(BuiltPrompt {
                persona_bytes: persona.len(),
                summary_bytes: summary.len(),
                text,
                removed_turns,
                removed_examples,
                removed_preferences,
            });
        }

        // 여기가 이 예제의 전부다. 순서를 바꾸면 캐릭터가 달라진다.
        if !turns.is_empty() {
            // 대화는 시간순이라 앞이 오래된 것이다. 방금 한 말을 남긴다.
            turns.remove(0);
            removed_turns += 1;
        } else if examples.pop().is_some() {
            // 예시는 앞쪽이 더 중요하도록 넣어 둔 목록이라 뒤부터 뺀다.
            removed_examples += 1;
        } else if preferences.pop().is_some() {
            removed_preferences += 1;
        } else if !summary.is_empty() {
            // 덩어리는 비우기 전에 잘라서 맞춰 본다.
            summary = halve_or_clear(&summary);
        } else if !persona.is_empty() {
            // 캐릭터의 정체성을 가장 늦게 버린다.
            persona = halve_or_clear(&persona);
        } else {
            // 빈 캐릭터로 답하느니 답하지 않는다.
            return Err(BuildError::BudgetExceeded);
        }
    }
}

fn render(
    persona: &str,
    examples: &[String],
    summary: &str,
    preferences: &[String],
    turns: &[String],
    current_user_text: &str,
) -> String {
    // 신뢰 구간과 비신뢰 구간을 나눈다. 페르소나도 비신뢰 쪽이다 —
    // 사용자가 편집할 수 있으면 거기에 지시문을 적을 수 있다.
    let mut out = String::from("[system] 정책과 응답 형식\n");
    out.push_str("[untrusted_persona]\n");
    out.push_str(persona);
    out.push('\n');
    for example in examples {
        out.push_str("- ");
        out.push_str(example);
        out.push('\n');
    }
    out.push_str("[untrusted_memory]\n");
    out.push_str(summary);
    out.push('\n');
    for preference in preferences {
        out.push_str("- ");
        out.push_str(preference);
        out.push('\n');
    }
    for turn in turns {
        out.push_str("> ");
        out.push_str(turn);
        out.push('\n');
    }
    out.push_str("[untrusted_user_text]\n");
    out.push_str(current_user_text);
    out
}

/// UTF-8 글자 경계를 지키며 앞에서부터 남긴다.
/// 3바이트짜리 한글을 2바이트에서 자르면 깨진 바이트가 남는다.
pub fn truncate_utf8_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

/// 덩어리를 절반으로 줄인다. 더 줄일 수 없으면 비운다.
fn halve_or_clear(value: &str) -> String {
    let target = value.len() / 2;
    if target == 0 {
        return String::new();
    }
    let shortened = truncate_utf8_bytes(value, target);
    if shortened.len() == value.len() {
        String::new()
    } else {
        shortened
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PromptInput {
        PromptInput {
            persona: "P".repeat(512),
            summary: "S".repeat(256),
            examples: vec!["example-a".into(), "example-b".into()],
            preferences: vec!["pref-a".into(), "pref-b".into()],
            recent_turns: vec!["oldest".into(), "middle".into(), "newest".into()],
            current_user_text: "user".into(),
        }
    }

    #[test]
    fn categories_are_removed_in_the_required_order() {
        let baseline = build_prompt_with_budget(sample(), usize::MAX).expect("baseline");

        // 예산을 1바이트씩 줄이면 대화가 가장 먼저 사라진다.
        let one_turn =
            build_prompt_with_budget(sample(), baseline.text.len() - 1).expect("drop a turn");
        assert_eq!(one_turn.removed_turns, 1);
        assert_eq!(one_turn.removed_examples, 0);

        // 대화를 다 버린 뒤에야 예시로 넘어간다.
        let mut budget = one_turn.text.len() - 1;
        let mut built = build_prompt_with_budget(sample(), budget).expect("drop turns");
        while built.removed_turns < 3 {
            budget = built.text.len() - 1;
            built = build_prompt_with_budget(sample(), budget).expect("drop turns");
        }
        assert_eq!(built.removed_examples, 0, "대화가 남았는데 예시를 버렸다");

        budget = built.text.len() - 1;
        let after_turns = build_prompt_with_budget(sample(), budget).expect("drop an example");
        assert_eq!(after_turns.removed_examples, 1);
        assert_eq!(after_turns.removed_preferences, 0);
    }

    #[test]
    fn persona_survives_longer_than_summary() {
        // 예산을 아주 좁히면 요약이 먼저 비고 페르소나가 남는다.
        let mut input = sample();
        input.examples.clear();
        input.preferences.clear();
        input.recent_turns.clear();

        let built = build_prompt_with_budget(input, 260).expect("fits");
        assert_eq!(built.summary_bytes, 0, "요약이 먼저 비어야 한다");
        assert!(built.persona_bytes > 0, "페르소나가 마지막까지 남아야 한다");
    }

    #[test]
    fn oversized_user_text_is_rejected_not_truncated() {
        let mut input = sample();
        input.current_user_text = "x".repeat(MAX_CURRENT_USER_BYTES + 1);
        assert_eq!(
            build_prompt_with_budget(input, usize::MAX),
            Err(BuildError::UserTextTooLarge)
        );

        let mut input = sample();
        input.current_user_text = "한".repeat(MAX_CURRENT_USER_SCALARS + 1);
        assert_eq!(
            build_prompt_with_budget(input, usize::MAX),
            Err(BuildError::UserTextTooLarge)
        );
    }

    #[test]
    fn irreducible_input_fails_instead_of_answering_empty() {
        let input = PromptInput {
            current_user_text: "user".into(),
            ..Default::default()
        };
        assert_eq!(
            build_prompt_with_budget(input, 1),
            Err(BuildError::BudgetExceeded)
        );
    }

    #[test]
    fn truncation_keeps_utf8_boundaries() {
        // 한글 세 글자 9바이트를 8바이트로 자르면 두 글자만 남는다.
        assert_eq!(truncate_utf8_bytes("가나다", 8), "가나");
        assert_eq!(truncate_utf8_bytes("가나다", 9), "가나다");
    }
}
