# Herdr Sidebar `dev` 브랜치 설치 안내

이 문서는 fork 저장소의 `dev` 브랜치로 Herdr Sidebar를 설치하거나,
로컬 checkout을 개발용 플러그인으로 연결하는 방법을 정리합니다.

## 1. 원격 `dev` 브랜치 설치

먼저 개발 브랜치를 fork 원격 저장소에 올립니다.

```bash
git push -u origin dev
```

기존에 설치된 동일 플러그인이 있다면 제거한 뒤, `--ref dev`로 설치합니다.

```bash
herdr plugin uninstall herdr-sidebar
herdr plugin install smilejk930/herdr-sidebar/plugins/herdr-sidebar --ref dev --yes
```

`dev`는 release tag가 아닌 개발 브랜치이므로, 설치 과정에서 해당 checkout의 소스를
release build합니다. 따라서 Rust 1.89 이상과 Cargo가 필요합니다.

설치 후 Herdr를 다시 열거나 sidebar를 토글합니다.

```bash
herdr plugin action invoke herdr-sidebar.open-sidebar
```

Windows에서는 아래 Windows용 action을 사용합니다.

```powershell
herdr plugin action invoke herdr-sidebar.open-sidebar-windows
```

## 2. 현재 로컬 `dev` checkout 연결

원격 push 전에 현재 작업 디렉터리의 소스를 바로 시험하려면 이 방식을 사용합니다.

```bash
cd /home/smilejk930/develop/workspace/herdr-sidebar/plugins/herdr-sidebar
cargo build --release
herdr plugin link .
```

기존 원격 설치가 있으면 `link`가 같은 plugin id를 로컬 checkout으로 교체합니다.
Herdr가 실행 중이라면 빌드 후 아래 action으로 실행 중인 sidebar를 갱신합니다.

```bash
herdr plugin action invoke redeploy --plugin herdr-sidebar
```

## 3. 원격 설치로 복귀

로컬 연결을 제거한 뒤 원하는 원격 브랜치를 다시 설치합니다.

```bash
herdr plugin unlink herdr-sidebar
herdr plugin install smilejk930/herdr-sidebar/plugins/herdr-sidebar --ref dev --yes
```

## 확인

```bash
herdr plugin list --plugin herdr-sidebar --json
git branch --show-current
```

첫 명령에서 플러그인의 설치 정보와 연결 상태를 확인하고, 두 번째 명령에서 현재
checkout이 `dev`인지 확인합니다.
