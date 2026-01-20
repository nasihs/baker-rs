# Baker

> **B**uild **A**utomation **K**it for **E**mbedded **R**elease

A command-line tool for automating embedded firmware post-build packaging, written in Rust.

## Features

- **Configuration-driven**: Define all build variants in a single TOML file
- **Version extraction**: Auto-extract version from C headers, CMake, Git tags
- **Firmware merging**: Combine bootloader and application with offset control
- **OTA packaging**: Generate update packages with custom headers
- **CI/CD friendly**: Single binary, no runtime dependencies

## Quick Start
1. Create a baker.toml in your project:

    ```toml
    [project]
    name = "baker_demo"
    default = "merge_test"

    [env.output]
    dir = "release"

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



## Lisence

