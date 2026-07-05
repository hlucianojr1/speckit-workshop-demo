# Local Build Guide — Windows (Self-Guided)

Get `engine_demo` compiling, tested, and the `ea-sandbox` game running on a Windows
machine or VM. Every step below was verified on a Windows VM with Visual Studio 2022
Enterprise 17.14 and no dedicated GPU.

Time required: ~10 minutes (first vcpkg dependency build can add 10–20 minutes).

---

## 1. Prerequisites

| Requirement | Notes |
|---|---|
| **Visual Studio 2022** (17.8+) | With the **"Desktop development with C++"** workload. This bundles MSVC, CMake, Ninja, **and vcpkg** — no separate installs needed. |
| **Git** | Any recent version. |
| ~6 GB free disk | vcpkg builds EASTL, GoogleTest, fmt, nlohmann-json, and raylib from source on first configure. |
| *(Optional)* **uv** + **Python 3.11+** | Only needed for the Spec-Kit (Specify) CLI in step 6. The `/speckit.*` slash commands in this repo work without it. |

> **Don't have the C++ workload?** Open *Visual Studio Installer* → *Modify* → check
> **Desktop development with C++** → ensure "C++ CMake tools for Windows" and
> "vcpkg package manager" are selected under Installation details → *Modify*.

Verify from a regular PowerShell:

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
```

This prints your VS install path (e.g. `C:\Program Files\Microsoft Visual Studio\2022\Enterprise`).
You'll need it below — referred to as `<VS>` from here on.

---

## 2. Open a Developer PowerShell

The build tools (cl, cmake, ninja) are **not** on the default PATH. You must work
inside a *Developer PowerShell for VS 2022*. Either:

- **Start menu** → "Developer PowerShell for VS 2022", **or**
- From any PowerShell (replace `<VS>` with your install path):

```powershell
& "<VS>\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -SkipAutomaticLocation
```

> **Important:** `-Arch amd64` matters. The default dev shell targets x86 and the
> build will fail linking 64-bit libraries.

Confirm the tools resolve:

```powershell
cmake --version   # expect 3.28+
ninja --version
cl                # prints "Microsoft (R) C/C++ Optimizing Compiler ... for x64"
```

---

## 3. Point `VCPKG_ROOT` at vcpkg

The CMake preset reads the `VCPKG_ROOT` environment variable. Use the copy bundled
with Visual Studio (no clone required):

```powershell
$env:VCPKG_ROOT = "<VS>\VC\vcpkg"
```

To make it permanent (survives new terminals):

```powershell
[Environment]::SetEnvironmentVariable("VCPKG_ROOT", "<VS>\VC\vcpkg", "User")
```

> **Alternative:** if you prefer a standalone vcpkg, `git clone https://github.com/microsoft/vcpkg C:\vcpkg`
> (a **full** clone — not `--depth 1` — so the manifest's `builtin-baseline` resolves)
> and point `VCPKG_ROOT` there instead.

---

## 4. Configure, build, test

From the repository root (`C:\code\speckit-workshop-demo`):

```powershell
cmake --preset default-debug                       # configure (vcpkg installs deps here)
cmake --build --preset default-debug               # build library + tests + sandbox
ctest --preset default-debug --output-on-failure   # run the GoogleTest suite
```

Expected results:

- Configure ends with `Build files have been written to: .../build`.
- Build completes with no errors. You **will** see many
  `cl : Command line warning D9025 : overriding '/EHs' with '/EHs-'` warnings —
  these are **benign** (the project deliberately disables exceptions per the
  constitution) and can be ignored.
- CTest reports **`100% tests passed, 0 tests failed out of 7`**.

---

## 5. Run the game

### Headless (works everywhere, no GPU needed)

```powershell
.\build\apps\sandbox\ea-sandbox.exe --headless --seed 42 --frames 600 --out trace.csv
```

Expected output: `trace_digest=... frames=600 scene=rope` and exit code 0. Same seed
always produces the same digest — that's the determinism guarantee (Article 5).

### Interactive window

```powershell
.\build\apps\sandbox\ea-sandbox.exe
```

A 1280×720 window opens showing the verlet rope scene. Press `1`–`4` to switch
scenes, `Space` to pause, `Esc` to quit. Full controls are in
[apps/sandbox/README.md](../apps/sandbox/README.md).

> **On a VM / RDP session this will likely fail** with
> `GLFW: Error: 65542 ... WGL: The driver does not appear to support OpenGL`.
> That is not a build problem — the VM's virtual display adapter has no hardware
> OpenGL driver. Fix it in step 7.

### Screenshot capture (headless-ish visual check)

```powershell
.\build\apps\sandbox\ea-sandbox.exe --seed 42 --scene rope --screenshot proof.png --warmup 120
```

> Use a **relative** output path — raylib prefixes the working directory onto the
> path you give it, so absolute paths fail to save.

---

## 6. Install Spec-Kit (Specify CLI) locally

> **Already wired into this repo:** the `.specify/` folder (templates, scripts,
> constitution) and the `.github/prompts/speckit.*.prompt.md` files are committed,
> so the `/speckit.*` slash commands work in VS Code Copilot Chat out of the box —
> no install needed to follow the workshop. Install the CLI below to verify your
> environment (`specify check`) and to scaffold Spec-Kit into **your own** projects.

Requires Git (from step 1) and Python 3.11+ — if no suitable Python is on the
machine, uv downloads a managed one automatically during the install below.

### Install uv, then the Specify CLI

```powershell
# 1. Install uv (Python package/tool manager)
winget install --id astral-sh.uv
#    ...or, without winget:
#    powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"

# 2. Open a NEW PowerShell so uv is on PATH, then install the CLI.
#    Pin a release tag for a reproducible workshop setup — check
#    https://github.com/github/spec-kit/releases for the latest (v0.12.4 as of writing):
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v0.12.4
```

> **Why pin?** This repo's `.specify/` templates and prompts were generated by a
> specific Spec-Kit release. Pinning the same tag keeps everyone in the workshop on
> identical tooling. Drop the `@v0.12.4` suffix to install the latest instead, and
> upgrade later with `specify self upgrade`.

### Verify

```powershell
specify version
specify check
```

`specify check` confirms the CLI can find Git and your AI coding agents (VS Code
with Copilot counts). Both commands succeeding means Spec-Kit is installed.

### Optional — scaffold Spec-Kit into your own project

```powershell
cd C:\path\to\your-project
specify init --here --integration copilot
```

> **Do not run `specify init` inside this repo** — it regenerates `.specify/` and
> the `.github/prompts/speckit.*` files and can overwrite the workshop's customized
> versions. Use it only in your own projects.

---

## 7. VM fix — software OpenGL (Mesa llvmpipe)

If the interactive window fails with the WGL error above, drop Mesa's
software-rendered `opengl32.dll` next to the exe. Verified steps:

```powershell
# 1. Download Mesa3D for Windows (MSVC release) and the standalone 7-Zip extractor
Invoke-WebRequest "https://github.com/pal1000/mesa-dist-win/releases/download/26.1.3/mesa3d-26.1.3-release-msvc.7z" -OutFile "$env:TEMP\mesa3d.7z"
Invoke-WebRequest "https://www.7-zip.org/a/7zr.exe" -OutFile "$env:TEMP\7zr.exe"

# 2. (Recommended) Verify the download against the GitHub release SHA-256 digest
(Get-FileHash "$env:TEMP\mesa3d.7z" -Algorithm SHA256).Hash.ToLower()
# expect: 6dd431f4620cea73970b13e3ffa94f721f2a3924306b8a4283c97648cdb6eb9c

# 3. Extract and copy the x64 DLLs next to the game executable
& "$env:TEMP\7zr.exe" x "$env:TEMP\mesa3d.7z" -o"$env:TEMP\mesa3d" -y
Copy-Item "$env:TEMP\mesa3d\x64\*.dll" ".\build\apps\sandbox\" -Force
```

Run the game again — the log should now show:

```text
INFO: GL: OpenGL device information:
INFO:     > Vendor:   Mesa
INFO:     > Renderer: llvmpipe (LLVM ..., 256 bits)
```

Expect ~15–20 fps (CPU rendering). That's normal and plenty for the workshop demo.
Newer Mesa releases work too — check
[pal1000/mesa-dist-win releases](https://github.com/pal1000/mesa-dist-win/releases)
and verify the SHA-256 digest shown on the release page.

> The DLLs live only in `build\apps\sandbox\` (untracked build output). Deleting the
> build folder removes them — just re-run the copy after a clean rebuild.

---

## 8. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `cmake` / `ninja` not recognized | You're not in a Developer PowerShell. Redo step 2. |
| `Could not find toolchain file: /scripts/buildsystems/vcpkg.cmake` | `VCPKG_ROOT` is unset in this shell. Redo step 3. |
| vcpkg error about `builtin-baseline` commit | Standalone vcpkg was cloned shallow. Run `git fetch --unshallow` inside `$env:VCPKG_ROOT`, or use the VS-bundled vcpkg. |
| Linker errors about machine type x86 vs x64 | Dev shell launched without `-Arch amd64`. Open a new shell with the flag. |
| `WGL: The driver does not appear to support OpenGL` + assertion in `window.c` | No hardware OpenGL (VM/RDP). Apply step 7. |
| `MESA: error: ZINK: failed to load vulkan-1.dll` in the log | Harmless — Mesa probes Vulkan first, then falls back to llvmpipe. |
| Hundreds of `D9025 overriding '/EHs' with '/EHs-'` warnings | Expected. Exceptions are disabled by design (constitution Article 2). |
| Screenshot "Failed to export image" | Absolute output path. Use a relative path (see step 5). |
| CI runner / no raylib wanted | Configure with `cmake --preset default-debug -DENGINE_DEMO_BUILD_SANDBOX=OFF`. Tests still run. |
| Stale/broken `build\` after toolchain change | Delete the `build` folder and reconfigure from step 4. |
| `specify` / `uv` not recognized | Open a new PowerShell after installing uv (step 6), or re-run the `uv tool install` line. |

---

## 9. Quick reference — full clean setup, one block

```powershell
# In a fresh PowerShell:
$vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
& "$vs\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -SkipAutomaticLocation
$env:VCPKG_ROOT = "$vs\VC\vcpkg"
cd C:\code\speckit-workshop-demo

cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure

.\build\apps\sandbox\ea-sandbox.exe --headless --seed 42 --frames 600 --out trace.csv
.\build\apps\sandbox\ea-sandbox.exe    # interactive (apply step 7 first on a VM)

# Optional — Spec-Kit CLI (see step 6; needs a new shell after installing uv)
winget install --id astral-sh.uv
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v0.12.4
specify check
```

You're ready for the workshop — head back to
[docs/speckit-workshop-training.md](speckit-workshop-training.md).
