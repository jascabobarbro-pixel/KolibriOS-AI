# Koli Language

The Kolibri Language (Koli) is an AI-first programming language designed for seamless integration with the KolibriOS AI operating system.

## Features

### AI-Native Syntax

Koli provides native constructs for AI interaction:

```koli
// Define an AI agent
ai CodeAssistant {
    capability: code_completion
    capability: code_explanation
    capability: bug_detection
}

// Use AI in your code
fn get_help(code: string) -> string {
    let explanation = ask CodeAssistant "Explain: " + code;
    return explanation;
}
```

### Living Cell Architecture

Define autonomous cells with properties and behaviors:

```koli
cell SelfHealingMemory {
    // Properties (state)
    let total_size: int = 1024 * 1024
    let used_size: int = 0

    // Behaviors (methods)
    behavior allocate(size: int) -> pointer {
        if used_size + size > total_size {
            self.heal();
        }
        let ptr = internal_alloc(size);
        used_size = used_size + size;
        return ptr;
    }

    behavior heal() {
        // Self-healing logic
        compact();
    }
}

// Spawn cells
let memory = spawn SelfHealingMemory();
```

### Modern Language Features

- **Type inference**: `let x = 42;`
- **Explicit types**: `let name: string = "Kolibri";`
- **Generics**: `fn process<T>(item: T) -> T`
- **Arrays**: `let items = [1, 2, 3];`
- **Structs**: `let point = { x: 10, y: 20 };`

### Control Flow

```koli
// Conditionals
if x > 0 {
    print("positive");
} else if x < 0 {
    print("negative");
} else {
    print("zero");
}

// While loops
while condition {
    do_something();
}

// For-in loops
for item in collection {
    process(item);
}
```

## Project Structure

```
koli_lang/
├── compiler/           # Koli compiler
│   ├── src/
│   │   ├── lib.rs     # Compiler entry point
│   │   ├── lexer.rs   # Lexical analyzer
│   │   ├── parser.rs  # Parser (AST builder)
│   │   ├── ast.rs     # Abstract Syntax Tree definitions
│   │   ├── type_check.rs  # Type checking
│   │   └── codegen.rs # Code generation
│   └── Cargo.toml
├── runtime/            # Koli runtime
│   ├── src/
│   │   ├── lib.rs     # Runtime entry point
│   │   ├── value.rs   # Value types
│   │   ├── vm.rs      # Virtual machine
│   │   ├── gc.rs      # Garbage collector
│   │   └── ai_bridge.rs # AI integration
│   └── Cargo.toml
└── examples/           # Example Koli programs
    ├── hello.koli
    ├── ai_demo.koli
    ├── cell_demo.koli
    ├── simple_test.koli
    └── comprehensive_test.koli
```

## Compiler Pipeline

1. **Lexer**: Tokenizes source code into tokens
2. **Parser**: Builds Abstract Syntax Tree (AST) from tokens
3. **Type Check**: Validates types and builds symbol table
4. **Code Generator**: Produces target code (Rust, LLVM IR, or Bytecode)

## Building

```bash
# Build the compiler
cargo build -p koli-compiler

# Build the runtime
cargo build -p koli-runtime

# Run tests
cargo test -p koli-compiler
```

## Language Reference

### Types

| Type | Description | Example |
|------|-------------|---------|
| `int` | 64-bit integer | `42`, `-100` |
| `float` | 64-bit float | `3.14`, `1.5e-10` |
| `bool` | Boolean | `true`, `false` |
| `string` | UTF-8 string | `"hello"` |
| `array<T>` | Array of T | `[1, 2, 3]` |
| `pointer<T>` | Pointer to T | `*ptr` |
| `void` | No value | (return type only) |

### Operators

| Operator | Description | Precedence |
|----------|-------------|------------|
| `!` `-` | Unary not, negate | Highest |
| `*` `/` `%` | Multiply, divide, modulo | High |
| `+` `-` | Add, subtract | Medium |
| `<` `<=` `>` `>=` | Comparison | Low |
| `==` `!=` | Equality | Lower |
| `&&` | Logical and | Lower |
| `\|\|` | Logical or | Lowest |

### Keywords

```
fn let if else while for in return break continue
ai ask cell spawn capability behavior property
int float bool string void array pointer
true false null self
```

### Special Syntax

| Syntax | Description |
|--------|-------------|
| `->` | Return type arrow |
| `::` | Namespace separator |
| `..` | Range operator |
| `+=` `-=` `*=` `/=` | Compound assignment |
| `?` | Optional type suffix |

## Integration with KolibriOS

Koli code compiles to Rust or bytecode that integrates with KolibriOS components:

- **Cells** become KolibriOS cell services
- **AI definitions** connect to the unified AI agent
- **Kernel interaction** through the gene API

## License

Licensed under the MIT License. See LICENSE file for details.
