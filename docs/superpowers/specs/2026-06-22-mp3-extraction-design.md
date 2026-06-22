# MP3 추출 기능 설계

## 배경

YouTube Player는 mpv로 영상을 재생하는 Rust/egui 데스크톱 앱이다. mpv는 내부적으로
yt-dlp를 호출해 YouTube를 가져온다. 이 기능은 저장된 YouTube 항목에서 **오디오만
추출해 mp3 파일로 저장**하는 기능을 추가한다.

## 범위

- **대상**: 저장된 YouTube URL 항목만. (로컬 파일 추출, 임의 URL 입력 추출은 범위 밖)
- **저장 위치**: 추출할 때마다 네이티브 폴더 선택 다이얼로그로 사용자가 지정.
- **변환 방식**: yt-dlp가 다운로드 + ffmpeg로 mp3 변환 (`yt-dlp -x --audio-format mp3`).
- **추가 의존성**: ffmpeg (`brew install ffmpeg`). yt-dlp는 mpv가 이미 사용 중.

## 아키텍처

### 새 모듈: `src/audio.rs`

mp3 추출 로직을 담는 신규 모듈. `player.rs`는 mpv 재생 전용으로 유지한다.
PATH 해석 로직(`EXTRA_PATHS`, `augmented_path()`)은 player.rs와 동일한 패턴이
필요하므로, 공유가 깔끔하면 작은 헬퍼로 추출하되 그렇지 않으면 audio.rs에 동일
패턴을 둔다. (`.app`을 Finder에서 실행하면 PATH가 좁아 Homebrew 경로를 못 찾는
문제를 동일하게 해결해야 함.)

함수:

- `yt_dlp_command() -> PathBuf` — `EXTRA_PATHS`에서 `yt-dlp`를 찾고, 없으면 bare name.
- `is_ffmpeg_available() -> bool` — `ffmpeg -version`을 augmented PATH로 실행해 확인.
- `build_extract_args(url: &str, out_dir: &Path) -> Vec<String>` — **순수 함수**, 테스트 대상.
  생성 인자(순서 고정):
  ```
  -x
  --audio-format mp3
  --audio-quality 0
  -o <out_dir>/%(title)s.%(ext)s
  <url>
  ```
- `extract_mp3(url: &str, out_dir: &Path) -> Result<(), String>` — yt-dlp를 spawn 후
  **종료까지 대기**(`output()` 또는 `wait()`). 실패 시 yt-dlp의 stderr를 에러 메시지에
  포함. 이 함수는 블로킹이므로 **백그라운드 스레드에서만 호출**한다.

### UI/동작: `src/app.rs`

`App` 구조체에 추가:
- `extract_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>` — 진행 중인
  추출 결과 수신용. `Ok(저장경로 또는 안내)` / `Err(에러 메시지)`.
- `extracting: bool` — 진행 중 여부 (버튼 비활성화 및 repaint 트리거에 사용).
- `ffmpeg_available: bool` — 시작 시 1회 확인 (mpv처럼).

각 저장 항목 행: 기존 Play 버튼 옆에 `MP3` 버튼 추가. 활성 조건은 mpv처럼
`mpv_available && ffmpeg_available && !extracting`.

클릭 처리:
1. URL을 `url::validate`로 검증. 실패 시 상태에 "Invalid YouTube URL." 표시 후 중단.
2. `rfd::FileDialog::new().pick_folder()`로 출력 폴더 선택. 취소 시 조용히 중단.
3. `mpsc::channel()` 생성, `extract_rx`에 receiver 저장, `extracting = true`,
   상태 "Extracting…".
4. `std::thread::spawn`으로 `extract_mp3` 호출, 결과를 sender로 전송.

`update()` 폴링:
- `extracting`이면 `ctx.request_repaint()`로 다음 프레임을 예약(스레드 완료 시점에
  상태를 반영하기 위함; egui는 이벤트 없으면 다시 그리지 않음).
- `extract_rx`에서 `try_recv()` 시도. 결과 수신 시 `extracting = false`,
  `extract_rx = None`, 상태를 "Saved: <경로>" 또는 에러 메시지로 갱신.

상단 배너: ffmpeg 미설치 시 mpv 안내와 같은 스타일로
"ffmpeg is required: run `brew install ffmpeg` …" 표시.

## 데이터 흐름

```
MP3 클릭 → URL 검증 → 폴더 선택 → 채널 생성 + extracting=true (상태: Extracting…)
   → thread: extract_mp3(url, dir) → yt-dlp 실행/변환 → 결과를 채널로 전송
   → update() try_recv() → extracting=false → 상태: Saved: <경로> / 에러
```

## 에러 처리

- yt-dlp/ffmpeg 미설치: 시작 시 배너 + MP3 버튼 비활성화. (yt-dlp는 mpv 가용성으로
  간접 보장되지만 ffmpeg는 별도 확인.)
- yt-dlp 실행 실패(잘못된 URL, 네트워크 등): stderr 일부를 상태 메시지에 포함.
- 폴더 선택 취소: 아무 동작 없이 종료.
- 동시 추출: `extracting` 플래그로 한 번에 하나만 (MVP). 버튼은 진행 중 비활성화.

## 테스트

- `build_extract_args` 순수 함수 단위 테스트: 인자 순서, `-o` 출력 템플릿(폴더 경로
  결합), URL 위치 검증.
- 실제 네트워크 다운로드/ffmpeg 변환은 테스트하지 않음(외부 의존, 비결정적).

## 범위 밖 (YAGNI)

- 로컬 파일 → mp3, 임의 URL 입력 추출
- 추출 진행률(%) 표시 — 시작/완료 상태만
- mp3 외 포맷 선택, 비트레이트 UI
- 동시 다중 추출 큐
