# version-cmake — CMakeLists.txt Version Extraction

Demonstrates extracting version from `CMakeLists.txt` using the
standard `project(... VERSION x.y.z)` convention.

## What it shows

- `file = "CMakeLists.txt"` — version source is not limited to C headers
- Single-line template (no multiline `"""` needed)
- Inline placeholders: `project(MyFirmware VERSION ${MAJOR}.${MINOR}.${PATCH})`

## Run

```bash
baker build
# Output: release/my_firmware_v3.1.4.bin
```
