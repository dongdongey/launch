# 프로그램 런처

이 프로그램은 서버 여러 개를 실행하는 게 너무 귀찮아서 만든 아주 단순한 런처입니다.

## 🚀 주요 기능

-   하나의 설정 파일로 여러 개의 작업을 백그라운드 또는 즉시 실행 형태로 관리
-   TOML 기반의 직관적인 설정
-   `start`, `end`, `restart` 명령어로 쉽게 작업 제어
-   작업별 실행 경로, 로그 파일, 환경 변수 지정 가능

## 사용법

기본적으로 홈 디렉터리에 있는 `Launch.toml` 파일을 읽습니다.

### 예시

```toml
# Launch.toml

[[task_name]] # 현재 세션과 독립적으로 백그라운드에서 실행되는 프로그램
command = "command1 ar gu ment"
current_dir = "/current/working/directory1"
log_file = "/your/log/file1.log"

[[task_name]] # 복수의 프로세스를 동시에 실행할 수도 있습니다
command = "command2 ar gu ment"
current_dir = "/current/working/directory1"
log_file = "/your/log/file2.log"
env = { KEY1 = "value1", KEY2 = "value2" }

[instant_task] # 현재 세션에서 실행되고 종료되는 프로그램
command = "cargo run -r"
current_dir = "/my/cargo/project/directoy"

[[another_task]] # 백그라운드 실행 프로그램을 여러 개 지정할 수도 있습니다
command = "another_server"
current_dir = "/another/server/working/directory"

[another_instant_task] # 즉시 실행되는 프로그램도 여러 개 만들 수 있습니다
command = "node ./"
current_dir = "/my/nodejs/project/directoy"
```

-   **현재 세션에서 실행되는 프로그램**은 대괄호 한 겹(`[이름]`)으로 작성합니다.
-   **백그라운드에서 실행되는 프로그램**은 대괄호 두 겹(`[[이름]]`)으로 작성합니다.

`command`는 필수 항목이며, `current_dir`, `log_file`, `env`는 선택 항목입니다.

---

## 명령어

### 백그라운드 실행

```
launch < start | end | restart > <task_name> [옵션...]

예시: launch start task_name
```

### 즉시 실행 (현재 세션에서 실행)

```
launch <task_name> [옵션...]

예시: launch instant_task
```

### 등록된 작업 목록 보기

```
launch list
```

### 옵션

```
--path | -P <경로> : 실행할 Launch.toml 파일의 경로를 지정할 수 있습니다.

예시: launch end another_task --path ./
(현재 디렉터리에 있는 Launch.toml 파일을 기준으로 실행합니다)
```
