# Baker Examples

Each subdirectory is a focused, runnable example. Run `baker build` from inside any example directory.

| Example | What it shows |
|---------|--------------|
| [`complete/`](complete/) | All features combined (merge + pack + convert + groups) |
| [`merge/`](merge/) | Merge bootloader + app; version from C integer macros |
| [`pack/`](pack/) | OTA packaging with custom binary header DSL |
| [`convert/`](convert/) | Convert HEX to BIN and SREC; no version extraction |
| [`version-c-string/`](version-c-string/) | Version from a single C string macro `"v1.2.3"` |
| [`version-cmake/`](version-cmake/) | Version from `CMakeLists.txt` `project(... VERSION ...)` |
| [`custom-fields/`](custom-fields/) | Extract non-version fields (BOARD_ID, HW_REV) into output name |

## Prerequisites

Build baker from the repo root, then add it to your PATH:

```bash
cargo build --release
# Add target/release/ to PATH, or invoke with full path
```
