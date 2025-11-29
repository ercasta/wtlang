# WTLang

WTLang is a Domain Specific Language to create web pages that show and allow editing tables. This documents is a blueprint for creating the language

## Characteristics of the language

- The language allows to define multiple pages
- Each page contains one or more table
- The language has keywords to represent pages, title, subtitles, table showing sections, buttons to performs save / export operations.
- The source code can be split in multiple files, with an "import" statement that allows importing symbols from sibling / parent / files in children folders
- The tables can be exported to excel
- The language provides a standard library with common operations such as filtering, sorting, etc.
- The language has constructs for if statements and "iteration" statements (e.g. forall) 
- The language has scalar types (int, float, string, date, currency) and a "table" type that allows defining the structure of a table. Moreover it has a "function chain" type (more on this laters)
- Within table definition, reference from a field to another table is made explicit, to allow creating a hierarchy of tables
- The language allows calling external table generation / manipulation functions created in other languages. These external functions must be "imported" in terms of declaration
- The language does not allow defining custom types
- The language is strongly typed
- The language allows creating user defined functions, with multiple parameters and one return value
- Function calling does not modify parameters. Tables are immutable: language statements and functions create new tables without modifying the original one
- Function chaining uses a special syntax (->) to easily allow concatenating functions without lots of nesting. When using chaining, the number and type of output parameters from the previous function must match the number and type parameters of the following one. Some parameters can be provided upfront to create a "closure", with unbounded parameters represented by an underscore.
- Function chains can be defined once and extended appending new functions to the chain, or substituting some elements of the chain (eg. chain[2] = filterfuntion)
- Functions and chains are first class citizens
- The language syntax is based on curly brackets to define code blocks. It does not use spaces or delimiters (e.g. semicolon) to separate statements.
- The language allows defining validity checks and rules for table content, such as unique - non null constraints on fields, or more complex, user defined validation functions.


## Target platform

- WTLang compiles to Streamlit pages 
- Other target platforms might be added in the future
- The external functions that allow manipulating tables are written in python

## Characteristics of the compiler and related tools

- The compiler shall be fast, so languages such as Rust for the compiler are used
- Proper support tooling is needed, in particular a Language Server for VSCode, based on Language Server Protocol
- Support tools must also allow autocompletion of external functions.



