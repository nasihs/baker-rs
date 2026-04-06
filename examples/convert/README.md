# convert — Format Conversion

Demonstrates converting a HEX file to BIN and SREC formats.
No version file needed — shows that `[env.version]` is optional.

## What it shows

- `type = "convert"` targets with different `output_format` values
- `"bin"` | `"srec"` output formats
- `${PROJECT}` and `${TIME.YYYYMMDD}` built-in variables
- `[groups]` to build multiple targets in one command

## Run

```bash
baker build              # runs both conversions (default group: all)
baker build to_bin       # BIN only
baker build to_srec      # SREC only
# Output: release/my_firmware_<date>.bin  and  .srec
```
