# WTLang Language Design Considerations

This document evaluates design alternatives for the WTLang domain-specific language and provides rationale for the chosen approach.

## 1. Type System

### Alternatives Considered

**A. Dynamic Typing**
- Pros: Faster prototyping, more flexible
- Cons: Runtime errors, poor tooling support, harder to maintain

**B. Strong Static Typing (Chosen)**
- Pros: Catches errors at compile time, better IDE support, self-documenting code
- Cons: More verbose, steeper learning curve

**C. Optional/Gradual Typing**
- Pros: Balance between flexibility and safety
- Cons: Complexity in implementation, inconsistent guarantees

### Rationale
Strong static typing was chosen because WTLang targets business users working with tables and financial data. Type safety prevents costly runtime errors in data manipulation and ensures data integrity. The trade-off of verbosity is acceptable given the domain's need for reliability.

## 2. Immutability of Tables

### Alternatives Considered

**A. Mutable Tables**
- Pros: Familiar to users from Excel/databases, potentially more efficient
- Cons: Side effects make reasoning difficult, harder to parallelize, debugging complexity

**B. Immutable Tables (Chosen)**
- Pros: Easier reasoning about code, no side effects, better for functional composition
- Cons: Potential performance overhead, mental shift for some users

**C. Copy-on-Write**
- Pros: Balance between mutability and immutability
- Cons: Implementation complexity, hidden performance characteristics

### Rationale
Immutability was chosen to align with functional programming principles and make code more predictable. Since WTLang compiles to Streamlit (which encourages stateless operations), immutability is a natural fit. The function chaining syntax makes working with immutable data intuitive.

## 3. Function Chaining Syntax

### Alternatives Considered

**A. Traditional Nested Function Calls**
```
filter(sort(table, "name"), row => row.age > 18)
```
- Pros: Familiar to many developers
- Cons: Hard to read with deep nesting, inside-out evaluation

**B. Method Chaining (Fluent Interface)**
```
table.sort("name").filter(row => row.age > 18)
```
- Pros: Readable, common in modern languages
- Cons: Requires tables to have methods, not easily extensible

**C. Pipeline Operator with Closures (Chosen)**
```
table -> sort(_, "name") -> filter(_, row => row.age > 18)
```
- Pros: Clear data flow, extensible, works with user-defined functions
- Cons: Less familiar syntax, requires understanding closures

### Rationale
The pipeline operator (`->`) with underscore placeholders provides the best balance of readability and flexibility. It allows users to compose operations naturally while supporting partial application. This syntax is particularly powerful for data transformation workflows common in table manipulation.

## 4. Syntax Style (Curly Braces vs. Indentation)

### Alternatives Considered

**A. Indentation-based (Python-style)**
- Pros: Cleaner appearance, forces consistent formatting
- Cons: Whitespace sensitivity, copy-paste errors, tooling challenges

**B. Curly Braces (Chosen)**
- Pros: Explicit scope, robust to formatting, familiar to C-family developers
- Cons: More verbose, possibility of inconsistent indentation

**C. Keywords (begin/end)**
- Pros: Very explicit, readable
- Cons: Very verbose, outdated style

### Rationale
Curly braces were chosen for robustness and clarity. Since WTLang will be used in business environments where code may be shared via email or documentation, explicit delimiters prevent formatting-related errors. The C-family syntax is also familiar to many developers.

## 5. Import System

### Alternatives Considered

**A. Relative Imports Only**
- Pros: Simple, explicit paths
- Cons: Fragile when moving files, verbose

**B. Module System with Package Names**
- Pros: Robust, scalable for large projects
- Cons: Overhead for small projects, requires package configuration

**C. Flexible Path-based Imports (Chosen)**
```
import table_utils from "./utils"
import validators from "../common/validators"
import standard.filters
```
- Pros: Intuitive for file organization, works for siblings/parents/children
- Cons: Can become complex in large hierarchies

### Rationale
A flexible path-based import system balances simplicity for small projects with scalability for larger ones. Supporting relative paths (sibling, parent, child directories) gives users organizational freedom while keeping the mental model simple.

## 6. Standard Library Design

### Alternatives Considered

**A. Minimal Standard Library**
- Pros: Small language core, encourages external functions
- Cons: Common operations require boilerplate

