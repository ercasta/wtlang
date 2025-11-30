# WTLang Syntax Reference

**Version:** 0.1.0  
**Date:** November 30, 2025  
**Status:** Current implementation

This document provides a complete reference for all implemented WTLang syntax elements. It serves as the authoritative guide for what is currently supported in the compiler and LSP.

---

## Table of Contents

1. [Program Structure](#program-structure)
2. [Comments](#comments)
3. [Table Definitions](#table-definitions)
4. [Page Definitions](#page-definitions)
5. [Function Definitions](#function-definitions)
6. [External Functions](#external-functions)
7. [Test Blocks](#test-blocks)
8. [Statements](#statements)
9. [Expressions](#expressions)
10. [Types](#types)
11. [Operators](#operators)
12. [Built-in Functions](#built-in-functions)
13. [Reserved Keywords](#reserved-keywords)
14. [Known Limitations](#known-limitations)

---

## Program Structure

A WTLang program consists of zero or more top-level items:

```ebnf
Program ::= ProgramItem*

ProgramItem ::=
    | TableDef
    | FunctionDef
    | ExternalFunction
    | Page
    | Test
```

**Example:**
```wtlang
table User {
    id: int
    name: string
}

function process_users(users: table) -> int {
    return count(users)
}

page Dashboard {
    title "Main Dashboard"
}
```

---

## Comments

**Single-line comments:** Start with `//` and continue to end of line

```wtlang
// This is a comment
let x = 42  // Comment after code
```

**Multi-line comments:** Not currently supported

---

## Table Definitions

Tables define structured data types with fields and constraints.

### Syntax

```ebnf
TableDef ::= "table" Identifier "{" Field* "}"

Field ::= Identifier ":" Type Constraints?

Constraints ::= "[" Constraint ("," Constraint)* "]"

Constraint ::=
    | "unique"
    | "non_null"
    | "validate" "(" Expr ")"
    | "references" Identifier "." Identifier
```

### Examples

**Basic table:**
```wtlang
table Product {
    id: int
    name: string
    price: currency
}
```

**Table with constraints:**
```wtlang
table User {
    id: int [unique, non_null]
    email: string [non_null, validate(x => contains(x, "@"))]
    age: int [validate(x => x >= 0 && x <= 150)]
    status: string
}
```

**Table with foreign keys:**
```wtlang
table Order {
    order_id: int [unique, non_null]
    customer_id: int [references Customer.id]
    product_id: int [references Product.id]
    quantity: int [validate(x => x > 0)]
}
```

---

## Page Definitions

Pages are the main UI containers in WTLang applications.

### Syntax

```ebnf
Page ::= "page" Identifier "{" Statement* "}"
```

### Example

```wtlang
page HomePage {
    title "Welcome"
    subtitle "Dashboard"
    
    let users = load_csv("users.csv", User)
    show(users)
}
```

**Scope:** Each page has its own variable scope. Variables declared in one page are not visible in another.

---

## Function Definitions

Functions encapsulate reusable logic.

### Syntax

```ebnf
FunctionDef ::= "function" Identifier "(" Parameters? ")" "->" Type "{" Statement* "}"

Parameters ::= Parameter ("," Parameter)*

Parameter ::= Identifier ":" Type
```

### Examples

**Simple function:**
```wtlang
function double(x: int) -> int {
    return x * 2
}
```

**Function with table parameter:**
```wtlang
function filter_adults(users: table) -> table {
    return users -> where(_, row => row.age >= 18)
}
```

**Function with multiple parameters:**
```wtlang
function calculate_total(price: float, quantity: int, tax_rate: float) -> float {
    let subtotal = price * quantity
    let tax = subtotal * tax_rate
    return subtotal + tax
}
```

---

## External Functions

External functions allow calling Python code from WTLang.

### Syntax

```ebnf
ExternalFunction ::= "external" "function" Identifier "(" Parameters? ")" "->" Type "from" StringLiteral
```

### Examples

```wtlang
external function analyze_sentiment(text: string) -> float from "nlp.sentiment"

external function predict_churn(data: table) -> table from "ml.models.churn"
```

**Usage:**
```wtlang
page Analysis {
    let text = "This product is great!"
    let score = analyze_sentiment(text)
    text "Sentiment: {score}"
}
```

---

## Test Blocks

Test blocks define test cases (syntax parsed but execution not implemented).

### Syntax

```ebnf
Test ::= "test" StringLiteral "{" Statement* "}"
```

### Example

```wtlang
test "filter removes rows correctly" {
    let users = table_from([
        {name: "Alice", age: 25},
        {name: "Bob", age: 17}
    ])
    let adults = users -> where(_, row => row.age >= 18)
    
    assert count(adults) == 1
}
```

---

## Statements

Statements are executable actions within pages, functions, and test blocks.

### Variable Declaration

```ebnf
Let ::= "let" Identifier (":" Type)? ("=" Expr)?
```

**Examples:**
```wtlang
let x = 42                    // Type inferred as int
let name: string = "Alice"    // Explicit type
let result: float             // Declaration without initialization
```

### Variable Assignment

```ebnf
Assign ::= Identifier "=" Expr
```

**Example:**
```wtlang
let total: float
if condition {
    total = 100.0
} else {
    total = 50.0
}
```

### Display Statements

```ebnf
Title ::= "title" StringLiteral
Subtitle ::= "subtitle" StringLiteral
Text ::= "text" StringLiteral
```

**Examples:**
```wtlang
title "My Application"
subtitle "Dashboard Overview"
text "Welcome, user!"
text "Total: {total_amount}"  // String interpolation supported
```

### Button

```ebnf
Button ::= "button" StringLiteral "{" Statement* "}"
```

**Example:**
```wtlang
button "Save Changes" {
    save_csv(users, "updated_users.csv")
    text "Saved successfully!"
}
```

### Section

```ebnf
Section ::= "section" StringLiteral "{" Statement* "}"
```

**Example:**
```wtlang
section "Summary Statistics" {
    let total = sum(sales, "amount")
    let average = average(sales, "amount")
    
    text "Total: ${total}"
    text "Average: ${average}"
}
```

### Conditional (if/else)

```ebnf
If ::= "if" Expr "{" Statement* "}" ("else" "{" Statement* "}")?
```

**Example:**
```wtlang
if count(users) > 0 {
    show(users)
} else {
    text "No users found"
}
```

### Loop (forall)

```ebnf
Forall ::= "forall" Identifier "in" Expr "{" Statement* "}"
```

**Example:**
```wtlang
let categories = ["Electronics", "Clothing", "Food"]

forall category in categories {
    section category {
        let filtered = products -> where(_, p => p.category == category)
        show(filtered)
    }
}
```

### Return

```ebnf
Return ::= "return" Expr
```

**Example:**
```wtlang
function get_max(a: int, b: int) -> int {
    if a > b {
        return a
    } else {
        return b
    }
}
```

### Function Call Statement

Function calls can also be statements (for functions with side effects):

```wtlang
save_csv(users, "output.csv")
show(products)
```

---

## Expressions

### Literals

```ebnf
Literal ::=
    | IntLiteral        // 42, -10, 0
    | FloatLiteral      // 3.14, -0.5, 2.0
    | StringLiteral     // "hello", "world"
    | BoolLiteral       // true, false
```

**Examples:**
```wtlang
let count = 42
let price = 99.99
let name = "Alice"
let active = true
```

### Identifiers

Variable and function names:

```wtlang
let users = load_csv("users.csv", User)
let result = calculate_total(price, quantity, tax_rate)
```

### Function Calls

```ebnf
FunctionCall ::= Identifier "(" Arguments? ")"

Arguments ::= Expr ("," Expr)*
```

**Examples:**
```wtlang
count(users)
sum(sales, "amount")
load_csv("data.csv", Product)
where(users, row => row.age >= 18)
```

**Note:** Named arguments (like `on: (a, b) => ...`) are NOT currently supported.

### Binary Operations

```ebnf
BinaryOp ::=
    | Expr "+" Expr      // Addition
    | Expr "-" Expr      // Subtraction
    | Expr "*" Expr      // Multiplication
    | Expr "/" Expr      // Division
    | Expr "%" Expr      // Modulo
    | Expr "==" Expr     // Equal
    | Expr "!=" Expr     // Not equal
    | Expr "<" Expr      // Less than
    | Expr "<=" Expr     // Less than or equal
    | Expr ">" Expr      // Greater than
    | Expr ">=" Expr     // Greater than or equal
    | Expr "&&" Expr     // Logical AND
    | Expr "||" Expr     // Logical OR
```

**Examples:**
```wtlang
let total = price * quantity
let is_adult = age >= 18
let is_valid = count > 0 && status == "active"
```

### Unary Operations

```ebnf
UnaryOp ::=
    | "!" Expr          // Logical NOT
    | "-" Expr          // Numeric negation
```

**Examples:**
```wtlang
let is_inactive = !active
let negative = -value
```

### Lambda Expressions

```ebnf
Lambda ::= "(" Parameters? ")" "=>" Expr

// For single parameter, shorthand allowed:
Lambda ::= Identifier "=>" Expr
```

**Examples:**
```wtlang
// Single parameter
let adults = users -> where(_, row => row.age >= 18)

// Multiple parameters (used in join - NOT IMPLEMENTED)
// on: (order, customer) => order.customer_id == customer.customer_id
```

### Field Access

```ebnf
FieldAccess ::= Expr "." Identifier
```

**Example:**
```wtlang
let user_age = user.age
let product_price = product.price
```

**Context:** Primarily used in lambda expressions for row field access.

### Index Access

```ebnf
Index ::= Expr "[" Expr "]"
```

**Example:**
```wtlang
let first_user = users[0]
let item = items[index]
```

### Pipeline (Chain)

```ebnf
Chain ::= Expr "->" Expr
```

**Examples:**
```wtlang
let result = users
    -> where(_, row => row.active)
    -> sort(_, "name")
    -> limit(_, 10)

// With underscore placeholder
let filtered = products -> where(_, p => p.price > 100.0)
```

The underscore `_` represents the value being piped through the chain.

### Array Literals

```ebnf
ArrayLiteral ::= "[" (Expr ("," Expr)*)? "]"
```

**Examples:**
```wtlang
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]
let filters = [filter("department", single), filter("role", multi)]
```

### Table Literals

```ebnf
TableLiteral ::= "{" (Identifier ":" Expr ("," Identifier ":" Expr)*)? "}"
```

**Example:**
```wtlang
let user = {name: "Alice", age: 25, email: "alice@example.com"}
```

**Note:** Primarily used with `table_from()` for testing.

### Filter Literals

```ebnf
FilterLiteral ::= "filter" "(" StringLiteral "," FilterMode ")"

FilterMode ::= "single" | "multi"
```

**Examples:**
```wtlang
let dept_filter = filter("department", single)
let role_filter = filter("role", multi)

show(employees, [filter("department", single), filter("position", multi)])
```

---

## Types

### Primitive Types

| Type | Description | Example Values |
|------|-------------|----------------|
| `int` | Integer number | `42`, `-10`, `0` |
| `float` | Floating-point number | `3.14`, `-0.5`, `99.99` |
| `string` | Text string | `"hello"`, `"world"` |
| `date` | Date value | (parsed from strings in CSV) |
| `currency` | Monetary value | `99.99`, `1234.56` |
| `bool` | Boolean | `true`, `false` |

### Complex Types

| Type | Description | Example |
|------|-------------|---------|
| `table` | Table type (unspecified) | Used for generic table parameters |
| `table<TypeName>` | Table of specific type | `table<User>`, `table<Product>` |
| `filter` | Filter for table columns | `filter("column", single)` |

### Type Annotations

Type annotations can be used in:
- Variable declarations: `let x: int = 42`
- Function parameters: `function foo(x: int) -> int`
- Function return types: `-> string`
- Variable declarations without initialization: `let result: float`

---

## Operators

### Arithmetic Operators

| Operator | Description | Example | Precedence |
|----------|-------------|---------|------------|
| `+` | Addition | `a + b` | 4 |
| `-` | Subtraction | `a - b` | 4 |
| `*` | Multiplication | `a * b` | 5 |
| `/` | Division | `a / b` | 5 |
| `%` | Modulo | `a % b` | 5 |

### Comparison Operators

| Operator | Description | Example | Precedence |
|----------|-------------|---------|------------|
| `==` | Equal | `a == b` | 3 |
| `!=` | Not equal | `a != b` | 3 |
| `<` | Less than | `a < b` | 3 |
| `<=` | Less than or equal | `a <= b` | 3 |
| `>` | Greater than | `a > b` | 3 |
| `>=` | Greater than or equal | `a >= b` | 3 |

### Logical Operators

| Operator | Description | Example | Precedence |
|----------|-------------|---------|------------|
| `&&` | Logical AND | `a && b` | 2 |
| `||` | Logical OR | `a || b` | 1 |
| `!` | Logical NOT | `!a` | 6 |

### Other Operators

| Operator | Description | Example | Precedence |
|----------|-------------|---------|------------|
| `->` | Pipeline/Chain | `x -> f(_, y)` | 0 |
| `.` | Field access | `user.name` | 7 |
| `[]` | Index access | `arr[0]` | 7 |
| `=>` | Lambda | `x => x + 1` | N/A |

**Precedence:** Higher numbers bind tighter (7 = highest, 0 = lowest)

---

## Built-in Functions

### Data Loading and Saving

#### `load_csv(filename: string, table_type) -> table`

Load a CSV file into a table with validation against the table definition.

```wtlang
let users = load_csv("users.csv", User)
```

#### `save_csv(table, filename: string)`

Save a table to a CSV file.

```wtlang
save_csv(updated_users, "users_updated.csv")
```

### Display Functions

#### `show(table, filters?: filter[]) -> table`

Display a table with optional filters. Returns the table for chaining.

```wtlang
show(users)
show(users, [filter("department", single)])
```

#### `show_editable(table, filters?: filter[]) -> table`

Display an editable table with optional filters. Returns the edited table.

```wtlang
let updated = show_editable(users)
let updated_filtered = show_editable(users, [filter("role", multi)])
```

### Table Transformation Functions

#### `where(table, predicate: row -> bool) -> table`

Filter table rows based on a predicate function.

```wtlang
let adults = where(users, row => row.age >= 18)

// With chaining
let active_adults = users
    -> where(_, row => row.active)
    -> where(_, row => row.age >= 18)
```

#### `sort(table, column: string) -> table`

Sort table by column in ascending order.

```wtlang
let sorted = sort(users, "name")

// With chaining
let sorted_users = users -> sort(_, "age")
```

#### `sort_desc(table, column: string) -> table`

Sort table by column in descending order.

```wtlang
let sorted = sort_desc(sales, "amount")
```

### Aggregation Functions

#### `sum(table, column: string) -> number`

Calculate the sum of a numeric column.

```wtlang
let total_sales = sum(sales, "amount")
```

#### `average(table, column: string) -> number`

Calculate the average of a numeric column.

```wtlang
let avg_age = average(users, "age")
```

#### `count(table) -> int`

Count the number of rows in a table.

```wtlang
let user_count = count(users)
```

#### `min(table, column: string) -> number`

Find the minimum value in a numeric column.

```wtlang
let min_price = min(products, "price")
```

#### `max(table, column: string) -> number`

Find the maximum value in a numeric column.

```wtlang
let max_price = max(products, "price")
```

### Utility Functions

#### `filter(column: string, mode: single|multi) -> filter`

Create a filter definition for use with `show()` and `show_editable()`.

```wtlang
let dept_filter = filter("department", single)
let filters = [filter("dept", single), filter("role", multi)]
```

#### `table_from(data: array) -> table`

Create a table from an array of objects (primarily for testing).

```wtlang
let test_data = table_from([
    {name: "Alice", age: 25},
    {name: "Bob", age: 30}
])
```

### Advanced Functions (DOCUMENTED BUT NOT IMPLEMENTED)

The following functions are mentioned in the tutorial but are **NOT currently implemented**:

- ❌ `join(table1, table2, on: (a, b) => bool) -> table` - Join two tables
- ❌ `select(table, columns: string[]) -> table` - Select specific columns
- ❌ `add_column(table, name: string, expr: row -> value) -> table` - Add computed column
- ❌ `group_by(table, column: string, aggregations: {...}) -> table` - Group and aggregate
- ❌ `limit(table, n: int) -> table` - Limit number of rows
- ❌ `export_excel(table, filename: string)` - Export to Excel

**Note:** These functions would require:
1. Named argument syntax (e.g., `on: ...`)
2. Extended AST to support named parameters
3. Code generation for complex operations

---

## Reserved Keywords

### Top-level Keywords

| Keyword | Purpose |
|---------|---------|
| `table` | Define a table type |
| `page` | Define a page |
| `function` | Define a function |
| `external` | Declare external function |
| `test` | Define a test case |

### Statement Keywords

| Keyword | Purpose |
|---------|---------|
| `let` | Declare a variable |
| `if` | Conditional statement |
| `else` | Else branch |
| `forall` | Loop over collection |
| `in` | Used in forall |
| `return` | Return from function |
| `title` | Set page title |
| `subtitle` | Set page subtitle |
| `text` | Display text |
| `button` | Create a button |
| `section` | Create a section |

### Type Keywords

| Keyword | Type |
|---------|------|
| `int` | Integer type |
| `float` | Floating-point type |
| `string` | String type |
| `date` | Date type |
| `currency` | Currency type |
| `bool` | Boolean type |

### Constraint Keywords

| Keyword | Purpose |
|---------|---------|
| `unique` | Unique constraint |
| `non_null` | Non-null constraint |
| `validate` | Validation constraint |
| `references` | Foreign key reference |

### Other Keywords

| Keyword | Purpose |
|---------|---------|
| `from` | Used in external/import |
| `single` | Single-select filter |
| `multi` | Multi-select filter |
| `filter` | Filter type/function |

---

## Known Limitations

### 1. Named Arguments Not Supported

The documentation shows syntax like:
```wtlang
join(orders, products, on: (o, p) => o.product_id == p.product_id)
```

This is **NOT supported**. All function arguments must be positional.

### 2. Missing Built-in Functions

The following functions are documented in the tutorial but not implemented:
- `join()` - Table joins
- `select()` - Column selection
- `add_column()` - Computed columns
- `group_by()` - Grouping and aggregation
- `limit()` - Row limiting
- `export_excel()` - Excel export

### 3. Multi-line Comments Not Supported

Only single-line comments (`//`) are supported.

### 4. No String Escaping Documentation

While string literals support basic escaping, the exact escape sequences supported are not formally documented.

### 5. No Module/Import System

While `external` functions can reference modules, there's no way to import WTLang code from other files. The `import` keyword shown in some documentation examples is **NOT implemented**.

### 6. Test Execution Not Implemented

`test` blocks are parsed but not executed. The `assert` statement shown in examples is **NOT implemented**.

### 7. No Table Literal Type

While you can write `{name: "Alice", age: 25}`, there's no direct way to create a single-row table from this without `table_from()`.

### 8. Limited Date Handling

Dates are parsed from CSV strings but there are no date manipulation functions or date literal syntax.

### 9. No Custom Operators

Users cannot define custom operators or override existing ones.

### 10. No Pattern Matching

Unlike languages like Rust or Haskell, WTLang doesn't support pattern matching in function parameters or case expressions.

---

## Grammar Summary (EBNF)

```ebnf
Program ::= ProgramItem*

ProgramItem ::=
    | TableDef
    | FunctionDef
    | ExternalFunction
    | Page
    | Test

TableDef ::= "table" Ident "{" Field* "}"

Field ::= Ident ":" Type ("[" Constraint ("," Constraint)* "]")?

Constraint ::=
    | "unique"
    | "non_null"
    | "validate" "(" Expr ")"
    | "references" Ident "." Ident

Type ::=
    | "int" | "float" | "string" | "date" | "currency" | "bool"
    | "table" ("<" Ident ">")?
    | "filter"

FunctionDef ::= "function" Ident "(" Params? ")" "->" Type "{" Statement* "}"

ExternalFunction ::= "external" "function" Ident "(" Params? ")" "->" Type "from" StringLit

Params ::= Param ("," Param)*

Param ::= Ident ":" Type

Page ::= "page" Ident "{" Statement* "}"

Test ::= "test" StringLit "{" Statement* "}"

Statement ::=
    | "title" StringLit
    | "subtitle" StringLit
    | "text" StringLit
    | "button" StringLit "{" Statement* "}"
    | "section" StringLit "{" Statement* "}"
    | "let" Ident (":" Type)? ("=" Expr)?
    | Ident "=" Expr
    | "if" Expr "{" Statement* "}" ("else" "{" Statement* "}")?
    | "forall" Ident "in" Expr "{" Statement* "}"
    | "return" Expr
    | Expr

Expr ::=
    | IntLit | FloatLit | StringLit | BoolLit
    | Ident
    | Expr BinOp Expr
    | UnOp Expr
    | Expr "." Ident
    | Expr "[" Expr "]"
    | Expr "->" Expr
    | "(" Params? ")" "=>" Expr
    | Ident "(" Args? ")"
    | "[" (Expr ("," Expr)*)? "]"
    | "{" (Ident ":" Expr ("," Ident ":" Expr)*)? "}"
    | "filter" "(" StringLit "," ("single" | "multi") ")"

BinOp ::= "+" | "-" | "*" | "/" | "%" 
        | "==" | "!=" | "<" | "<=" | ">" | ">=" 
        | "&&" | "||"

UnOp ::= "!" | "-"

Args ::= Expr ("," Expr)*
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | Nov 30, 2025 | Initial syntax reference based on current implementation |

---

## See Also

- [Language Design](language_design.md) - Design rationale and alternatives
- [Tutorial](tutorial.md) - Learn WTLang with examples
- [Error Codes](error_codes.md) - Complete error reference
- [Compiler Tools Design](compiler_tools_design.md) - Implementation architecture

---

**Note:** This document reflects the current implementation. Features marked as "NOT IMPLEMENTED" are documented in the tutorial but not yet available in the compiler.
