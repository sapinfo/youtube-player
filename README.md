# YouTube Player

YouTube URL을 이름과 함께 저장/관리하고, 골라서 작은 mpv 창으로 재생하는 macOS GUI 앱.

## 사전 요구사항

- [Rust](https://rustup.rs) (2021 edition)
- mpv: `brew install mpv`
- yt-dlp: `brew install yt-dlp` (mpv가 YouTube를 해석할 때 내부적으로 사용)

## 실행

```bash
cargo run
```

릴리스 빌드:

```bash
cargo build --release
./target/release/youtube-player
```

## 사용법

1. 하단 폼에 이름과 YouTube URL을 입력하고 **추가**.
2. 목록에서 항목의 **▶**를 눌러 재생. 영상은 작은 별도 창으로 뜬다.
3. **✎** 수정, **🗑** 삭제, **↑/↓** 순서 이동.
4. 상단 **재생 설정**에서 "항상 위(ontop)"와 창 크기 프리셋(작음/중간/큼)을 조절.
   설정과 목록은 변경 즉시 자동 저장된다.

## 데이터 저장 위치

`~/Library/Application Support/youtube-player/data.json` (macOS)

## mpv 창 크기 조절

재생 중 mpv 창 모서리를 드래그하거나, `Alt+0`(절반) / `Alt+1`(원래) / `Alt+2`(두 배) 단축키 사용.

## 개발

```bash
cargo test    # 단위 테스트 (url / player / store)
cargo build   # 컴파일
```

모듈 구성:

- `src/url.rs` — YouTube URL 검증
- `src/player.rs` — 재생 설정 타입 + mpv 인자 구성/실행
- `src/store.rs` — 데이터 모델, CRUD/순서이동, JSON 영속화
- `src/app.rs` — egui GUI
- `src/main.rs` — 진입점