**B. Rich Standard Library (Chosen)**
- Pros: Productivity, consistency, less reliance on external code
- Cons: Larger language surface, maintenance burden

**C. Plugin-based Library**
- Pros: Extensible, community contributions
- Cons: Fragmentation, version management complexity

### Rationale
A rich standard library for common table operations (filtering, sorting, aggregation) was chosen to maximize productivity. Since table manipulation is the core domain, providing well-tested, optimized operations out-of-the-box reduces friction and ensures consistency.

## 7. Custom Type Definition

### Alternatives Considered

**A. Allow Custom Types**
- Pros: Flexibility, can model complex domains
- Cons: Increases language complexity, harder to compile to Streamlit

**B. No Custom Types (Chosen)**
- Pros: Simplicity, forces composition over inheritance
- Cons: Limited abstraction capabilities

**C. Type Aliases Only**
- Pros: Some abstraction without full custom types
- Cons: Limited value add, still adds complexity

### Rationale
Disallowing custom types keeps WTLang focused on its core purpose: table manipulation. Users can model complex data through table structures and relationships rather than custom type hierarchies. This constraint simplifies both the language and its compilation to Streamlit.

## 8. Validation and Constraints

### Alternatives Considered

**A. Runtime-only Validation**
- Pros: Flexible, easy to implement
- Cons: Errors caught late, performance overhead

**B. Compile-time and Runtime Validation (Chosen)**
```
table Users {
  id: int [unique, non_null]
  email: string [non_null, validate(is_valid_email)]
  age: int [validate(x => x >= 0 && x <= 150)]
}
```
- Pros: Early error detection, better documentation, optimizable
- Cons: More complex compiler

**C. Schema Definition Language**
- Pros: Declarative, potentially more powerful
- Cons: Two languages to learn, integration complexity

### Rationale
Integrated validation with both compile-time checks (for static constraints) and runtime checks (for dynamic validation) provides the best user experience. Constraints are part of the table definition, making data requirements explicit and self-documenting.

## 9. External Function Integration

### Alternatives Considered

**A. Foreign Function Interface (FFI)**
- Pros: High performance, tight integration
- Cons: Complex, platform-dependent, safety concerns

**B. Declaration-based Import (Chosen)**
```
external function analyze_sentiment(text: string) -> float from "nlp.sentiment"
```
- Pros: Type-safe, clear boundaries, easy to mock for testing
- Cons: Requires declaration overhead, less flexible

**C. Dynamic Import**
- Pros: No declarations needed, very flexible
- Cons: No type checking, runtime errors, poor tooling

### Rationale
Declaration-based imports provide type safety and clear contracts while allowing integration with Python functions. Users must declare external function signatures, which enables compile-time checking and better IDE support. This approach balances flexibility with safety.

## 10. Functions as First-Class Citizens

### Alternatives Considered

**A. Functions as Values (Chosen)**
```
let sorter = sort(_, "name")
let filter_adult = filter(_, row => row.age >= 18)
let pipeline = sorter -> filter_adult
```
- Pros: Composable, reusable, powerful abstractions
- Cons: Requires understanding of higher-order concepts

**B. Functions Only for Direct Calls**
- Pros: Simpler mental model
- Cons: Limited reusability, verbose code

**C. Macros/Templates**
- Pros: Code generation capabilities
- Cons: Complexity, harder to debug

### Rationale
First-class functions align with the functional programming paradigm and enable powerful composition patterns. The ability to store, pass, and manipulate function chains is essential for building reusable data transformation pipelines. While this requires more sophisticated language design, it significantly enhances expressiveness.

## 11. Testing Support

### Alternatives Considered

**A. No Built-in Testing Support**
- Pros: Simpler language, users choose their own tools
- Cons: Inconsistent testing approaches, poor testability

**B. External Testing Framework Only**
- Pros: Leverage existing tools (pytest for generated Python)
- Cons: Tests written in different language, no WTLang-specific assertions

**C. Built-in Testing Constructs (Chosen)**
```wtlang
test "filter removes rows correctly" {
  let users = table_from([
    {name: "Alice", age: 25},
    {name: "Bob", age: 17}
  ])
  let adults = users -> filter(_, row => row.age >= 18)
  
  assert adults.count() == 1
  assert adults[0].name == "Alice"
}

test "chain composition" {
  let pipeline = sort(_, "name") -> filter(_, row => row.age >= 18)
  let result = test_data -> pipeline
  
  assert_table_equals(result, expected_data)
}
```
- Pros: Tests in same language, domain-specific assertions, integrated tooling
- Cons: More language complexity, requires test runner

