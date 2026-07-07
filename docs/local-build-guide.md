# Local Build Guide — Windows + macOS (Self-Guided)

Get `engine_demo` compiling, tested, and the `ea-sandbox` game running locally.
This single guide includes both Windows and macOS paths so one document stays
up to date for the workshop.

Time required: ~10 minutes (first vcpkg dependency build can add 10–20 minutes).

---

## 1. Prerequisites

| Requirement | Windows | macOS |
|---|---|---|
| Core build tools | Visual Studio 2022 (17.8+) with **Desktop development with C++** workload (includes MSVC, CMake, Ninja, vcpkg) | Xcode Command Line Tools (`xcode-select --install`) + Homebrew |
| CMake + Ninja | Included with VS workload | `brew install cmake ninja` |
| Git | Any recent version | Any recent version |
| vcpkg | VS-bundled (`<VS>\VC\vcpkg`) or standalone clone | Standalone full clone at `$HOME/vcpkg` |
| Disk | ~6 GB free for first dependency build | ~6 GB free for first dependency build |
| Optional tooling | uv + Python 3.11+ for local Specify CLI | uv + Python 3.11+ for local Specify CLI |

Windows: verify your VS install path from PowerShell:

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
```

macOS: create and bootstrap vcpkg if needed:

```bash
git clone https://github.com/microsoft/vcpkg "$HOME/vcpkg"
"$HOME/vcpkg/bootstrap-vcpkg.sh"
```

> vcpkg must be a full clone (not `--depth 1`) so the manifest `builtin-baseline`
> commit resolves.

---

## 2. Open the right shell

Windows: open **Developer PowerShell for VS 2022**, or run:

```powershell
& "<VS>\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -SkipAutomaticLocation
```

> `-Arch amd64` is required. The default shell target can be x86.

macOS: use a normal terminal (zsh) in the repository root.

Verify tools:

```text
cmake --version
ninja --version
```

Windows-only check:

```powershell
cl
```

---

## 3. Set `VCPKG_ROOT`

Windows (VS-bundled vcpkg):

```powershell
$env:VCPKG_ROOT = "<VS>\VC\vcpkg"
```

Optional permanent Windows user env var:

```powershell
[Environment]::SetEnvironmentVariable("VCPKG_ROOT", "<VS>\VC\vcpkg", "User")
```

macOS:

```bash
export VCPKG_ROOT="$HOME/vcpkg"
```

Optional persistent zsh setting:

```bash
echo 'export VCPKG_ROOT="$HOME/vcpkg"' >> "$HOME/.zshrc"
```

Optional macOS step if you build in VS Code with CMake Tools:

- Add the CMake environment setting so the extension resolves vcpkg:

```json
{ "cmake.environment": { "VCPKG_ROOT": "$HOME/vcpkg" } }
```

- In this workshop repo, that setting is in `settings.json`.

---

## 4. Configure, build, test

Run from the repository root.

Windows:

```powershell
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

macOS (Apple Silicon):

