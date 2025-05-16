# 프로그램 런쳐

이 프로그램은 그냥 서버 두개이상 실행 시키는 거 레전드로다가 귀찮아서 만든거야.

## 사용법

이 프로그램은 기본적으로는 홈 디렉토리에 있는 `Launch.toml` 파일을 찾습니다.

### 예시

```toml
# Launch.toml

[[task_name]]# 현재 세션과 독립적으로 백그라운드에서 실행되는 프로그램
command = "command1 ar gu ment"
current_dir = "/current/working/directory1"
log_file = "/your/log/file1.log"


[[task_name]] # 복수의 프로세스를 동시에 돌릴 수도 있음
command = "command2 ar gu ment"
current_dir = "/current/working/directory1"
log_file = "/your/log/file2.log"
env = { KEY1 = "value1", KEY2 = "value2" }

[instant_task] # 현재 세션에서 실행되고 종료되는 프로그램
command = "cargo run -r"
current_dir = "/my/cargo/project/directoy"


[[another_task]] # 현재 세션과 독립적으로 실행되는 프로그램 여러개 만들 수도 있음
command = "another_server"
current_dir = "/another/server/working/directory"

[another_instant_task] # 이것또한 여러개 만들 수 있음
command = "node ./"
current_dir = "/my/nodejs/project/directoy"
```

현재 세션에서 실행되는 프로그램은 이름을 대괄호 한 겹으로 감쌉니다. ( [이름] )

백그라운드에서 실행되는 프로그램은 이름을 대괄호 두 겹으로 감쌉니다. ( [[이름]] )

command는 필수 요소입니다. current_dir, log_file, env는 선택사항입니다.

## 명령어

백그라운드 실행

```
launch < start | end | restart > < task name > [option...]

example : launch start task_name
```

현재 세션에서 실행

```
launch < task name > [option...]

example : launch instant_task
```

목록 보기

```
launch list
```

Option:

```
--path | -P < path > : 내가 원하는 디렉토리나 파일에 대하여 launch를 실행 시킬 수 있습니다.

    example : launch end another_task --path ./ (현재 디렉토리에 있는 Launch.toml 파일을 찾아서 실행)

```
