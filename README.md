# Baker

> **B**uild **A**utomation **K**it for **E**mbedded **R**elease

A command-line tool for automating embedded firmware post-build packaging, written in Rust.

## Features

- **Configuration-driven**: Define all build variants in a single TOML file
- **Version extraction**: Auto-extract version from C headers with flexible field mapping
- **Template variables**: Dynamic output naming with version, date/time, and project variables
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
    source = "header"
    file = "version.h"
    string = "VERSION_STR"       # Full version string (e.g., "v1.2.3-beta")
    build = "BUILD_NUMBER"       # Optional build number

    [bootloaders.default]
    file = "build/bt.hex"
    base_addr = 0x0800_0000
    app_offset = 0x8000

    # custom header definiton
    [headers.custom_fpk]
    suffix = "fpk"
    def = """
    @endian = little;

    struct header @packed {
        magic:          [u8; 4] = @bytes("fpk");
        config:         u32 = 0;
        old_version:    [u8; 16];
        new_version:    [u8; 16] = [1,2,3];
        watermark:      [u8; 16] = @bytes("DELBIN_DEMO");
        partition:      [u8; 16] = @bytes("app");
        img_size:       u32 = @sizeof(image);
        packed_size:    u32 = @sizeof(image);
        timestamp:      u32 = 0x696f03eb;
        img_crc32:      u32 = @crc32(image);
        packed_crc32:   u32 = @crc32(image);
        header_crc32:   u32 = @crc32(@self[..header_crc32]);
        _padding:       [u8; 1024 - @offsetof(_padding)];
    }
    """

    output_name = "{PROJECT}_v{VERSION}_build{BUILD}_{DATE}"  # Template variables
    [targets.merge_test]
    type = "merge"
    description = "Merge bootloader and app"
    bootloader = "default"
    app_file = "build/app.hex"
    output_format = "hex"  # use bin if not specified

    [targets.pack_from_bin]
    type = "pack"
    description = "Pack with custom header"
    header = "custom_fpk"  # use custom header
    app_file = "build/app.bin"
    app_offset = 0x8000  # needed if app is binary
    output_name = "pack_from_bin"

    [targets.convert_to_bin]
    type = "convert"
    description = "Convert hex to bin"
    input_file = "build/app.hex"
    output_format = "bin"
    output_name = "converted_bin"

    [targets.convert_to_srec]
    type = "convert"
    description = "Convert to S-record format"
    input_file = "build/app.hex"
    output_format = "srec"
    output_name = "converted_srec"

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

Baker supports automatic version extraction from C/C++ header files and provides template variables for dynamic output naming.

### Quick Example

```c
// version.h
#define VERSION_STR "v1.2.3-beta.2+20260125"
#define BUILD_NUMBER 100
```

```toml
# baker.toml
[env.version]
source = "header"
file = "version.h"
string = "VERSION_STR"
build = "BUILD_NUMBER"

[targets.release]
output_name = "{PROJECT}_v{VERSION}_build{BUILD}_{DATE}"
# Output: myapp_v1.2.3_build100_20260125.hex
```

### Available Template Variables

- **Version**: `{MAJOR}`, `{MINOR}`, `{PATCH}`, `{VERSION}`, `{VERSION_FULL}`, `{BUILD}`
- **DateTime**: `{DATE}`, `{TIME}`, `{DATETIME}`, `{TIMESTAMP}`
- **Project**: `{PROJECT}`, `{TARGET}`

For detailed documentation on version extraction and template variables, see [docs/version-templating.md](docs/version-templating.md).


## Lisence

