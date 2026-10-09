# Heterogeneous reference-array initializer fixture

This fixture has independent `factory` (positive type-proof) and `direct` (fresh-construction/array-store composition control) archives. Each archive contains its complete six-class hierarchy and has no external classpath inputs. The earlier combined draft and successful dual-JDK compile record are retained under `plan-v1/`; the operative scope is `source-plan.md`.

## Build and execution

Saved runner: `build_fixture_v2.py` (SHA-256 `99210a3d2747e126a3c09dd39883e74fe5b3a22970991ea5cf896e98d5131ccc`); manifest: `build-manifest-v2.json` (SHA-256 `6d4dd2525cb3747080fc0fbc9f97bea043c0f5193c381240a9c8244f07585040`). The runner records exact argv, cwd, exit codes, stdout/stderr files and hashes, JDK executable hashes, all source hashes, and six class hashes per family/leg. Both compiles use explicit empty `-classpath` and `-sourcepath`; original runs use `-Xverify:all`.

| family | compiler leg | javac version | java version | compiler flags | javac | runtime | stdout SHA-256 | stderr SHA-256 |
|---|---|---|---|---|---:|---:|---|---|
| factory | javac8 | `javac 1.8.0_432` | `openjdk version "1.8.0_432" OpenJDK Runtime Environment Corretto-8.432.06.1 (build 1.8.0_432-b06) OpenJDK 64-Bit Server VM Corretto-8.432.06.1 (build 25.432-b06, mixed mode)` | `-source 8 -target 8 -g:none` | 0 | 0 | `407b8b80ea3144d8ef934c961289c215a7e9a9d304c0b43032e46d8aafc78d5b` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| factory | javac23 | `javac 23.0.1` | `openjdk version "23.0.1" 2024-10-15 OpenJDK Runtime Environment (build 23.0.1+11-39) OpenJDK 64-Bit Server VM (build 23.0.1+11-39, mixed mode, sharing)` | `--release 8 -g:none` | 0 | 0 | `407b8b80ea3144d8ef934c961289c215a7e9a9d304c0b43032e46d8aafc78d5b` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| direct | javac8 | `javac 1.8.0_432` | `openjdk version "1.8.0_432" OpenJDK Runtime Environment Corretto-8.432.06.1 (build 1.8.0_432-b06) OpenJDK 64-Bit Server VM Corretto-8.432.06.1 (build 25.432-b06, mixed mode)` | `-source 8 -target 8 -g:none` | 0 | 0 | `407b8b80ea3144d8ef934c961289c215a7e9a9d304c0b43032e46d8aafc78d5b` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| direct | javac23 | `javac 23.0.1` | `openjdk version "23.0.1" 2024-10-15 OpenJDK Runtime Environment (build 23.0.1+11-39) OpenJDK 64-Bit Server VM (build 23.0.1+11-39, mixed mode, sharing)` | `--release 8 -g:none` | 0 | 0 | `407b8b80ea3144d8ef934c961289c215a7e9a9d304c0b43032e46d8aafc78d5b` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Both families produce identical deterministic stdout on the two original JDK legs; stderr is empty. The source families differ in the tested array element form: typed factory calls (`factory`) versus direct constructor expressions at the array element (`direct`). The direct family is a valid Java source/runtime oracle, not a negative assignability case. Its integration assertion preserves the current `new@1` shape refusal; it is not counted as a type-proof result.

## Frozen inputs

Every source and class SHA-256, command and raw stream is listed in `build-manifest-v2.json`. Each family/leg has `Main.class`, `Base.class`, `Mid.class`, `DerivedA.class`, `DerivedB.class`, and `LocalInterface.class`, compiled together. Tests place those exact class bytes in one archive so the selected snapshot can answer same-archive hierarchy reads.

## Source plans

`source-plan.md` defines the current split-family scope. `source-plan-v1.md` and `plan-v1/` preserve the first combined draft and its compile/run artifacts for history; that draft is not the acceptance unit.
