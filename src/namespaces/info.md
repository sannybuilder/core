## Namespaces

Namespace is a scoped collection of named elements and values with them.

There are two types of namespaces: enums and classes.

### Enum

Enum is a collection of string literals (enum elements) under the common name (enum name)
By default each enum element gets a value that is the element index in the enum, starting with 0.

```
enum X {
  A, // 0
  B, // 1
  C, // 3
}

Enum values could be any integer values, floating-point values, or strings

enum X {
  Pi    = 3.14,
  Zero  = 0
  Sq    = 'Square'
}
```

### Class

Class is a collection of opcodes of two types: methods and properties.
