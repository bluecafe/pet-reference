//! 예산을 좁혀 가며 무엇이 먼저 사라지는지 보여준다.
//!
//! ```text
//! cargo run
//! ```

use prompt_budget::{build_prompt_with_budget, BuildError, PromptInput};

fn sample() -> PromptInput {
    PromptInput {
        persona: "너는 창틀 위를 걸어다니는 작은 캐릭터다. ".repeat(8),
        summary: "사용자는 어제 배포 실패를 겪었다. ".repeat(4),
        examples: vec!["안녕! 오늘도 왔네".into(), "그건 좀 어려운데".into()],
        preferences: vec!["존댓말 싫어함".into(), "짧은 답 선호".into()],
        recent_turns: vec![
            "오래된 대화".into(),
            "중간 대화".into(),
            "방금 한 말".into(),
        ],
        current_user_text: "오늘 뭐 해?".into(),
    }
}

fn main() {
    let full = build_prompt_with_budget(sample(), usize::MAX).expect("baseline");
    println!("자르지 않았을 때: {} bytes\n", full.text.len());

    println!(
        "{:>8}  {:>6} {:>6} {:>6} {:>8} {:>8}",
        "예산", "대화↓", "예시↓", "선호↓", "요약B", "페르소나B"
    );
    println!("{}", "-".repeat(52));

    let mut budget = full.text.len();
    loop {
        match build_prompt_with_budget(sample(), budget) {
            Ok(built) => {
                println!(
                    "{:>8}  {:>6} {:>6} {:>6} {:>8} {:>8}",
                    budget,
                    built.removed_turns,
                    built.removed_examples,
                    built.removed_preferences,
                    built.summary_bytes,
                    built.persona_bytes
                );
                if budget < 40 {
                    break;
                }
                budget = built.text.len().saturating_sub(1);
            }
            Err(BuildError::BudgetExceeded) => {
                println!("{budget:>8}  더 버릴 것이 없다 → 실패로 끝낸다");
                break;
            }
            Err(other) => {
                println!("{other:?}");
                break;
            }
        }
    }

    println!();
    println!("대화가 먼저 사라지고, 페르소나가 마지막까지 남는다.");
    println!("빈 캐릭터로 답하느니 답하지 않는다.");
}
