# Version Extraction and Template Variables

Baker can extract version information from any text file using a line-pattern template, and makes those values available as `${VER.*}` variables throughout `baker.toml`.

## Table of Contents

- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [Template Syntax](#template-syntax)
- [Type Inference](#type-inference)
- [Template Variables Reference](#template-variables-reference)
- [Examples](#examples)
- [Error Handling](#error-handling)

## How It Works

Each line in the `template` string that contains one or more `${VAR}` placeholders is compiled into a regex pattern. The surrounding text on that line is matched literally against every line of the target file. When a match is found, the captured value is stored as `VER.<VAR>` in baker's environment. Lines without placeholders are ignored.

All `${VER.*}` variables can then be used in:
- `output_name` fields of any target
- The delbin binary header DSL (`def` field of `[headers.*]`)

## Configuration

```toml
[env.version]
source = "file"       # Supported: "file"
file = "version.h"    # Any text file, relative to baker.toml
template = """
#define VERSION_MAJOR  ${MAJOR}
#define VERSION_MINOR  ${MINOR}
#define VERSION_PATCH  ${PATCH}
#define BUILD_NUMBER   ${BUILD}
"""
```

`source = "file"` works with any text-based format — C headers, `CMakeLists.txt`, Python source, INI files, etc.

## Template Syntax

### Variable Placeholder

`${VAR}` captures the value at that position. The placeholder name must match `[A-Z_][A-Z0-9_]*`. The captured value is registered as `VER.<VAR>` in baker's environment.

```toml
# ${MAJOR} → captured as VER.MAJOR
template = '#define VERSION_MAJOR  ${MAJOR}'
```

### Multiple Placeholders on One Line

A single template line can contain several placeholders. Each captures a distinct value from the same line in the file:

```toml
template = '#define VERSION_STR  "v${MAJOR}.${MINOR}.${PATCH}"'
# Matches: #define VERSION_STR  "v1.2.3"
# → VER.MAJOR="1", VER.MINOR="2", VER.PATCH="3"
```

### Whitespace Flexibility

Minor differences in spacing between the template line and the file line are tolerated — a single space in the template matches one or more whitespace characters in the file.

### Comment Tolerance

Trailing C-style line comments (`// ...`) are ignored when matching.

### Multiline Template (recommended)

```toml
template = """
#define VERSION_MAJOR  ${MAJOR}
#define VERSION_MINOR  ${MINOR}
#define VERSION_PATCH  ${PATCH}
#define BUILD_NUMBER   ${BUILD}
"""
```

All template lines that contain `${VAR}` must each match at least one line in the target file, or baker reports an error.

## Type Inference

Captured values are automatically typed:

| Captured string | Stored type | Example |
|---|---|---|
| Decimal integer | `u32` | `3` → `3` |
| Hex integer (`0x`/`0X`) | `u32` | `0x0A` → `10` |
| Binary integer (`0b`/`0B`) | `u32` | `0b11` → `3` |
| Anything else | `String` | `"beta"` → `"beta"` |

Quoted strings (e.g. `"1.2.3"`) have their surrounding quotes stripped before type inference.

## Template Variables Reference

### Version Variables (`VER.*`)

These are defined entirely by your `template`. The names shown below are conventional but you can use any uppercase identifier:

| Template placeholder | Baker variable | Typical use |
|---|---|---|
| `${MAJOR}` | `${VER.MAJOR}` | Major version number |
| `${MINOR}` | `${VER.MINOR}` | Minor version number |
| `${PATCH}` | `${VER.PATCH}` | Patch version number |
| `${BUILD}` | `${VER.BUILD}` | Build/revision number |
| `${ANY_NAME}` | `${VER.ANY_NAME}` | Any custom field |

### Date/Time Variables

Always available, generated at build time:

| Variable | Description | Example | Format |
|---|---|---|---|
| `${TIME.YYYYMMDD}` | Current date (4-digit year) | `20260405` | `YYYYmmdd` |
| `${TIME.YYMMDD}` | Current date (2-digit year) | `260405` | `yymmdd` |
| `${TIME.HHMMSS}` | Current time | `143052` | `HHmmss` |
| `${TIME.YYMMDDHHMM}` | Date + hour + minute | `2604051430` | `yymmddHHMM` |
| `${TIME.DATETIME}` | Date and time combined | `20260405_143052` | `YYYYmmdd_HHmmss` |
| `${TIME.EPOCH}` | Unix timestamp | `1743901375` | Seconds since epoch (u64) |
| `${TIME.EPOCH32}` | Unix timestamp (u32) | `1743901375` | Same value, u32 type — use this in delbin `u32` fields; safe until year 2106 |

### Project Variables

Always available:

| Variable | Description | Example |
|---|---|---|
| `${PROJECT}` | Project name | `myapp` |
| `${TARGET}` | Name of the target being built | `release_build` |

### Git Variables

Available only when baker is run inside a git repository:

| Variable | Description | Example | Notes |
|---|---|---|---|
| `${GIT.HASH}` | Short commit hash | `a1b2c3d` | Error if not in a git repo |

## Examples

### Integer Macros (C header)

```c
// version.h
#define VERSION_MAJOR 1
#define VERSION_MINOR 2
#define VERSION_PATCH 3
#define BUILD_NUMBER  100
```

```toml
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
# → myapp_v1.2.3_build100_20260405.hex
```

### Inline String Parsing (C header)

```c
// version.h
#define VERSION_STR "v1.2.3"
#define BUILD_NUMBER 100
```

```toml
[env.version]
source = "file"
file = "version.h"
template = """
#define VERSION_STR "v${MAJOR}.${MINOR}.${PATCH}"
#define BUILD_NUMBER ${BUILD}
"""

[targets.release]
output_name = "${PROJECT}_v${VER.MAJOR}.${VER.MINOR}.${VER.PATCH}_build${VER.BUILD}_${TIME.YYYYMMDD}"
```

### Hex Values

```c
#define VER_MAJOR 0x01
#define VER_MINOR 0x0A
```

```toml
template = """
#define VER_MAJOR ${MAJOR}
#define VER_MINOR ${MINOR}
#define VER_PATCH ${PATCH}
"""
# VER.MAJOR = 1 (u32), VER.MINOR = 10 (u32)
```

### CMakeLists.txt

```cmake
project(MyFirmware VERSION 1.2.3)
```

```toml
[env.version]
source = "file"
file = "CMakeLists.txt"
template = 'project(MyFirmware VERSION ${MAJOR}.${MINOR}.${PATCH})'
```

### Python Package

```python
# src/__init__.py
__version__ = "1.2.3"
```

```toml
[env.version]
source = "file"
file = "src/__init__.py"
template = '__version__ = "${MAJOR}.${MINOR}.${PATCH}"'
```

### Version Variables in OTA Header DSL

```toml
[headers.fpk]
suffix = "fpk"
def = """
@endian = little;
struct header @packed {
    new_version: [u8; 16] = [${VER.MAJOR}, ${VER.MINOR}, ${VER.PATCH}];
    timestamp:   u32      = ${TIME.EPOCH32};
    img_size:    u32      = @sizeof(image);
    img_crc32:   u32      = @crc32(image);
}
"""
```

## Error Handling

### Template Pattern Not Matched

Every template line with a `${VAR}` placeholder must match at least one line in the target file.

```
Error: template pattern not matched in file: '#define VERSION_MAJOR  ${MAJOR}'
```

**Fix**: Check that the template line (spacing, macro name) matches the actual content of the file.

### Undefined Variable in output_name

```
Error: missing template variable 'VER.BUILD'
```

**Fix**: Add `${BUILD}` to your template, or remove `${VER.BUILD}` from `output_name`.

### File Not Found

```
Error: version file not found: version.h
```

**Fix**: Verify the `file` path is correct and relative to `baker.toml`.