```bash
cmake --preset macos-arm64
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

macOS (Intel):

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

Expected results:

- Configure ends with `Build files have been written to: .../build`.
- Build succeeds and produces sandbox binary output in `build/apps/sandbox/`.
- CTest reports all tests passing.

Windows note: warnings like `D9025 ... overriding '/EHs' with '/EHs-'` are expected.

---

## 5. Run the game

Full controls are in [apps/sandbox/README.md](../apps/sandbox/README.md).

macOS app paths (from repository root):

- App bundle path: `build/apps/sandbox/ea-sandbox.app`
- Executable path: `build/apps/sandbox/ea-sandbox.app/Contents/MacOS/ea-sandbox`

Windows interactive:

```powershell
.\build\apps\sandbox\ea-sandbox.exe
```

macOS interactive:

```bash
open build/apps/sandbox/ea-sandbox.app --args --seed 42
```

macOS direct binary (useful for stderr output):

```bash
./build/apps/sandbox/ea-sandbox.app/Contents/MacOS/ea-sandbox --seed 42
```

Cross-platform headless determinism run:

Windows:

```powershell
.\build\apps\sandbox\ea-sandbox.exe --headless --seed 42 --frames 600 --out trace.csv
```

macOS:

```bash
./build/apps/sandbox/ea-sandbox.app/Contents/MacOS/ea-sandbox --headless --seed 42 --frames 600 --out trace.csv
```

Screenshot capture (relative path only):

Windows:

```powershell
.\build\apps\sandbox\ea-sandbox.exe --seed 42 --scene rope --screenshot proof.png --warmup 120
```

macOS:

```bash
./build/apps/sandbox/ea-sandbox.app/Contents/MacOS/ea-sandbox --seed 42 --scene rope --screenshot proof.png --warmup 120
```

---

## 6. Install Spec-Kit (Specify CLI) locally

> This repository already includes `.specify/` and the Copilot prompt wiring, so
> `/speckit.*` prompts work in VS Code without local CLI install. Install Specify
> CLI only if you want `specify check` locally or to scaffold Spec-Kit into your
> own projects.

Windows install:

```powershell
winget install --id astral-sh.uv
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v0.12.4
```

macOS install (Homebrew uv):

```bash
brew install uv
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v0.12.4
```

Verify (both platforms):

```text
specify version
specify check
```

Optional scaffold in your own project:

```text
specify init --here --integration copilot
```

> Do not run `specify init` in this workshop repo because it can overwrite
> customized workshop templates/prompts.

---

## 7. Windows VM fix — software OpenGL (Mesa llvmpipe)

Windows-only. If interactive launch fails with `WGL ... does not appear to support
OpenGL`, copy Mesa software OpenGL DLLs next to the sandbox exe:

```powershell
Invoke-WebRequest "https://github.com/pal1000/mesa-dist-win/releases/download/26.1.3/mesa3d-26.1.3-release-msvc.7z" -OutFile "$env:TEMP\mesa3d.7z"
Invoke-WebRequest "https://www.7-zip.org/a/7zr.exe" -OutFile "$env:TEMP\7zr.exe"

(Get-FileHash "$env:TEMP\mesa3d.7z" -Algorithm SHA256).Hash.ToLower()
# expect: 6dd431f4620cea73970b13e3ffa94f721f2a3924306b8a4283c97648cdb6eb9c

& "$env:TEMP\7zr.exe" x "$env:TEMP\mesa3d.7z" -o"$env:TEMP\mesa3d" -y
Copy-Item "$env:TEMP\mesa3d\x64\*.dll" ".\build\apps\sandbox\" -Force
```

---

## 8. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `Could not find toolchain file ... vcpkg.cmake` | `VCPKG_ROOT` is unset in this shell. Re-export and reconfigure. |
| `cmake` / `ninja` not recognized (Windows) | Not in Developer PowerShell. Redo section 2. |
| Linker mismatch x86 vs x64 (Windows) | Developer shell launched without `-Arch amd64`. |
| `open ea-sandbox.app ...` fails (macOS) | Wrong path from repo root. Use `open build/apps/sandbox/ea-sandbox.app --args ...`. |
| Missing app bundle after build (macOS) | Re-run configure/build from section 4 with `macos-arm64` on Apple Silicon. |
| `WGL ... does not appear to support OpenGL` (Windows VM/RDP) | Apply section 7 Mesa workaround. |
| Screenshot export fails | Use a relative output path. |
| CI/no raylib needed | Configure with `cmake --preset default-debug -DENGINE_DEMO_BUILD_SANDBOX=OFF`. |
| `specify` / `uv` not recognized | Open a new shell and re-run install command. |

---

## 9. Quick reference

Windows:

```powershell
$vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
& "$vs\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -SkipAutomaticLocation
$env:VCPKG_ROOT = "$vs\VC\vcpkg"

cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
.\build\apps\sandbox\ea-sandbox.exe
```

macOS (Apple Silicon):

```bash
export VCPKG_ROOT="$HOME/vcpkg"
cmake --preset macos-arm64
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
open build/apps/sandbox/ea-sandbox.app --args --seed 42
```

You're ready for the workshop. Continue with
[docs/speckit-workshop-training.md](speckit-workshop-training.md).
