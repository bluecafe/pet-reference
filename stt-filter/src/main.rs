//! 필터가 무엇을 통과시키고 무엇을 버리는지 표로 찍는다.
//!
//! ```text
//! cargo run
//! ```

use stt_filter::{accept_transcript, RejectReason};

fn main() {
    let cases = [
        "안녕하세요",
        "[음악]",
        "[박수]",
        "[Music]",
        "[BLANK_AUDIO]",
        " [웃음] ",
        "[00:12]",
        "[3초 침묵]",
        "[음악] 안녕하세요",
        "[]",
        "[[음악]]",
        "<THINK>숨은 지시</THINK>",
        "```fenced```",
    ];

    println!("{:<28} {:>8}  {}", "입력", "결과", "이유");
    println!("{}", "-".repeat(62));

    for case in cases {
        let (verdict, reason) = match accept_transcript(case) {
            Ok(_) => ("통과", "실제 발화로 본다"),
            Err(RejectReason::Empty) => ("무음", "대괄호 자막 표기 또는 빈 결과"),
            Err(RejectReason::TooLarge) => ("거절", "길이 상한 초과"),
            Err(RejectReason::Unsafe) => ("거절", "프롬프트 경계를 깨뜨린다"),
        };
        println!("{:<28} {:>8}  {}", format!("{case:?}"), verdict, reason);
    }

    println!();
    println!("무음은 오류가 아니다. 아무 말도 안 들린 것처럼 넘긴다 —");
    println!("사용자가 말한 적 없는데 캐릭터가 사과하면 그게 더 이상하다.");
}
