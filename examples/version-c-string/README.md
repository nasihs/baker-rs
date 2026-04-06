# version-c-string — Single String Macro

Demonstrates extracting version from a C header that stores the full
version in a single string macro, e.g. `#define FW_VERSION "v2.5.10"`.

Multiple `${VAR}` placeholders on one template line each capture a
distinct component from the same line in the file.

## What it shows

- Template line: `#define FW_VERSION "v${MAJOR}.${MINOR}.${PATCH}"`
- Contrast with the `merge/` example which uses separate integer macros

## Run

```bash
baker build
# Output: release/my_firmware_v2.5.10_build42.bin
```
