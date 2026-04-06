# custom-fields — Non-Version Field Extraction

Demonstrates that the template extractor is not limited to version numbers.
Any named field from any text file can be captured and used in `output_name`.

## What it shows

- Extracting `BOARD_ID` (hex value → u32) and `HW_REV` alongside version
- `${VER.BOARD}` and `${VER.HW_REV}` in `output_name`
- Hex literal `0x42` is automatically parsed as `u32` (value 66)

## Run

```bash
baker build
# Output: release/my_firmware_board66_hwrev3_v1.0.bin
```
