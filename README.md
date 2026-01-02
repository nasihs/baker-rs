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
    name = "my-firmware"
    default = "factory"
    
    [version]
    source = "header"
    file = "src/version.h"
    
    [targets.factory]
    type = "merge"
    app = "build/app.hex"
    bootloader = "build/bootloader.hex"
    app_offset = 0x8000
    
    [targets.ota]
    type = "ota"
    input = "build/app.bin"
    header = "none"
    output_name = "v{version}"
    ```
1. Run Baker
   ```bash
   baker build              # Build default target
   baker build factory ota  # Build specific targets
   baker list               # List all targets
   ```



## Lisence

