# Import

The `import` directive allows you to include type definitions and other declarations from other `.zng` files into your main specification. Types, traits, and modules can appear multiple times across the transitive set of imported files, and their content is merged together.

## Syntax

Zngur supports two types of import statements:

```zng
// File path imports (relative to current file)
import "path/to/file.zng";

// Dependency-based imports (from Cargo dependencies)
import "@dependency-name/path/to/file.zng";
```

## Path Resolution

### File Path Imports

File path imports are resolved relative to the directory containing the current `.zng` file:

- `import "./types.zng";` - relative to current directory
- `import "subdir/types.zng";` - relative subdirectory

Above, "current" refers to the `.zng` file being parsed, which is not necessarily the top-level `.zng` file passed to `zngur` on the command line.

**Note**: Absolute paths (e.g., `"/absolute/path.zng"`) are not supported.

### Dependency-Based Imports

Dependency-based imports use the `@dependency-name/path` syntax to reference files within Cargo dependencies:

- `import "@my-crate/types.zng";` - imports `types.zng` from the root of the `my-crate` dependency
- `import "@my-crate/subdir/types.zng";` - imports from a subdirectory within the dependency

The dependency name corresponds to the name used in your `Cargo.toml` dependencies section. If you've renamed a dependency, use the renamed version.

**Requirements**:
- Dependency-based imports require the `--cargo-manifest` flag to specify the path to your `Cargo.toml` file
- If your project is a workspace with multiple packages, you may also need the `--package` flag to specify which package's dependencies to use

```bash
# Basic usage with cargo manifest
zngur generate main.zng --cargo-manifest ./Cargo.toml

# Workspace usage with specific package
zngur generate main.zng --cargo-manifest ./Cargo.toml --package my-package
```

## Behavior

When an import statement is processed:

1. The parser reads and parses the imported file
2. All declarations from the imported file are _merged_ into the current specification
3. Imported content becomes available as if it were defined in the importing file
4. Import processing happens recursively - imported files can themselves contain import statements

## Merging

Zngur's merge algorithm attempts to compute the union of each set of declarations which share an identity (e.g. every `type crate::Inventory { ... }` across all imported files). Duplicates are ignored, but contradictions will raise a compiler error. For example, if two different `type crate::Inventory { ... }` declarations both specify `wellknown_traits(Debug);`, parsing will succeed. However, if they specify different layouts, an error will be reported.

## Example

For a complete working example of both file path and dependency-based imports, see the [`examples/import`](https://github.com/zngur/zngur/tree/main/examples/import) directory in the repository.

This example demonstrates:

- Using `@dependency-name/file.zng` syntax with renamed dependencies
- Merging type definitions across multiple imported files
- Extending imported types with additional methods
- Proper `Cargo.toml` configuration for dependency imports
