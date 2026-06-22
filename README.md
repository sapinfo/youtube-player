# YouTube Player

YouTube URL을 이름과 함께 저장·관리하거나 로컬 동영상 파일을 골라서, 작은 mpv 창으로
재생하는 macOS GUI 앱. 현장/라이브 행사에서 미리 등록해 둔 영상을 빠르게 트는 용도로 만들었다.

## 주요 기능

- YouTube URL을 이름표와 함께 저장 · 수정 · 삭제 · 순서 이동
- 저장한 항목을 골라 별도의 작은 mpv 창으로 재생
- **MP3 음원 추출** — 저장한 YouTube 항목에서 오디오만 뽑아 mp3로 저장 (폴더 선택)
- **로컬 동영상 파일 재생** — 네이티브 파일 선택기로 파일을 골라 바로 재생
- 재생 창 옵션: "항상 위(ontop)" 토글, 창 크기 프리셋(Small/Medium/Large)
- 설정과 목록은 변경 즉시 JSON으로 자동 저장

## 사전 요구사항

- [Rust](https://rustup.rs) (2021 edition) — 소스에서 빌드할 때
- mpv: `brew install mpv` — 실제 재생을 담당
- yt-dlp: `brew install yt-dlp` — mpv가 YouTube URL을 해석할 때 내부적으로 사용
  (로컬 파일만 재생한다면 필요 없음)
- ffmpeg: `brew install ffmpeg` — MP3 음원 추출 시 변환에 사용 (추출 기능을 쓸 때만 필요)

## 설치 / 실행

### 1. `.app`으로 설치 (권장)

```bash
./build-app.sh
cp -R "dist/YouTube Player.app" /Applications/
```

`build-app.sh`가 릴리스 빌드 → 아이콘 생성 → `dist/YouTube Player.app` 번들 조립 →
ad-hoc 코드 서명까지 수행한다. Finder에서 실행해도 Homebrew의 mpv/yt-dlp를 찾도록
PATH를 보강해 둔다.

### 2. 소스에서 바로 실행 (개발용)

```bash
cargo run                       # 디버그 실행
cargo build --release           # 릴리스 빌드
./target/release/youtube-player
```

## 사용법

1. **Add** 폼에 이름과 YouTube URL을 입력하고 **Add**를 눌러 저장.
2. 목록에서 **Play**를 눌러 재생. 영상은 작은 별도 창으로 뜬다.
   **Edit** 수정 · **Delete** 삭제 · **Up / Down** 순서 이동.
3. 각 항목의 **MP3** 버튼을 누르면 저장할 폴더를 고른 뒤, 해당 영상의 오디오를
   `.mp3`로 추출해 그 폴더에 저장한다 (yt-dlp + ffmpeg 사용). 추출은 백그라운드에서
   진행되어 앱이 멈추지 않으며, 하단 상태줄에 진행 상황과 최종 저장 폴더가 표시된다.
4. 로컬 파일은 **Play local file…** 버튼으로 선택기를 열어 바로 재생
   (목록에는 저장되지 않는 일회성 재생).
5. 상단 **Playback settings**에서 "Always on top"과 창 크기(Small/Medium/Large)를 조절.
   설정과 목록은 변경 즉시 자동 저장된다.

> mpv가 설치돼 있지 않으면 상단에 안내가 뜨고 재생 버튼이 비활성화된다.
> 마찬가지로 ffmpeg가 없으면 안내가 뜨고 **MP3** 버튼이 비활성화된다.

## 지원 로컬 파일 형식

파일 선택기의 Video 필터: `mp4`, `mov`, `mkv`, `avi`, `webm`, `m4v`, `flv`, `wmv`,
`mpg`, `mpeg`, `ts`, `m2ts`. (mpv가 재생할 수 있는 형식이면 대부분 동작한다.)

## 데이터 저장 위치

`~/Library/Application Support/youtube-player/data.json` (macOS)

## mpv 창 크기 조절

재생 중 mpv 창 모서리를 드래그하거나, `Alt+0`(절반) / `Alt+1`(원래) / `Alt+2`(두 배)
단축키를 사용한다.

## 개발

```bash
cargo test    # 단위 테스트 (url / player / store)
cargo build   # 컴파일
```

모듈 구성:

- `src/url.rs` — YouTube URL 검증
- `src/player.rs` — 재생 설정 타입 + mpv 인자 구성/실행 (로컬 파일·URL 공통)
- `src/audio.rs` — yt-dlp + ffmpeg로 YouTube 오디오를 mp3로 추출
- `src/store.rs` — 데이터 모델, CRUD/순서이동, JSON 영속화
- `src/app.rs` — egui GUI (저장 목록, 폼, 로컬 파일 선택기, mp3 추출)
- `src/main.rs` — 진입점
