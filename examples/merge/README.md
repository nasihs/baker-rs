# merge — Bootloader + App Merge

Demonstrates merging a bootloader and application HEX into a single firmware image.
Version is extracted from `version.h` using separate integer macros.

## What it shows

- `type = "merge"` target
- `[bootloaders]` config with `base_addr` and `app_offset`
- `[env.version]` with C integer macro template
- `${VER.*}` and `${TIME.YYYYMMDD}` in `output_name`

## Run

```bash
baker build
# Output: release/my_firmware_v1.0.0_build1_<date>.hex
```
