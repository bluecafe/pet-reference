# Desktop AI Pet — 코드 예제

[Desktop AI Pet](https://bluecafe.github.io/pet-releases/download/)을 만들면서 블로그에 쓴 글의 코드를 **혼자 돌아가는 최소 예제**로 옮겨 둔 곳입니다.

앱 저장소를 그대로 공개한 것이 아닙니다. 글에 나온 부분만, 그 글을 읽은 사람이 직접 돌려볼 수 있는 형태로 다시 만듭니다.

## 어떻게 이어지나

| | 어디 | 무엇 |
| --- | --- | --- |
| 읽는다 | [블로그](https://blog.bluecafe.kr) | 왜 그렇게 정했는지 |
| 본다 | **이 저장소** | 그게 코드로 어떻게 생겼는지 |
| 받는다 | [다운로드](https://bluecafe.github.io/pet-releases/download/) | 돌아가는 결과물 |

## 폴더

연재가 진행되면서 한 편에 폴더 하나씩 추가됩니다.

| 폴더 | 주제 | 상태 |
| --- | --- | --- |
| [`overlay-window/`](overlay-window) | 투명·항상 위·클릭 통과 창 | 9 |
| [`sprite-sheet/`](sprite-sheet) | 스프라이트 시트 규격과 매니페스트 | 10 |
| [`asset-pipeline/`](asset-pipeline) | 프레임 정규화와 체크섬 | 10 |
| [`macos-unsigned/`](macos-unsigned) | 서명 없이 macOS 앱 배포 | 7 |
| [`size-measure/`](size-measure) | 번들 크기 실측 | 7 |
| [`stt-filter/`](stt-filter) | 음성 인식 결과에서 환청 걸러내기 | 5 |
| [`prompt-budget/`](prompt-budget) | 좁은 컨텍스트에 맞춰 프롬프트 자르기 | 5 |
| [`store-lock/`](store-lock) | 기억 파일 잠금과 삭제 확인 | 3 |
| [`omok-barks/`](omok-barks) | 결정론으로 캐릭터 대사 고르기 | 6 |

오른쪽 숫자는 각 폴더의 테스트 수입니다. 전부 합쳐 62개입니다.

## 돌려보기

```bash
cargo test                                       # Rust 예제
node --experimental-strip-types --test *.test.ts # TypeScript 예제
```

**`overlay-window`만 Tauri가 필요합니다.** 창이 실제로 떠야 확인되는 내용이라 어쩔 수 없었고, 대신 판정 로직은 따로 떼어 창 없이도 검증됩니다. 나머지 여덟은 의존성이 없습니다 — Rust는 표준 라이브러리만, TypeScript는 Node 22가 타입을 그대로 실행합니다.

각 폴더에는 그 편의 블로그 글 링크와 실행 방법을 담은 `README.md`가 함께 들어갑니다.

## 여기 없는 것

- **캐릭터 에셋** — 스프라이트, 초상, 페르소나 설정. 원본 디자인이 제 것이 아닙니다
- **앱 전체 소스** — 비공개로 둡니다
- **앱에 묶인 배선** — HTTP 클라이언트, 요청 취소, 렌더 루프처럼 원본 맥락 없이는 못 읽는 부분

각 폴더의 README에 "원본과 다른 점"을 적어 뒀습니다. 무엇을 덜어냈는지 알고 읽으시라고 남긴 것입니다.

예제는 캐릭터 없이 도형이나 placeholder로 돌아가게 만듭니다. 구조를 보는 데는 그것으로 충분합니다.

## 라이선스

코드는 [MIT](LICENSE)입니다. 복사해서 쓰시라고 만든 것이니 마음껏 가져가세요.

**앱 배포판은 다릅니다.** [`pet-releases`](https://github.com/bluecafe/pet-releases)의 [이용 조건](https://github.com/bluecafe/pet-releases/blob/main/TERMS.md)을 따르고, 오픈소스 라이선스가 아닙니다. 그 안에 캐릭터 에셋이 들어 있기 때문입니다.

에르 캐릭터의 원본 디자인은 [디스코드 에르메스단](https://discord.gg/7gFbU56ggY)의 Asin_cartel님에게 문의해 주세요.

## 문의

[문의 페이지](https://blog.bluecafe.kr/contact/)에서 받습니다.
