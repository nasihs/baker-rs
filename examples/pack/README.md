# pack — OTA Packaging with Custom Header

Demonstrates packaging a firmware binary with a custom OTA header defined
in the delbin DSL. Version fields and build timestamp are embedded directly
into the header struct.

## What it shows

- `type = "pack"` target
- `[headers]` with a custom delbin DSL struct
- `${VER_MAJOR}`, `${VER_BUILD}` variables embedded in the header `def`
- `${TIME_EPOCH32}` for a u32 Unix timestamp field
- Note: inside DSL `def`, use underscore names (`${VER_MAJOR}`); in `output_name`, use dotted names (`${VER.MAJOR}`)

## Run

```bash
baker build
# Output: release/my_firmware_v2.1.0_build42.fpk
```
