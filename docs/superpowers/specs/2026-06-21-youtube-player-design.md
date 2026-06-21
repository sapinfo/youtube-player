# youtube-player 설계 문서

작성일: 2026-06-21

## 개요

YouTube URL을 이름과 함께 저장·관리하고, 현장에서 골라 재생할 수 있는 macOS GUI 런처.
재생은 외부 플레이어 **mpv**가 담당하며, mpv가 내부적으로 yt-dlp를 호출해 YouTube 스트림을
해석한다. 영상은 전체화면이 아닌 **작은 별도 창**으로 떠서, 현장에서 다른 화면을 모니터링하면서
사용할 수 있다.

핵심 사용 시나리오: 미리 여러 YouTube URL을 이름표와 함께 저장해두고, 행사 현장에서 목록을
보고 원하는 항목을 골라 ▶ 한 번으로 즉시 재생한다.

## 런타임 의존성

- **mpv** (필수): 실제 영상 재생. 미설치 시 `brew install mpv` 안내.
- **yt-dlp** (필수, 간접): mpv가 YouTube URL을 해석할 때 내부적으로 호출한다. 우리 코드는
  직접 호출하지 않지만, 없으면 YouTube 재생이 실패한다.

두 도구가 모두 시스템에 설치돼 있어야 한다(개발 환경에서 확인됨: `/opt/homebrew/bin/mpv`,
`/opt/homebrew/bin/yt-dlp`).

## 기술 스택

- 언어: Rust (1.95.0)
- GUI: `eframe` / `egui` — 의존성이 적고 단일 실행파일로 빌드되는 즉시모드 GUI
- 직렬화: `serde` + `serde_json`
- 경로: `directories` 크레이트로 OS 표준 설정 경로 획득
- 프로세스 실행: 표준 라이브러리 `std::process::Command`

## 데이터 모델

```rust
struct Entry {
    name: String,   // 현장에서 알아볼 이름표, 예: "오프닝 영상"
    url: String,    // YouTube URL
}

enum WindowSize {   // mpv 시작 창 크기 프리셋
    Small,          // 480x270
    Medium,         // 640x360
    Large,          // 960x540
}

struct Settings {
    ontop: bool,            // mpv --ontop 적용 여부 (항상 위)
    window_size: WindowSize,
}

struct AppData {            // 디스크에 저장되는 전체 상태
    entries: Vec<Entry>,
    settings: Settings,
}
```

### 저장 위치

`directories`로 얻은 설정 디렉터리 아래 단일 JSON 파일:
`~/Library/Application Support/youtube-player/data.json` (macOS 기준).
디렉터리가 없으면 최초 저장 시 생성한다.

## 컴포넌트(모듈)

각 모듈은 하나의 책임만 가지며 독립적으로 테스트 가능하다.

1. **`main.rs`** — eframe 진입점. 윈도우를 만들고 `App`을 실행한다.
2. **`app.rs`** — egui UI 상태와 렌더링. `AppData`를 메모리에 보유하고, 변경 시 `store`를
   통해 저장한다. mpv 미설치 배너, 목록, 추가/수정 폼, 재생 설정 섹션을 그린다.
3. **`store.rs`** — `AppData`의 JSON 로드/저장과 CRUD 로직.
   - `load() -> AppData` (파일 없으면 기본값)
   - `save(&AppData) -> Result<()>`
   - `add`, `update`, `delete`, `move_up`, `move_down` (entries 조작 순수 함수)
4. **`player.rs`** — mpv 실행.
   - `is_mpv_available() -> bool`
   - `build_args(url, &Settings) -> Vec<String>` (테스트 가능하도록 인자 구성을 분리)
   - `play(url, &Settings) -> Result<()>` (자식 프로세스 spawn)
5. **`url.rs`** — `validate(url) -> bool`: youtube.com / youtu.be 형식의 간단한 검증.

## 화면 구성 (egui)

```
┌─────────────────────────────────────────────┐
│ [배너] mpv가 필요합니다: brew install mpv     │  ← 미설치 시에만 표시
├─────────────────────────────────────────────┤
│ 재생 설정                                     │
│   ☐ 항상 위(ontop)    창 크기: [중간 640x360 ▾] │
├─────────────────────────────────────────────┤
│ 저장된 영상                                   │
│   오프닝 영상   [▶][✎][🗑][↑][↓]              │
│   행사 BGM      [▶][✎][🗑][↑][↓]              │
│   마무리 영상   [▶][✎][🗑][↑][↓]              │
├─────────────────────────────────────────────┤
│ 추가/수정                                     │
│   이름 [____________]  URL [________________] │
│   [추가]   (수정 중이면 [저장] [취소])         │
└─────────────────────────────────────────────┘
```

## 데이터 흐름

1. 앱 시작 → `store::load()`로 `AppData` 읽어 화면 표시. `player::is_mpv_available()`로 배너 결정.
2. 추가/수정/삭제/위·아래 이동 또는 설정 변경 → 메모리 갱신 → 즉시 `store::save()`.
3. ▶ 클릭 → `url::validate()` 통과 시 `player::play(url, &settings)` → mpv가 설정된 크기·
   ontop 여부로 작은 창에서 재생. 실패 시 상태 메시지 표시.

## mpv 실행 옵션

- 항상: `--no-fullscreen`
- 창 크기: `--autofit=<W>x<H>` (프리셋 값)
- 항상 위: `settings.ontop`이 참이면 `--ontop` 추가
- 마지막 인자: URL

예) 항상 위 + 중간 크기 → `mpv --no-fullscreen --autofit=640x360 --ontop <URL>`

## 에러 처리

- mpv 미설치 → 상단 배너 안내 + ▶ 버튼 비활성화.
- yt-dlp 미설치로 인한 재생 실패 → 상태 메시지에 "mpv와 yt-dlp가 모두 설치돼 있어야 합니다" 포함.
- 잘못된/빈 URL → 추가·재생 시 입력란 아래 빨간 안내.
- 저장 실패(권한 등) → 상태 메시지로 표시(앱은 계속 동작).

## 테스트

- `store.rs`: 임시 파일을 이용한 저장/로드 라운드트립, CRUD 및 move_up/move_down 경계
  (목록 맨 위/맨 아래에서 이동) 단위 테스트.
- `url.rs`: 유효(youtube.com/watch, youtu.be) / 무효(빈 문자열, 비 YouTube) 케이스.
- `player.rs`: `build_args`가 설정에 따라 올바른 인자 목록을 만드는지 (ontop on/off, 각 크기 프리셋).

GUI 렌더링(`app.rs`)과 실제 mpv spawn은 자동 테스트 대상에서 제외하고 수동 검증한다.

## 범위 제외 (YAGNI)

- 연속 재생 큐(자동 다음 곡 진행)
- 화질/코덱 선택 UI
- 영상 다운로드 기능
- 폴더·카테고리 분류
- 창 위치(좌표) 지정 — 크기와 ontop만 제어
