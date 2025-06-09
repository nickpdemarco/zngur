# Example: Import and Merge

A clean demonstration of Zngur's `import` and `merge` functionality using four focused modules.

## Structure

- **`primitives.zng`** - Defines basic primitive types (`bool`)
- **`foo.{zng,cpp}`** - Imports primitives, defines `Vec<i32>` APIs and returns a populated Vec
- **`bar.{zng,cpp}`** - Imports primitives, defines `Option<String>` APIs and returns an Option
- **`main.{zng,cpp}`** - Imports foo and bar (transitively gets primitives), extends both types with additional APIs, and demonstrates everything

## Key Features Demonstrated

1. **Import**: `main.zng` imports APIs from two separate `.zng` files
2. **Transitive Import**: `main.zng` automatically gets `primitives.zng` types through `foo.zng` and `bar.zng`
3. **API Extension**: `main.zng` adds new methods to imported `Vec<i32>` and `Option<String>` types
4. **Merge**: The `std` module is automatically merged across all files, combining APIs seamlessly
5. **Interop**: C++ functions create and return Rust types seamlessly
6. **Separation**: Each module focuses on specific types, with shared primitives

## API Extensions in main.zng

The main module doesn't just import - it extends the imported types:

- **Vec<i32>**: Adds `is_empty()` and `clear()` methods beyond `foo.zng`'s `new()` and `push()`
- **Option<String>**: Adds `unwrap()` method beyond `bar.zng`'s constructors and `is_some()`/`is_none()`

## Files

```
primitives.zng  # Defines primitive types (bool)
foo.zng         # Imports primitives, exports Vec<i32> APIs
foo.cpp         # Creates and returns Vec<i32> with data
bar.zng         # Imports primitives, exports Option<String> APIs
bar.cpp         # Creates and returns Option<String>
main.zng        # Imports foo.zng and bar.zng (gets primitives transitively), extends both types
main.cpp        # Uses both imported and extended APIs, demonstrates everything
```

## Running

```bash
make
./a.out
```

## Expected Output

```
=== Import/Merge Demonstration ===

foo(): Creating a Rust Vec<i32>
  Vec contents: [main.cpp:16] numbers = [
    10,
    20,
    30,
]
  Vec is_empty(): 0
  After clear(), is_empty(): 1

bar(): Creating Rust Option<String>
  some_value.is_some(): 1
  none_value.is_none(): 1
  Unwrapped string: [main.cpp:36] unwrapped_string = ""

=== Extended APIs from main.zng working alongside imported APIs ===
```

This demonstrates how modules can define their own APIs with shared primitive dependencies, how the import system supports transitive dependency resolution, and how higher-level modules can extend imported types with additional functionality - all merged into a single cohesive interface accessible from C++.