**D. Property-based Testing Support**
```wtlang
property "sorting preserves all rows" {
  forall table: Table<User> {
    let sorted = table -> sort(_, "name")
    assert sorted.count() == table.count()
  }
}
```
- Pros: Finds edge cases, formal verification
- Cons: Significant complexity, harder for non-experts

### Rationale
Built-in testing support is essential for a language targeting business applications where correctness is critical. Key features:

1. **Test Blocks**: First-class `test` keyword for defining test cases
2. **Assertions**: Domain-specific assertions for tables (`assert_table_equals`, `assert_contains`, etc.)
3. **Mock External Functions**: Easy mocking for external Python functions during testing
4. **Test Data Builders**: Utilities to create test tables quickly
5. **Coverage**: Compiler can track which code paths are tested

**Testing Philosophy:**
- **Unit Testing**: Test individual functions and transformations
- **Integration Testing**: Test complete page workflows
- **Mock External Functions**: Isolate WTLang logic from Python dependencies
- **Deterministic**: Immutability ensures tests are reproducible

**Example Test Structure:**
```wtlang
import users_module from "./users"

// Mock external function
mock external analyze_sentiment(text: string) -> float {
  if text == "good" return 0.8
  if text == "bad" return 0.2
  return 0.5
}

test "sentiment filter works" {
  let comments = table_from([
    {id: 1, text: "good"},
    {id: 2, text: "bad"}
  ])
  
  let positive = comments -> filter(_, row => analyze_sentiment(row.text) > 0.5)
  
  assert positive.count() == 1
  assert positive[0].id == 1
}
```

**Compiler Support:**
- `wtc test` command runs all tests in a project
- Tests compile to pytest-compatible Python tests
- IDE integration shows test results inline
- Coverage reports identify untested code paths

The immutability and pure function design of WTLang makes testing natural—functions always produce the same output for the same input, with no hidden state or side effects.

## 12. Variable Scoping and Type Annotations

### Overview

WTLang uses a scoping system that balances clarity with functionality. Each page acts as an independent module with its own variable namespace, while nested constructs (sections, buttons, conditionals) create child scopes that can access their parent's variables.

### Alternatives Considered

**A. Global Variables Only**
```wtlang
let global_users = load_csv("users.csv")

page Dashboard {
  show(global_users)
}

page Reports {
  show(global_users)  // Shared global state
}
```
- Pros: Simple, no scope tracking, easy sharing between pages
- Cons: Namespace pollution, hard to reason about, tight coupling, name conflicts

**B. File-level Scoping (Python-style)**
```wtlang
// All code in file shares one scope
let users = load_csv("users.csv")

page Dashboard {
  show(users)
}

page Reports {
  show(users)  // Same file, visible
}
```
- Pros: Familiar to Python developers, simple mental model
- Cons: Couples pages together, discourages file splitting, breaks modularity

**C. Function-level Scoping Only**
- Pros: Very simple, matches early JavaScript
- Cons: Too coarse-grained, confusing for nested blocks

**D. Page-level Scoping with Nested Scopes (Chosen)**
```wtlang
page Dashboard {
  let users = load_csv("users.csv")  // Page scope
  
  section "Summary" {
    let count = count(users)  // Section scope, sees 'users'
    text "Count: {count}"
  }
  
  // 'count' is NOT visible here
  show(users)  // OK: 'users' still in scope
}

page Reports {
  // 'users' from Dashboard NOT visible
  let reports = load_csv("reports.csv")  // New page scope
}
```
- Pros: Clean separation, matches Streamlit architecture, composable, prevents coupling
- Cons: More complex implementation, requires scope tracking

### Rationale

**Page-level scoping was chosen because:**

1. **Streamlit Alignment**: Each page compiles to a separate Python file, so independent scopes match the runtime model
2. **Modularity**: Pages are independent units that can be developed and tested separately
3. **Clarity**: Variables are scoped where they're used, making code easier to understand
4. **No Shared State**: Prevents accidental dependencies between pages
5. **Nested Composition**: Sections and buttons create child scopes for organized code

**Scoping Rules:**

