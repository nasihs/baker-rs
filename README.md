# Baker - Build Automation Kit for Embedded Release

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Edition](https://img.shields.io/badge/rust%20edition-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)

A command-line tool for automating embedded firmware post-build packaging, written in Rust.

## Features

- **Configuration-driven**: Define all build variants in a single TOML file
- **Version extraction**: Extract version from any text file (C headers, CMakeLists.txt, Python, …) using a line-pattern template
- **Template variables**: Dynamic output naming with version, date/time, and project variables using `${VAR}` syntax
- **Firmware merging**: Combine bootloader and application with offset control
- **Format conversion**: Convert between HEX, BIN, and SREC formats
- **OTA packaging**: Generate update packages with custom binary headers (DSL-based)
- **CI/CD friendly**: Single binary, no runtime dependencies

## Quick Start

1. Create a `baker.toml` in your project root (all file paths are relative to `baker.toml`):

   ```toml
   [project]
   name = "baker_demo"
   default = "merge_test"   # target or group to run when no argument is given

   [env.output]
   dir = "release"          # default output directory for all targets

   [env.version]
   source = "file"          # currently only "file" is supported
   file = "version.h"
   template = """
   #define VERSION_MAJOR  ${MAJOR}
   #define VERSION_MINOR  ${MINOR}
   #define VERSION_PATCH  ${PATCH}
   #define BUILD_NUMBER   ${BUILD}
   """

   [bootloaders.default]
   file = "build/bt.hex"
   base_addr = 0x0800_0000
   app_offset = 0x8000

   [headers.custom_fpk]
   suffix = "fpk"
   def = """
   @endian = little;

   struct header @packed {
       magic:          [u8; 4] = @bytes("fpk");
       config:         u32 = 0;
       old_version:    [u8; 16];
       new_version:    [u8; 16] = [${VER.MAJOR}, ${VER.MINOR}, ${VER.PATCH}];
       watermark:      [u8; 16] = @bytes("baker");
       partition:      [u8; 16] = @bytes("app");
       img_size:       u32 = @sizeof(image);
       packed_size:    u32 = @sizeof(image);
       timestamp:      u32 = ${TIME.EPOCH32};
       img_crc32:      u32 = @crc32(image);
       packed_crc32:   u32 = @crc32(image);
       header_crc32:   u32 = @crc32(@self[..header_crc32]);
       _padding:       [u8; 1024 - @offsetof(_padding)];
   }
   """

   [targets.merge_test]
   type = "merge"           # "merge" | "pack" | "convert"
   description = "Merge bootloader and app"
   bootloader = "default"
   app_file = "build/app.hex"
   output_format = "hex"   # "bin" (default) | "hex" | "srec"
   output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}_build${VER.BUILD}_${TIME.YYYYMMDD}"
   output_dir = "release/merged"  # overrides [env.output].dir for this target

   [targets.pack_from_bin]
   type = "pack"
   description = "Pack with custom header"
   header = "custom_fpk"
   app_file = "build/app.bin"
   app_offset = 0x8000     # required when app_file is a raw binary
   output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}"

   [targets.convert_to_bin]
   type = "convert"
   description = "Convert hex to bin"
   input_file = "build/app.hex"
   output_format = "bin"
   output_name = "converted_bin"

   [groups]
   factory = ["merge_test", "pack_from_bin"]   # run multiple targets with: baker build factory
   ```

1. Run Baker
   ```bash
   baker build              # Build default target/group
   baker build factory      # Build specific target/group
   baker list               # List all targets/groups
   ```

## Version Extraction & Template Variables

Baker can extract version fields from any text file using a **line-pattern template**, and exposes them as `${VER.*}` variables for use in `output_name` and the delbin header DSL.

### Quick Example

```c
// version.h
#define VERSION_MAJOR 1
#define VERSION_MINOR 2
#define VERSION_PATCH 3
#define BUILD_NUMBER  100
```

```toml
# baker.toml
[env.version]
source = "file"
file = "version.h"
template = """
#define VERSION_MAJOR  ${MAJOR}
#define VERSION_MINOR  ${MINOR}
#define VERSION_PATCH  ${PATCH}
#define BUILD_NUMBER   ${BUILD}
"""

[targets.release]
output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}_build${VER.BUILD}_${TIME.YYYYMMDD}"
# Output: myapp_v1.2.3_build100_20260125.hex
```

### Available Variables

All `${VAR}` placeholders can be used in `output_name` and the delbin header DSL.

**Version variables** — defined by your `[env.version].template`:

| Variable | Description |
|---|---|
| `${VER.MAJOR}`, `${VER.MINOR}`, `${VER.PATCH}` | Captured from the version file |
| `${VER.<ANY>}` | Any name defined in your template |

**Date / Time** — generated at build time:

| Variable | Example | Notes |
|---|---|---|
| `${TIME.YYYYMMDD}` | `20260406` | 4-digit year date |
| `${TIME.YYMMDD}` | `260406` | 2-digit year date |
| `${TIME.HHMMSS}` | `143052` | Time of day |
| `${TIME.YYMMDDHHMM}` | `2604061430` | Date + hour + minute (last MM = minutes) |
| `${TIME.DATETIME}` | `20260406_143052` | Date and time combined |
| `${TIME.EPOCH}` | `1743901375` | Unix timestamp, u64 |
| `${TIME.EPOCH32}` | `1743901375` | Unix timestamp, u32 — use in delbin `u32` fields |

**Git** — available only inside a git repository:

| Variable | Example | Notes |
|---|---|---|
| `${GIT.HASH}` | `a1b2c3d` | Short commit hash; error if not in a git repo |

**Project**:

| Variable | Example | Notes |
|---|---|---|
| `${PROJECT}` | `baker_demo` | From `[project].name` |
| `${TARGET}` | `merge_test` | Current target name |

For full documentation on the template syntax and more examples, see [docs/version-templating.md](docs/version-templating.md).

## Contributing

Contributions are welcome! Please open an issue to discuss your idea before submitting a pull request. When submitting a PR:

- Follow the existing code style
- Add or update tests for any changed behavior
- Use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages

## License

This project is licensed under the [MIT License](LICENSE).
