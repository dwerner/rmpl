# YAML Test Suite Scope

## Overview
This document defines the scope of YAML features supported by the `yaml-parse` library.

## Supported Features

### Scalars
- ✅ Strings (quoted with `"` or `'`)
- ✅ Unquoted strings
- ✅ Integers (unsigned u64)
- ✅ Floats (f64)
- ✅ Booleans (`true`, `false`)
- ✅ Null values (`null`, `~`)

### Structures
- ✅ Blocks (key-value pairs)
- ✅ Lists (array items with `-`)
- ✅ Nested structures (blocks within blocks, lists within blocks)

### Comments
- ✅ Line comments (`# comment`)

## Not Supported (Future)
- Flow style (`[1, 2, 3]`, `{a: 1}`)
- Multi-line strings (`|`, `>`)
- Anchors and aliases (`&`, `*`)
- Tags (`!!`)
- Document markers (`---`, `...`)
- Single-line comments after values

## Test Cases

### Scalar Tests
1. **Double-quoted string**: `"hello world"` → `Value::String("hello world")`
2. **Single-quoted string**: `'hello world'` → `Value::String("hello world")`
3. **Unquoted string**: `hello` → `Value::String("hello")`
4. **Integer**: `42` → `Value::Number(42)`
5. **Float**: `3.14` → `Value::Float(3.14)`
6. **Boolean true**: `true` → `Value::Bool(true)`
7. **Boolean false**: `false` → `Value::Bool(false)`
8. **Null**: `null` → `Value::Null`
9. **Null tilde**: `~` → `Value::Null`

### Structure Tests
1. **Simple key-value**: `name: value` → `Value::Map([("name", Value::String("value"))])`
2. **Multiple key-values**: `a: 1\nb: 2` → `Value::Map([("a", Number(1)), ("b", Number(2))])`
3. **Simple list**: `- a\n- b` → `Value::List([String("a"), String("b")])`
4. **Nested block**: `person:\n  name: Alice` → `Value::Map([("person", Map([("name", String("Alice"))]))])`
5. **List in block**: `items:\n  - a\n  - b` → `Value::Map([("items", List([String("a"), String("b")]))])`

### Comment Tests
1. **Line comment**: `# comment\nkey: value` → parses without comment
2. **Trailing comment**: `key: value # comment` → parses value only

## Round-trip Tests (with serduh)
1. Serialize struct → YAML → deserialize → struct
2. Serialize nested struct → YAML → deserialize → struct
3. Serialize list → YAML → deserialize → list