- **Global Declarations**: Table definitions, functions, and external function declarations are visible everywhere
- **Page Scope**: Variables declared in a `page` block are visible throughout that page, but not in other pages
- **Nested Scopes**: Sections, buttons, if branches, and forall loops create child scopes
  - Child scopes can access parent scope variables
  - Variables declared in child scopes are not visible to parents or siblings
- **Block Structure**: `{ }` always creates a new scope (sections, buttons, conditionals, loops)

### Type Annotations

**Current Implicit Typing:**
```wtlang
let users = load_csv("users.csv", User)  // Type inferred from function
let count = 42  // Type inferred as int
let name = "Alice"  // Type inferred as string
```

**New Explicit Type Annotations (Chosen):**
```wtlang
// Colon syntax for type annotations
let users: table<User>
let count: int = 42
let total: float
let name: string = "Alice"

// Type inference still works when initializer is present
let data = load_csv("data.csv", Product)  // table<Product> inferred
let message = "Hello"  // string inferred
```

### Alternatives Considered for Type Annotations

**A. No Type Annotations (Current)**
- Pros: Less verbose, cleaner syntax
- Cons: Cannot declare variables without initialization, poor autocomplete, harder to catch type errors

**B. ML-style Type Inference Only**
- Pros: Clean, powerful
- Cons: Complex to implement, confusing error messages for non-experts

**C. Colon Notation (Chosen)**
```wtlang
let name: string
let count: int = 10
```
- Pros: Familiar to TypeScript/Kotlin/Swift users, clear and explicit, enables declaration without initialization
- Cons: More verbose than full inference

**D. Keyword-based**
```wtlang
let name as string
let count as int = 10
```
- Pros: More "natural language" feel
- Cons: Less familiar, uses extra keyword

### Rationale for Colon Notation

The colon syntax (`:`) for type annotations was chosen because:

1. **Familiarity**: Used by TypeScript, Kotlin, Swift, Python type hints, Rust
2. **Clarity**: Visually separates variable name from type
3. **Flexibility**: Works with or without initialization
4. **IDE Support**: Standard notation that LSP tools recognize
5. **Optional**: Can be omitted when type is obvious from initializer

### Declaration Without Initialization

**Problem:**
```wtlang
page Dashboard {
  let filtered_users  // ERROR in current design: must have initializer
  
  if some_condition {
    filtered_users = load_csv("users.csv")
  } else {
    filtered_users = load_csv("backup.csv")
  }
  
  show(filtered_users)
}
```

**Solution with Type Annotations:**
```wtlang
page Dashboard {
  let filtered_users: table<User>  // OK: type declared, no initializer needed
  
  if some_condition {
    filtered_users = load_csv("users.csv", User)
  } else {
    filtered_users = load_csv("backup.csv", User)
  }
  
  show(filtered_users)  // OK: definitely assigned by here
}
```

### Definite Assignment Analysis

The compiler tracks whether variables are initialized before use:

```wtlang
page Example {
  let total: float  // Declared but not initialized
  
  text "Total: {total}"  // ERROR: 'total' used before assignment
}
```

```wtlang
page Example {
  let total: float
  
  if condition {
    total = 100.0
  }
  // ERROR: 'total' not definitely assigned (what if condition is false?)
  
  text "Total: {total}"
}
```

```wtlang
page Example {
  let total: float
  
  if condition {
    total = 100.0
  } else {
    total = 50.0
  }
  // OK: 'total' assigned in all branches
  
  text "Total: {total}"
}
```

### Immutability and Reassignment

**Philosophy:**
WTLang follows functional programming principles with immutable data. However, for readability, we allow "single assignment" semantics:

```wtlang
page Example {
  let x: int
  x = 10  // OK: first assignment
  x = 20  // ERROR: cannot reassign variable
  
  // For tables (immutable), use transformation
  let users = load_csv("users.csv")
  let filtered = users -> filter(_, row => row.age > 18)  // New table
}
```

**Note**: While we use the term "reassignment," variables are actually assigned exactly once. The compiler enforces single-assignment semantics, ensuring functional purity while maintaining imperative syntax for conditional initialization.

### Scope Examples

**1. Page-level Variables:**
```wtlang
page Dashboard {
  let users = load_csv("users.csv")  // Visible throughout page
  
  text "User Count: {count(users)}"
  show(users)
}

page Reports {
  show(users)  // ERROR: 'users' not defined (different page scope)
}
```

