# Baker

> **B**uild **A**utomation **K**it for **E**mbedded **R**elease

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

1. Create a baker.toml in your project:

   ```toml
   [project]
   name = "baker_demo"
   default = "merge_test"

   [env.output]
   dir = "release"

   [env.version]
   source = "file"
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
       timestamp:      u32 = ${UNIX_TIMESTAMP};
       img_crc32:      u32 = @crc32(image);
       packed_crc32:   u32 = @crc32(image);
       header_crc32:   u32 = @crc32(@self[..header_crc32]);
       _padding:       [u8; 1024 - @offsetof(_padding)];
   }
   """

   [targets.merge_test]
   type = "merge"
   description = "Merge bootloader and app"
   bootloader = "default"
   app_file = "build/app.hex"
   output_format = "hex"
   output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}_build${VER.BUILD}_${DATE}"
   output_dir = "release/merged"  # will overrite [env.output_dir]

   [targets.pack_from_bin]
   type = "pack"
   description = "Pack with custom header"
   header = "custom_fpk"
   app_file = "build/app.bin"
   app_offset = 0x8000
   output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}"

   [targets.convert_to_bin]
   type = "convert"
   description = "Convert hex to bin"
   input_file = "build/app.hex"
   output_format = "bin"
   output_name = "converted_bin"

   [groups]
   factory = ["merge_test", "pack_from_bin"]
   ```

1. Run Baker
   ```bash
   baker build              # Build default target
   baker build factory      # Build specific targets
   baker list               # List all targets
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
output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}_build${VER.BUILD}_${DATE}"
# Output: myapp_v1.2.3_build100_20260125.hex
```

### Available Template Variables

**Extracted version variables** (defined by your `template`):

- `${VER.MAJOR}`, `${VER.MINOR}`, `${VER.PATCH}`, `${VER.BUILD}` — or any name you define in the template

**Always available**:

- **DateTime**: `${DATE}`, `${TIME}`, `${DATETIME}`, `${TIMESTAMP}`, `${UNIX_TIMESTAMP}`
- **Project**: `${PROJECT}`, `${TARGET}`

For full documentation on the template syntax and more examples, see [docs/version-templating.md](docs/version-templating.md).

## Lisence
