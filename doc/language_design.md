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

## Summary

WTLang's design prioritizes:
1. **Safety** through strong static typing and immutability
2. **Readability** through pipeline syntax and explicit scoping
3. **Productivity** through rich standard library and function composition
4. **Focus** by constraining to the table manipulation domain

These choices create a language that is both powerful for its intended use case and accessible to users who may not be professional developers but need to work with tabular data in web applications.