**2. Nested Scopes:**
```wtlang
page Analysis {
  let data = load_csv("data.csv")
  
  section "Summary" {
    let total = sum(data, "amount")  // Section scope
    text "Total: {total}"
  }
  
  section "Details" {
    text "Total: {total}"  // ERROR: 'total' not in scope
    show(data)  // OK: 'data' is in parent (page) scope
  }
}
```

**3. Conditional Scopes:**
```wtlang
page Conditional {
  let users = load_csv("users.csv")
  let threshold = 100
  
  if count(users) > threshold {
    let message = "Many users!"  // If-branch scope
    text message
  } else {
    let message = "Few users"  // Separate else-branch scope
    text message
  }
  
  text message  // ERROR: 'message' not in scope (declared in branches)
}
```

**4. Loop Scopes:**
```wtlang
page Loop {
  let categories = ["A", "B", "C"]
  
  forall cat in categories {
    let filtered = products -> filter(_, row => row.category == cat)  // Loop scope
    section cat {
      show(filtered)
    }
  }
  
  show(filtered)  // ERROR: 'filtered' not in scope (declared in loop)
}
```

**5. Function Scope:**
```wtlang
function calculate_discount(price: float, customer_type: string) -> float {
  // Function parameters are scoped to the function body
  let discount_rate: float  // Local variable
  
  if customer_type == "premium" {
    discount_rate = 0.20
  } else {
    discount_rate = 0.10
  }
  
  return price * (1.0 - discount_rate)
}

page Checkout {
  let total = 100.0
  let final_price = calculate_discount(total, "premium")
  
  // ERROR: 'discount_rate' not in scope (function-local variable)
  // text "Rate: {discount_rate}"
  
  // ERROR: 'price' not in scope (function parameter)
  // text "Price: {price}"
}
```

**Function scope rules:**
- Function signatures (names and parameter types) are global
- Parameters are visible only within the function body
- Local variables declared in function are not visible outside
- Functions can reference global tables and other global functions
- Functions cannot access page-level or section-level variables

### Benefits

**For Users:**
- Clear variable lifetime and visibility
- No confusion about variable availability
- Better error messages ("not in scope" vs "not defined")
- Encourages modular, organized code

**For IDE/LSP:**
- Accurate autocomplete (only show in-scope variables)
- Precise go-to-definition
- Better refactoring support
- Scope-aware hover information

**For Compiler:**
- Easier code generation (scopes map to Python scopes)
- Better optimization opportunities
- Definite assignment analysis
- Type checking with context

### Implementation Considerations

**Symbol Table Structure:**
- Global scope: tables, function signatures, external functions
- Function scope: function parameters and local variables (separate for each function)
- Page scope: variables declared in page blocks
- Nested scopes: sections, buttons, if branches, loops
- Each scope has a parent reference for lookups

**Type Checking:**
- Infer types from initializers when annotations absent
- Check type consistency when annotations present
- Verify definite assignment before variable use
- Enforce immutability (single assignment)

**Error Messages:**
```
Error: Undefined variable 'count'
  ┌─ dashboard.wt:15:8
  │
15│   show(count)
  │        ^^^^^ not found in this scope
  │
  = note: 'count' was defined in section scope at line 10
  = help: move this statement inside the section, or define 'count' at page level
```

## Summary

WTLang's design prioritizes:
1. **Safety** through strong static typing, immutability, and explicit scoping
2. **Readability** through pipeline syntax, type annotations, and clear scope boundaries
3. **Productivity** through rich standard library, function composition, and intelligent type inference
4. **Focus** by constraining to the table manipulation domain
5. **Testability** through built-in testing constructs and deterministic behavior
6. **Modularity** through page-level scoping that matches the Streamlit deployment model

Key design decisions:
- **Page-level scoping**: Clean separation between pages, prevents coupling
- **Type annotations**: Optional but encouraged, using familiar colon (`:`) syntax
- **Definite assignment**: Variables must be initialized in all code paths before use
- **Immutability**: Single-assignment semantics ensure functional purity
- **Nested scopes**: Sections, buttons, and control flow create composable scopes

These choices create a language that is both powerful for its intended use case and accessible to users who may not be professional developers but need to work with tabular data in web applications.
