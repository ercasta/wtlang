# WTLang Error Codes Reference

This document provides a comprehensive reference for all error codes in the WTLang compiler. Each error has a unique code for easy identification, documentation, and tooling integration (including LSP support).

## Error Code Categories

- **E1xxx**: Lexical errors (tokenization phase)
- **E2xxx**: Syntax errors (parsing phase)
- **E3xxx**: Semantic errors (type checking and validation)
- **E4xxx**: Table and data errors
- **E5xxx**: Import and external function errors

## Lexical Errors (E1xxx)

### E1001: Unterminated String Literal

**Description**: A string was started with a quote but never closed.

**Example**:
```wtlang
let message: string = "Hello, World
// Missing closing quote
```

**How to fix**: Add a closing quote (") at the end of the string.

```wtlang
let message: string = "Hello, World!"
```

---

### E1002: Invalid Number Format

**Description**: The number contains invalid characters or format.

**Example**:
```wtlang
let count: int = 12.34.56
// Multiple decimal points
```

**How to fix**: Use proper number format with at most one decimal point.

```wtlang
let count: float = 12.34
```

---

### E1003: Invalid Character

**Description**: An unexpected character was encountered that is not valid in WTLang.

**Example**:
```wtlang
let value = 42 @
// @ is not a valid character
```

**How to fix**: Remove or replace the invalid character.

---

### E1004: Unexpected End of File

**Description**: The file ended unexpectedly while parsing a token.

**How to fix**: Complete the statement or expression before the end of the file.

---

## Syntax Errors (E2xxx)

### E2001: Missing Closing Brace

**Description**: An opening brace `{` was found but no matching closing brace `}`.

**Example**:
```wtlang
page HomePage {
    text "Hello"
// Missing }
```

**How to fix**: Add the closing brace.

```wtlang
page HomePage {
    text "Hello"
}
```

---

### E2003: Missing Closing Parenthesis

**Description**: An opening parenthesis `(` was found but no matching closing parenthesis `)`.

**Example**:
```wtlang
function calculate(x: int, y: int {
    return x + y
}
```

**How to fix**: Add the closing parenthesis.

```wtlang
function calculate(x: int, y: int) {
    return x + y
}
```

---

### E2007: Expected Identifier

**Description**: An identifier (variable name, function name, etc.) was expected but not found.

**Example**:
```wtlang
let : int = 42
// Missing variable name
```

**How to fix**: Provide a valid identifier.

```wtlang
let count: int = 42
```

---

### E2008: Expected Type Annotation

**Description**: A type annotation was expected but not provided.

**Example**:
```wtlang
function add(x, y) -> int {
    return x + y
}
```

**How to fix**: Add type annotations for parameters.

```wtlang
function add(x: int, y: int) -> int {
    return x + y
}
```

---

### E2011: Unexpected Token

**Description**: A token was found that doesn't fit the expected syntax.

**Example**:
```wtlang
page Test {
    let x: int = 5 5
    // Two numbers in a row
}
```

**How to fix**: Remove the unexpected token or add the missing operator.

```wtlang
page Test {
    let x: int = 5 + 5
}
```

---

### E2016: Missing Colon in Type Annotation

**Description**: Type annotations require a colon `:` before the type.

**Example**:
```wtlang
let count int = 5
// Missing colon
```

**How to fix**: Add the colon before the type.

```wtlang
let count: int = 5
```

---

## Semantic Errors (E3xxx)

### E3001: Undefined Variable

**Description**: A variable is used before it is declared.

**Example**:
```wtlang
page Test {
    display count
    // count was never declared
}
```

**How to fix**: Declare the variable before using it.

```wtlang
page Test {
    let count: int = 42
    display count
}
```

---

### E3002: Undefined Function

**Description**: A function is called but was never defined.

**Example**:
```wtlang
page Test {
    let result: int = calculate(5, 10)
    // calculate is not defined
}
```

**How to fix**: Define the function or import it.

```wtlang
function calculate(x: int, y: int) -> int {
    return x + y
}

page Test {
    let result: int = calculate(5, 10)
}
```

---

### E3003: Undefined Table

**Description**: A table type is referenced but was never defined.

**Example**:
```wtlang
page Test {
    let users: table(User) = load_csv(User, "users.csv")
    // User table not defined
}
```

**How to fix**: Define the table structure first.

```wtlang
table User {
    id: int
    name: string
}

page Test {
    let users: table(User) = load_csv(User, "users.csv")
}
```

---

### E3004: Variable Already Defined

**Description**: Attempting to define a variable that already exists in the current scope.

**Example**:
```wtlang
page Test {
    let count: int = 5
    let count: int = 10
    // Duplicate definition
}
```

**How to fix**: Use a different variable name or use assignment instead.

```wtlang
page Test {
    let count: int = 5
    count = 10  // Assignment, not redefinition
}
```

---

### E3007: Type Mismatch in Assignment

**Description**: The value being assigned doesn't match the variable's declared type.

**Example**:
```wtlang
page Test {
    let count: int = "hello"
    // String assigned to int variable
}
```

**How to fix**: Ensure types match.

```wtlang
page Test {
    let count: int = 42
    let message: string = "hello"
}
```

---

### E3010: Wrong Number of Arguments

**Description**: A function is called with the wrong number of arguments.

**Example**:
```wtlang
function add(x: int, y: int) -> int {
    return x + y
}

page Test {
    let result: int = add(5)
    // Missing second argument
}
```

**How to fix**: Provide all required arguments.

```wtlang
page Test {
    let result: int = add(5, 10)
}
```

---

### E3011: Variable Used Before Initialization

**Description**: A variable is declared but used before it's assigned a value.

**Example**:
```wtlang
page Test {
    let result: int
    display result  // Used before initialization
}
```

**How to fix**: Initialize the variable before use or use conditional initialization.

```wtlang
page Test {
    let result: int
    if true {
        result = 42
    } else {
        result = 0
    }
    display result  // Now properly initialized
}
```

---

### E3012: Field Does Not Exist

**Description**: Accessing a field that doesn't exist in the table definition.

**Example**:
```wtlang
table User {
    id: int
    name: string
}

page Test {
    let users: table(User) = load_csv(User, "users.csv")
    forall user in users {
        display user.email  // email field doesn't exist
    }
}
```

**How to fix**: Use an existing field or add it to the table definition.

```wtlang
forall user in users {
    display user.name
}
```

---

## Table/Data Errors (E4xxx)

### E4001: Table Structure Mismatch with CSV

**Description**: The CSV file structure doesn't match the table definition.

**How to fix**: Ensure the CSV columns match the table field definitions, or update the table definition to match the CSV structure.

---

### E4005: Invalid Filter Definition

**Description**: A filter is defined incorrectly.

**Example**:
```wtlang
let name_filter: filter = filter(Users, "nonexistent", "single")
```

**How to fix**: Ensure the column exists and filter mode is valid ("single" or "multi").

---

### E4006: Filter on Non-Existent Column

**Description**: Attempting to create a filter on a column that doesn't exist in the table.

**How to fix**: Use an existing column name or add the column to the table definition.

---

## Import/External Errors (E5xxx)

### E5001: Cannot Find External Module

**Description**: The specified external Python module file could not be found.

**Example**:
```wtlang
external function process(data: string) -> string from "missing_module.py"
```

**How to fix**: Ensure the module file exists and the path is correct.

---

### E5002: Invalid External Function Definition

**Description**: The external function definition has incorrect syntax.

**How to fix**: Follow the correct external function syntax:
```wtlang
external function name(param: type) -> return_type from "module.py"
```

---

## Error Message Format

Errors are formatted as follows:

```
severity[code]: message
  --> location
   |
 N | source code line
   |
  = help: suggestion
```

**Example**:
```
error[E3001]: Undefined variable 'count'
  --> example.wt:5:13
   |
 5 |     display count
   |
  = help: Declare the variable before using it with 'let count'
```

## Using Error Codes in Tools

### For Compiler Users

Error codes help you:
- Quickly identify the type of problem
- Search documentation for specific errors
- Filter and categorize build errors

### For LSP Integration

Error codes enable:
- Consistent error reporting across tools
- Code actions and quick fixes
- Error documentation on hover
- Error filtering in IDEs

### For CI/CD

Error codes allow:
- Automated error categorization
- Custom error handling rules
- Error trend tracking
- Quality metrics

## Best Practices

1. **Read the error message carefully**: The message contains specific information about what went wrong
2. **Check the location**: Line and column numbers point to where the error occurred
3. **Review the help text**: Many errors include suggestions for fixing the issue
4. **Look for related errors**: Sometimes fixing one error resolves others
5. **Consult this reference**: For detailed explanations and examples

## Future Additions

As the language evolves, new error codes will be added. The error code system is designed to be extensible while maintaining backward compatibility.
