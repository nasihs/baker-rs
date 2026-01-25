# Version Extraction and Template Variables

Baker provides a powerful version extraction system that can parse version information from various sources and use them in dynamic output file naming through template variables.

## Table of Contents

- [Version Extraction](#version-extraction)
  - [Configuration](#configuration)
  - [Header File Extraction](#header-file-extraction)
  - [Flexible Field Mapping](#flexible-field-mapping)
- [Template Variables](#template-variables)
  - [Version Variables](#version-variables)
  - [DateTime Variables](#datetime-variables)
  - [Project Variables](#project-variables)
- [Examples](#examples)
- [Error Handling](#error-handling)

## Version Extraction

### Configuration

Version extraction is configured in the `[env.version]` section of your `baker.toml`:

```toml
[env.version]
source = "header"           # Currently supported: "header"
file = "version.h"          # Path to version file (relative to baker.toml)

# Option 1: Extract from full version string
string = "VERSION_STR"      # Macro containing version string

# Option 2: Extract from individual fields
major = "VERSION_MAJOR"
minor = "VERSION_MINOR"
patch = "VERSION_PATCH"

# Optional fields (for both options)
build = "BUILD_NUMBER"
pre_release = "PRE_RELEASE"
```

**Note**: You must provide either `string` OR all of (`major`, `minor`, `patch`). The `build` and `pre_release` fields are always optional.

### Header File Extraction

Baker can extract version information from C/C++ header files using `#define` macros.

#### Strategy 1: String-based Version

Extract from a single version string macro:

```c
// version.h
#define VERSION_STR "1.2.3-beta.2+20260125"
#define BUILD_NUMBER 100
```

```toml
[env.version]
source = "header"
file = "version.h"
string = "VERSION_STR"      # Parses semantic version
build = "BUILD_NUMBER"      # Supplements build info
```

**Supported version string formats:**
- Basic: `1.2.3`
- With v prefix: `v1.2.3` or `V1.2.3`
- With pre-release: `1.2.3-beta.2`, `1.2.3-rc.1`
- With build metadata: `1.2.3+20260125`
- Full semantic version: `1.2.3-beta.2+20260125`

#### Strategy 2: Field-based Version

Extract from individual version field macros:

```c
// version.h
#define VERSION_MAJOR 1
#define VERSION_MINOR 2
#define VERSION_PATCH 3
#define BUILD_NUMBER 100
```

```toml
[env.version]
source = "header"
file = "version.h"
major = "VERSION_MAJOR"
minor = "VERSION_MINOR"
patch = "VERSION_PATCH"
build = "BUILD_NUMBER"
```

**Supported number formats:**
- Decimal: `100`
- Hexadecimal: `0x64` or `0X64`
- Binary: `0b01100100` or `0B01100100`

### Flexible Field Mapping

All field names in `[env.version]` are macro names from your header file. You can use any naming convention:

```toml
# Example 1: Uppercase with underscores
major = "VERSION_MAJOR"
minor = "VERSION_MINOR"
patch = "VERSION_PATCH"

# Example 2: Prefix notation
major = "VER_MAJ"
minor = "VER_MIN"
patch = "VER_PAT"

# Example 3: Camel case
major = "VersionMajor"
minor = "VersionMinor"
patch = "VersionPatch"
```

## Template Variables

Template variables use the `{VARIABLE_NAME}` syntax and can be used in any `output_name` field.

### Version Variables

These variables are available when `[env.version]` is configured:

| Variable | Description | Example Value | Notes |
|----------|-------------|---------------|-------|
| `{MAJOR}` | Major version number | `1` | Always available |
| `{MINOR}` | Minor version number | `2` | Always available |
| `{PATCH}` | Patch version number | `3` | Always available |
| `{VERSION}` | Basic version string | `1.2.3` | Format: `major.minor.patch` |
| `{VERSION_FULL}` | Full semantic version | `1.2.3-beta.2+20260125` | Includes pre-release and metadata |
| `{BUILD}` | Build number | `100` | Only if configured |
| `{PRE_RELEASE}` | Pre-release identifier | `beta.2` | Only if present in string |
| `{BUILD_METADATA}` | Build metadata | `20260125` | Only if present in string |

### DateTime Variables

These variables are always available and generated at build time:

| Variable | Description | Example Value | Format |
|----------|-------------|---------------|--------|
| `{DATE}` | Current date | `20260125` | `YYYYmmdd` |
| `{TIME}` | Current time | `143052` | `HHMMSS` |
| `{DATETIME}` | Date and time | `20260125_143052` | `YYYYmmdd_HHMMSS` |
| `{TIMESTAMP}` | Unix timestamp | `1737804652` | Seconds since epoch |

### Project Variables

These variables are always available:

| Variable | Description | Example Value | Notes |
|----------|-------------|---------------|-------|
| `{PROJECT}` | Project name | `myapp` | From `[project].name` |
| `{TARGET}` | Current target name | `release_build` | Build target being executed |

## Examples

### Example 1: Release Build with Full Version

```toml
[project]
name = "firmware"

[env.version]
source = "header"
file = "include/version.h"
string = "FW_VERSION"
build = "BUILD_NUM"

[targets.release]
type = "merge"
bootloader = "default"
app_file = "build/app.hex"
output_name = "{PROJECT}_v{VERSION_FULL}_{DATE}"
# Output: firmware_v1.2.3-rc.1+git.abc123_20260125.hex
```

### Example 2: Nightly Build with Timestamp

```toml
[targets.nightly]
type = "pack"
header = "ota_header"
app_file = "build/app.bin"
output_name = "{PROJECT}_nightly_{DATETIME}"
# Output: firmware_nightly_20260125_143052.fpk
```

### Example 3: Development Build with Target Name

```toml
[targets.dev_debug]
type = "convert"
input_file = "build/debug.hex"
output_format = "bin"
output_name = "{PROJECT}_{TARGET}_{DATE}"
# Output: firmware_dev_debug_20260125.bin
```

### Example 4: Multi-variant Builds

```toml
[env.version]
source = "header"
file = "version.h"
major = "VER_MAJOR"
minor = "VER_MINOR"
patch = "VER_PATCH"

[targets.factory_v1]
type = "merge"
bootloader = "v1_bootloader"
app_file = "build/app_v1.hex"
output_name = "{PROJECT}_factory_v{MAJOR}.{MINOR}.{PATCH}_hw1_{BUILD}"

[targets.factory_v2]
type = "merge"
bootloader = "v2_bootloader"
app_file = "build/app_v2.hex"
output_name = "{PROJECT}_factory_v{MAJOR}.{MINOR}.{PATCH}_hw2_{BUILD}"
```

### Example 5: String-only Configuration

When you only have a version string macro and no separate build number:

```c
// version.h
#define VERSION "v1.2.3"
```

```toml
[env.version]
source = "header"
file = "version.h"
string = "VERSION"
# No build, major, minor, or patch needed!

[targets.release]
output_name = "{PROJECT}_{VERSION}_{DATE}"
# Output: myapp_1.2.3_20260125.hex
```

## Error Handling

Baker validates template variables at build time and will fail with a clear error message if:

1. **Undefined variable**: Template references a variable not available
   ```
   Error: missing template variable 'BUILD' in [env.version]
   ```
   **Solution**: Either remove `{BUILD}` from template or add `build = "BUILD_NUMBER"` to `[env.version]`

2. **Missing configuration**: Required version fields not configured
   ```
   Error: required field 'string' or 'major'/'minor'/'patch' not configured in [env.version]
   ```
   **Solution**: Add either `string` or all three of `major`, `minor`, `patch`

3. **Macro not found**: Specified macro doesn't exist in header file
   ```
   Error: macro 'VERSION_MAJOR' not found in header file
   ```
   **Solution**: Check spelling and ensure the macro is defined in the header file

4. **Parse error**: Invalid version string format
   ```
   Error: failed to parse version string '1.2': expected format: major.minor.patch
   ```
   **Solution**: Ensure version string follows semantic versioning format

5. **Invalid number format**: Macro value cannot be parsed as integer
   ```
   Error: invalid macro value 'abc' for BUILD_NUMBER: not a valid integer
   ```
   **Solution**: Ensure numeric macros contain valid integers (decimal, hex, or binary)

## Best Practices

1. **Use semantic versioning**: Follow the `major.minor.patch` format for consistency
2. **Include build numbers**: Add build numbers for traceability in production
3. **Use date stamps**: Include `{DATE}` or `{TIMESTAMP}` for time-based versioning
4. **Keep names readable**: Balance uniqueness with human readability
5. **Validate early**: Run `baker build` locally before CI/CD to catch template errors
6. **Document variables**: Comment your templates to explain the naming scheme

## Future Enhancements

The following version sources are planned but not yet implemented:

- **CMake**: Extract from `CMakeLists.txt` or `CMakeCache.txt`
- **TOML**: Extract from `Cargo.toml` or custom TOML files
- **Environment**: Extract from environment variables
- **Git**: Extract from git tags and commit info

Stay tuned for updates!
