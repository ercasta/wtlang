# WTLang Tutorial

Welcome to WTLang! This tutorial will teach you how to create interactive web applications for displaying and editing tables. WTLang is designed to be simple yet powerful, letting you focus on your data rather than web development complexity.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Understanding Variables and Scoping](#understanding-variables-and-scoping)
   - [Variable Declaration](#variable-declaration)
   - [Type Annotations](#type-annotations)
   - [Page-Level Scoping](#page-level-scoping)
   - [Nested Scopes](#nested-scopes)
   - [Variable Initialization Rules](#variable-initialization-rules)
   - [Immutability](#immutability)
   - [Function Scope](#function-scope)
   - [Global Declarations](#global-declarations)
   - [Best Practices](#best-practices)
3. [Part 1: Data Presentation](#part-1-data-presentation)
   - [Creating Your First Page](#creating-your-first-page)
   - [Working with Tables](#working-with-tables)
   - [Multiple Pages](#multiple-pages)
   - [Adding Interactivity](#adding-interactivity)
4. [Part 2: Data Manipulation](#part-2-data-manipulation)
   - [Functions and Transformations](#functions-and-transformations)
   - [Function Chaining](#function-chaining)
   - [User-Defined Functions](#user-defined-functions)
   - [External Functions](#external-functions)
5. [Advanced Topics](#advanced-topics)
6. [Complete Example](#complete-example)

---

## Getting Started

### Installation

```bash
# Install WTLang compiler
cargo install wtlang

# Verify installation
wtc --version
```

### Your First WTLang Project

```bash
# Create a new project
wtc init my-first-app
cd my-first-app

# Project structure:
# my-first-app/
#   ├── src/
#   │   └── main.wt
#   └── wtlang.toml
```

---

## Understanding Variables and Scoping

Before diving into data presentation, it's important to understand how variables work in WTLang.

### Variable Declaration

Variables are declared using the `let` keyword:

```wtlang
page Example {
  title "Variables Example"
  
  // Type is inferred from the value
  let name = "Alice"          // string
  let age = 25                // int
  let price = 99.99           // float
  let active = true           // bool
}
```

### Type Annotations

You can optionally specify the type explicitly using a colon (`:`):

```wtlang
page TypedExample {
  title "Type Annotations"
  
  // Explicit type annotation
  let count: int = 10
  let name: string = "Bob"
  let total: float = 123.45
  
  // Type annotation without immediate initialization
  let result: float
  
  if some_condition {
    result = 100.0
  } else {
    result = 50.0
  }
  
  text "Result: {result}"
}
```

**When to use type annotations:**
- When declaring a variable without an initial value
- To make code more self-documenting
- When the type isn't obvious from the initializer
- For better IDE autocomplete and error checking

### Page-Level Scoping

**Each page has its own independent variable scope:**

```wtlang
page Dashboard {
  let users = load_csv("users.csv", User)  // Only visible in Dashboard
  show(users)
}

page Reports {
  // ERROR: 'users' is not defined here (different page)
  // show(users)
  
  // Must load data again for this page
  let reports = load_csv("reports.csv", Report)
  show(reports)
}
```

**Why page-level scoping?**
- Each page is compiled to a separate Python file
- Pages are independent and don't share state
- Prevents coupling between different parts of your application
- Makes code easier to understand and maintain

### Nested Scopes

Sections, buttons, and control flow create child scopes that can access parent variables:

```wtlang
page Analysis {
  let data = load_csv("data.csv", Data)     // Page scope
  
  section "Summary" {
    let total = sum(data, "amount")          // Section scope
    text "Total: {total}"                    // Can use 'total' here
  }
  
  section "Details" {
    // ERROR: 'total' is not in scope (defined in other section)
    // text "Total: {total}"
    
    // But 'data' is available (parent page scope)
    show(data)                               // OK
  }
  
  button "Export" {
    let filename = "export.xlsx"             // Button scope
    export_excel(data, filename)             // Can use 'data' from page scope
  }
  
  // 'filename' is NOT visible here (button scope ended)
}
```

**Scope rules:**
- Child scopes can access variables from parent scopes
- Variables in child scopes are NOT visible to parents or siblings
- Each `{ }` block creates a new scope

### Conditional Scopes

```wtlang
page Conditional {
  let total = 1000
  let threshold = 500
  
  if total > threshold {
    let message = "High value!"              // Only in if-branch
    text message
  } else {
    let message = "Low value"                // Separate else-branch scope
    text message
  }
  
  // ERROR: 'message' not in scope here
  // text message
}
```

### Loop Scopes

```wtlang
page Loop {
  let categories = ["Electronics", "Books", "Clothing"]
  
  forall category in categories {
    // 'category' is available in loop body
    let filtered = products -> filter(_, row => row.category == category)
    
    section category {
      show(filtered)
    }
  }
  
  // ERROR: 'category' and 'filtered' not in scope here
}
```

### Variable Initialization Rules

Variables must be initialized before use:

```wtlang
page Initialization {
  let result: float       // Declared but not initialized
  
  // ERROR: cannot use 'result' yet
  // text "Result: {result}"
  
  result = 100.0          // Now initialized
  text "Result: {result}" // OK
}
```

For conditional initialization, all branches must assign a value:

```wtlang
page ConditionalInit {
  let value: float
  
  if condition {
    value = 100.0
  } else {
    value = 50.0
  }
  // OK: 'value' assigned in both branches
  
  text "Value: {value}"
}
```

```wtlang
page IncompleteInit {
  let value: float
  
  if condition {
    value = 100.0
  }
  // ERROR: 'value' might not be assigned (no else branch)
  
  text "Value: {value}"
}
```

### Immutability

Variables in WTLang follow single-assignment semantics:

```wtlang
page Immutable {
  let count = 10
  // count = 20           // ERROR: cannot reassign
  
  // Instead, create a new variable
  let new_count = count + 10
  
  // For tables, transformations create new tables
  let users = load_csv("users.csv", User)
  let filtered = users -> filter(_, row => row.age > 18)  // New table
}
```

**Note:** While WTLang is immutable, the conditional initialization pattern allows a single assignment after declaration, making the code more readable while maintaining functional purity.

### Function Scope

Function bodies have their own independent scope, separate from pages:

```wtlang
function calculate_total(items: table<Item>) -> float {
  // Function parameters are only visible in the function body
  // 'items' is a parameter, scoped to this function
  
  let sum: float = 0.0  // Local variable, only visible in function
  
  forall item in items {
    sum = sum + item.price * item.quantity
  }
  
  return sum
}

page Invoice {
  let cart_items = load_csv("cart.csv", Item)
  let total = calculate_total(cart_items)  // Call the function
  
  text "Total: ${total}"
  
  // ERROR: 'sum' not in scope (it's local to the function)
  // text "Sum: {sum}"
  
  // ERROR: 'items' not in scope (it's a function parameter)
  // show(items)
}
```

**Key points about function scope:**
- Function parameters are only accessible within that function's body
- Variables declared inside a function are local to that function
- Functions can call other global functions and reference global tables
- Functions cannot access variables from pages or other functions
- Each function call creates a new scope instance

### Global Declarations

Table definitions, function signatures, and external function declarations are global and visible everywhere:

```wtlang
// Global table definition
table User {
  id: int [unique, non_null]
  name: string
  age: int
  email: string
}

// Global function - signature is global, body has its own scope
function is_adult(user: User) -> bool {
  let threshold = 18  // Local to function, not visible outside
  return user.age >= threshold
}

// Both pages can use the table and call the function
page Dashboard {
  let users = load_csv("users.csv", User)  // Uses global table
  let adults = users -> filter(_, is_adult)  // Calls global function
  show(adults)
}

page Reports {
  let users = load_csv("users.csv", User)  // Same global table
  let adult_users = users -> filter(_, is_adult)  // Same global function
  
  // 'threshold' from is_adult is NOT visible here (function-local)
  show(adult_users)
}
```

**What's global:**
- Table type definitions (can be used anywhere)
- Function names and signatures (can be called anywhere)
- External function declarations

**What's NOT global:**
- Function parameters (scoped to function body)
- Local variables in functions (scoped to function body)
- Page variables (scoped to that specific page)
- Variables in sections, buttons, etc. (scoped to their blocks)

### Best Practices

1. **Declare variables close to where they're used**
   ```wtlang
   section "Summary" {
     let total = sum(data, "amount")  // Declared in section where it's needed
     text "Total: {total}"
   }
   ```

2. **Use descriptive names**
   ```wtlang
   let active_users = users -> filter(_, row => row.active)  // Clear
   let x = users -> filter(_, row => row.active)             // Unclear
   ```

3. **Add type annotations for clarity**
   ```wtlang
   let total_revenue: currency  // Makes intent clear
   let user_count: int
   ```

4. **Keep page scopes independent**
   - Don't rely on shared global variables between pages
   - Each page should load its own data
   - Use functions to share logic, not variables

---

# Part 1: Data Presentation

This section covers how to create pages and display data to users.

## Creating Your First Page

Let's create a simple page that displays a welcome message and a table of users.

**src/main.wt:**
```wtlang
page Home {
  title "User Management"
  subtitle "Welcome to your first WTLang application!"
  
  table Users {
    id: int [unique, non_null]
    name: string [non_null]
    email: string [non_null]
    age: int
  }
  
  show Users
}
```

**Compile and run:**
```bash
wtc build src/main.wt --output dist/
cd dist
streamlit run Home.py
```

### Understanding the Code

- **`page Home { }`**: Defines a page named "Home"
- **`title`**: Sets the main heading
- **`subtitle`**: Sets a descriptive subheading
- **`table Users { }`**: Defines the structure of a table
- **`show Users`**: Displays the table on the page

### Table Field Types

WTLang supports these scalar types:
- **`int`**: Integer numbers (e.g., 42, -10)
- **`float`**: Decimal numbers (e.g., 3.14, -0.5)
- **`string`**: Text (e.g., "Hello", "alice@example.com")
- **`date`**: Dates (e.g., 2025-11-29)
- **`currency`**: Monetary values (e.g., $100.50)

### Field Constraints

You can add constraints to fields:
- **`unique`**: Values must be unique across rows
- **`non_null`**: Field cannot be empty
- **`validate(function)`**: Custom validation function

**Example:**
```wtlang
table Products {
  sku: string [unique, non_null]
  name: string [non_null]
  price: currency [non_null, validate(x => x > 0)]
  stock: int [validate(x => x >= 0)]
}
```

## Working with Tables

### Loading Data from Files

```wtlang
page Inventory {
  title "Product Inventory"
  
  table Products {
    sku: string [unique, non_null]
    name: string
    price: currency
    stock: int
  }
  
  // Load data from CSV file
  let products = load_csv("data/products.csv", Products)
  
  show products
}
```

### Creating Tables Programmatically

```wtlang
page Dashboard {
  title "Sales Dashboard"
  
  table Sale {
    id: int
    product: string
    amount: currency
    date: date
  }
  
  // Create table from literal data
  let sales = table_from([
    {id: 1, product: "Widget", amount: $25.50, date: 2025-11-01},
    {id: 2, product: "Gadget", amount: $42.00, date: 2025-11-15},
    {id: 3, product: "Widget", amount: $25.50, date: 2025-11-20}
  ])
  
  show sales
}
```

### Editable Tables

Make tables interactive by allowing users to edit data:

```wtlang
page UserEditor {
  title "Edit Users"
  
  table User {
    id: int [unique, non_null]
    name: string [non_null]
    email: string [non_null]
    active: bool
  }
  
  let users = load_csv("users.csv", User)
  
  // Enable editing
  show_editable users
  
  // Save button
  button "Save Changes" {
    save_csv(users, "users.csv")
  }
}
```

### Table Relationships

Define relationships between tables:

```wtlang
table Department {
  dept_id: int [unique, non_null]
  name: string [non_null]
}

table Employee {
  emp_id: int [unique, non_null]
  name: string [non_null]
  department: int [references Department.dept_id]
  salary: currency
}
```

The `references` keyword creates a foreign key relationship, which can be used for validation and joins.

## Multiple Pages

Create applications with multiple pages for better organization:

**src/main.wt:**
```wtlang
// Define shared table types
table User {
  id: int [unique, non_null]
  name: string [non_null]
  email: string
}

page Home {
  title "Dashboard"
  
  let users = load_csv("data/users.csv", User)
  let user_count = users.count()
  
  text "Total Users: {user_count}"
  
  show users
}

page UserManagement {
  title "User Management"
  
  let users = load_csv("data/users.csv", User)
  
  show_editable users
  
  button "Save" {
    save_csv(users, "data/users.csv")
  }
  
  button "Export to Excel" {
    export_excel(users, "users.xlsx")
  }
}

page Reports {
  title "User Reports"
  
  let users = load_csv("data/users.csv", User)
  
  // Show summary statistics
  text "Statistics coming soon..."
}
```

**Compile:**
```bash
wtc build src/main.wt --output dist/
```

This creates three separate Streamlit pages that users can navigate between.

## Adding Interactivity

### Buttons and Actions

```wtlang
page DataManager {
  title "Data Manager"
  
  table Item {
    id: int
    name: string
    quantity: int
  }
  
  let items = load_csv("items.csv", Item)
  
  show_editable items
  
  // Multiple buttons
  button "Save" {
    save_csv(items, "items.csv")
  }
  
  button "Export Excel" {
    export_excel(items, "items_export.xlsx")
  }
  
  button "Export CSV" {
    export_csv(items, "items_export.csv")
  }
}
```

### Conditional Display

```wtlang
page ConditionalView {
  title "Conditional Display"
  
  table Order {
    id: int
    status: string
    amount: currency
  }
  
  let orders = load_csv("orders.csv", Order)
  let total_amount = sum(orders, "amount")
  
  // Show different content based on conditions
  if total_amount > $10000 {
    text "High value orders detected!"
    show orders
  } else {
    text "Normal order volume"
  }
}
```

### Sections and Layout

```wtlang
page Dashboard {
  title "Sales Dashboard"
  
  section "Summary" {
    let sales = load_csv("sales.csv", Sale)
    let total = sum(sales, "amount")
    
    text "Total Sales: {total}"
  }
  
  section "Recent Orders" {
    let orders = load_csv("orders.csv", Order)
    show orders
  }
  
  section "Top Products" {
    let products = load_csv("products.csv", Product)
    show products
  }
}
```

---

# Part 2: Data Manipulation

This section covers how to transform and process data using functions and chains.

## Functions and Transformations

WTLang provides a rich standard library for data manipulation.

### Filtering

Filter rows based on conditions:

```wtlang
page ActiveUsers {
  title "Active Users"
  
  table User {
    id: int
    name: string
    active: bool
    age: int
  }
  
  let all_users = load_csv("users.csv", User)
  
  // Filter active users
  let active_users = filter(all_users, row => row.active == true)
  
  // Filter by age
  let adults = filter(all_users, row => row.age >= 18)
  
  // Multiple conditions
  let active_adults = filter(all_users, row => row.active == true && row.age >= 18)
  
  show active_adults
}
```

### Sorting

Sort tables by one or more columns:

```wtlang
page SortedProducts {
  title "Products by Price"
  
  table Product {
    name: string
    price: currency
    stock: int
  }
  
  let products = load_csv("products.csv", Product)
  
  // Sort by price (ascending)
  let by_price = sort(products, "price")
  
  // Sort by price (descending)
  let by_price_desc = sort_desc(products, "price")
  
  // Sort by multiple columns
  let sorted = sort(products, ["price", "name"])
  
  show sorted
}
```

### Aggregation

Calculate summary statistics:

```wtlang
page SalesSummary {
  title "Sales Summary"
  
  table Sale {
    product: string
    amount: currency
    quantity: int
    date: date
  }
  
  let sales = load_csv("sales.csv", Sale)
  
  // Sum of all amounts
  let total_revenue = sum(sales, "amount")
  
  // Average sale amount
  let avg_sale = average(sales, "amount")
  
  // Count rows
  let sale_count = count(sales)
  
  // Min and max
  let min_sale = min(sales, "amount")
  let max_sale = max(sales, "amount")
  
  text "Total Revenue: {total_revenue}"
  text "Average Sale: {avg_sale}"
  text "Number of Sales: {sale_count}"
  text "Smallest Sale: {min_sale}"
  text "Largest Sale: {max_sale}"
}
```

### Grouping and Aggregation

Group data and calculate aggregates per group:

```wtlang
page SalesByProduct {
  title "Sales by Product"
  
  table Sale {
    product: string
    amount: currency
    quantity: int
  }
  
  let sales = load_csv("sales.csv", Sale)
  
  // Group by product and sum amounts
  let product_totals = group_by(sales, "product", {
    total_amount: sum("amount"),
    total_quantity: sum("quantity"),
    avg_price: average("amount")
  })
  
  show product_totals
}
```

### Joining Tables

Combine data from multiple tables:

```wtlang
page OrderDetails {
  title "Order Details with Customer Info"
  
  table Customer {
    customer_id: int
    name: string
    email: string
  }
  
  table Order {
    order_id: int
    customer_id: int
    amount: currency
    date: date
  }
  
  let customers = load_csv("customers.csv", Customer)
  let orders = load_csv("orders.csv", Order)
  
  // Inner join
  let order_details = join(
    orders, 
    customers, 
    on: (order, customer) => order.customer_id == customer.customer_id
  )
  
  show order_details
}
```

### Selecting Columns

Choose which columns to display:

```wtlang
page UserSummary {
  title "User Summary"
  
  table User {
    id: int
    name: string
    email: string
    password_hash: string
    created_at: date
  }
  
  let users = load_csv("users.csv", User)
  
  // Select only specific columns (hide password_hash)
  let public_users = select(users, ["id", "name", "email", "created_at"])
  
  show public_users
}
```

### Adding Computed Columns

Create new columns based on existing data:

```wtlang
page EnrichedProducts {
  title "Products with Calculated Fields"
  
  table Product {
    name: string
    cost: currency
    price: currency
    stock: int
  }
  
  let products = load_csv("products.csv", Product)
  
  // Add profit margin column
  let with_margin = add_column(
    products, 
    "margin", 
    row => (row.price - row.cost) / row.price * 100
  )
  
  // Add inventory value
  let with_value = add_column(
    with_margin,
    "inventory_value",
    row => row.price * row.stock
  )
  
  show with_value
}
```

## Function Chaining

Function chaining makes complex transformations readable by using the `->` operator.

### Basic Chaining

Instead of nesting functions, chain them:

```wtlang
page ChainExample {
  title "Function Chaining Demo"
  
  table Sale {
    product: string
    amount: currency
    date: date
  }
  
  let sales = load_csv("sales.csv", Sale)
  
  // Without chaining (nested)
  let result1 = sort(filter(sales, row => row.amount > $100), "date")
  
  // With chaining (readable)
  let result2 = sales 
    -> filter(_, row => row.amount > $100)
    -> sort(_, "date")
  
  show result2
}
```

The underscore `_` represents where the previous result is inserted.

### Complex Chains

Build sophisticated data pipelines:

```wtlang
page SalesAnalysis {
  title "Sales Analysis Pipeline"
  
  table Sale {
    product: string
    category: string
    amount: currency
    quantity: int
    date: date
    region: string
  }
  
  let sales = load_csv("sales.csv", Sale)
  
  // Complex transformation pipeline
  let analysis = sales
    -> filter(_, row => row.date >= 2025-01-01)  // This year only
    -> filter(_, row => row.region == "West")     // West region only
    -> add_column(_, "revenue", row => row.amount * row.quantity)
    -> group_by(_, "category", {
         total_revenue: sum("revenue"),
         total_quantity: sum("quantity"),
         avg_price: average("amount")
       })
    -> sort_desc(_, "total_revenue")
    -> limit(_, 10)  // Top 10 categories
  
  show analysis
}
```

### Reusable Chains

Define chains once and reuse them:

```wtlang
page ReusableChains {
  title "Reusable Data Pipelines"
  
  table Order {
    id: int
    customer: string
    amount: currency
    status: string
    date: date
  }
  
  let orders = load_csv("orders.csv", Order)
  
  // Define a reusable chain
  let recent_completed = filter(_, row => row.status == "completed")
    -> filter(_, row => row.date >= 2025-11-01)
    -> sort_desc(_, "amount")
  
  // Apply the chain
  let results = orders -> recent_completed
  
  show results
  
  // Can modify chains
  let top_5_recent = recent_completed -> limit(_, 5)
  let more_results = orders -> top_5_recent
}
```

### Partial Application

Create specialized functions by pre-filling some parameters:

```wtlang
page PartialApplication {
  title "Partial Application Examples"
  
  table Product {
    name: string
    category: string
    price: currency
  }
  
  let products = load_csv("products.csv", Product)
  
  // Create specialized filter functions
  let filter_expensive = filter(_, row => row.price > $100)
  let filter_electronics = filter(_, row => row.category == "Electronics")
  let sort_by_price = sort(_, "price")
  
  // Compose them
  let expensive_electronics = products
    -> filter_expensive
    -> filter_electronics
    -> sort_by_price
  
  show expensive_electronics
}
```

### Chain Substitution

Modify parts of a chain dynamically:

```wtlang
page ChainModification {
  title "Dynamic Chain Modification"
  
  table Data {
    id: int
    value: float
    category: string
  }
  
  let data = load_csv("data.csv", Data)
  
  // Define a chain as an array
  let pipeline = [
    filter(_, row => row.value > 0),
    sort(_, "value"),
    limit(_, 100)
  ]
  
  // Replace the limit step
  pipeline[2] = limit(_, 50)
  
  // Apply modified chain
  let result = apply_chain(data, pipeline)
  
  show result
}
```

## Query Language

WTLang provides a concise query language for working with tables, making data manipulation more natural and readable.

### Filtering with WHERE

Use the `where` keyword with infix notation to filter table rows:

```wtlang
page FilteringExamples {
  title "WHERE Clause Examples"
  
  table User {
    id: int
    name: string
    age: int
    subscription: string
  }
  
  let users = load_csv("users.csv", User)
  
  // Simple filter
  let adults = users where age >= 18
  show(adults)
  
  // Complex conditions with parentheses
  let young_premium = users where (age < 30 and subscription == "premium")
  show(young_premium)
  
  // Multiple filters can be chained
  let active_seniors = users 
    where age >= 65 
    where subscription != "inactive"
  show(active_seniors)
}
```

### Sorting with SORT BY

Sort tables using the `sort by` syntax with ascending (`asc`) or descending (`desc`) order:

```wtlang
page SortingExamples {
  title "Sorting Examples"
  
  table Product {
    name: string
    category: string
    price: currency
    stock: int
  }
  
  let products = load_csv("products.csv", Product)
  
  // Sort by single column (ascending)
  let by_name = products sort by name asc
  show(by_name)
  
  // Sort descending
  let by_price_desc = products sort by price desc
  show(by_price_desc)
  
  // Multi-column sort
  let sorted = products sort by category asc, price desc
  show(sorted)
}
```

### Column Selection

Select specific columns from a table using bracket notation:

```wtlang
page ColumnSelection {
  title "Column Selection"
  
  table Employee {
    id: int
    name: string
    department: string
    salary: currency
    email: string
  }
  
  let employees = load_csv("employees.csv", Employee)
  
  // Select just name and department
  let names = employees[name, department]
  show(names)
  
  // Combine with filtering
  let it_emails = (employees where department == "IT")[name, email]
  show(it_emails)
}
```

### Set Operations

Perform set operations on tables with compatible schemas:

```wtlang
page SetOperations {
  title "Set Operations on Tables"
  
  table Person {
    id: int
    name: string
    age: int
  }
  
  let all_people = load_csv("people.csv", Person)
  
  // Create subsets
  let seniors = all_people where age >= 65
  let minors = all_people where age < 18
  let adults = all_people where (age >= 18 and age < 65)
  
  // Union: combine tables (removes duplicates)
  let special_rates = seniors + minors
  show(special_rates)
  
  // Difference: rows in first table but not in second
  let regular_customers = all_people - special_rates
  show(regular_customers)
  
  // Intersection: rows that appear in both tables
  let active_adults = adults & regular_customers
  show(active_adults)
}
```

### Combining Query Operations

All query operations can be combined for powerful data manipulation:

```wtlang
page CombinedQueries {
  title "Combined Query Operations"
  
  table Order {
    order_id: int
    customer_name: string
    product: string
    amount: currency
    status: string
  }
  
  let orders = load_csv("orders.csv", Order)
  
  // Filter, select columns, and sort
  let summary = orders
    where status == "completed"
    where amount > $100
    sort by amount desc
  let final = summary[customer_name, product, amount]
  show(final)
}
```

## Table References and Keys

Define relationships between tables using primary keys and reference types.

### Primary Keys

Mark a table field as the primary key using the `key` constraint:

```wtlang
table Department {
  code: string [key]  // Primary key
  name: string
  budget: currency
}

table Category {
  id: int [key]  // Primary key
  name: string
  description: string
}
```

**Key constraints ensure:**
- Each row has a unique identifier
- Only one field per table can be marked as `key`
- Referenced tables must have a key field

### Reference Types

Create references to other tables using the `ref` type:

```wtlang
table Employee {
  emp_id: int [key]
  name: string
  dept: ref Department  // References Department table
  salary: currency
}

table Product {
  id: int [key]
  name: string
  category: ref Category  // References Category table
  price: currency
}
```

**Reference types provide:**
- Type-safe references to other tables
- Automatic validation (referenced table must exist and have a key)
- Navigation from one table to another

### Navigating References

Access referenced table data directly through reference fields:

```wtlang
page ReferenceNavigation {
  title "Working with References"
  
  table Department {
    code: string [key]
    name: string
    budget: currency
  }
  
  table Employee {
    emp_id: int [key]
    name: string
    dept: ref Department
    salary: currency
  }
  
  let departments = load_csv("departments.csv", Department)
  let employees = load_csv("employees.csv", Employee)
  
  section "All Employees" {
    show(employees)
  }
  
  section "Department Information" {
    // Navigate to departments through reference
    // This automatically performs a lookup/join
    let dept_info = employees.dept
    show(dept_info)
  }
  
  section "Filtered by Department" {
    // Can filter using referenced table fields
    let it_staff = employees where dept.name == "IT"
    show(it_staff)
  }
}
```

**How reference navigation works:**
1. The compiler detects field access on a reference type
2. Generates code to perform a lookup/join operation
3. Returns the referenced table row(s)

### Best Practices for Keys and References

1. **Use meaningful primary keys** - Keys should be stable identifiers
   ```wtlang
   // Good: stable, meaningful ID
   table User {
     user_id: int [key]
     email: string
   }
   
   // Avoid: using mutable fields as keys
   table User {
     email: string [key]  // Email might change!
   }
   ```

2. **Reference validation** - The compiler validates references at compile time:
   ```wtlang
   table Employee {
     dept: ref Department  // Error if Department doesn't exist
   }
   ```

3. **One key per table** - Only one field can be marked as `key`:
   ```wtlang
   table Product {
     id: int [key]
     sku: string [key]  // Error: multiple keys not allowed
   }
   ```

## User-Defined Functions

Create your own functions for reusable logic.

### Basic Functions

```wtlang
page CustomFunctions {
  title "Custom Functions"
  
  table Sale {
    amount: currency
    tax_rate: float
  }
  
  // Define a function
  function calculate_total(sale_amount: currency, tax_rate: float) -> currency {
    return sale_amount * (1 + tax_rate)
  }
  
  let sales = load_csv("sales.csv", Sale)
  
  // Use the function
  let with_totals = add_column(
    sales,
    "total",
    row => calculate_total(row.amount, row.tax_rate)
  )
  
  show with_totals
}
```

### Functions with Table Parameters

```wtlang
page TableFunctions {
  title "Functions Operating on Tables"
  
  table Order {
    id: int
    amount: currency
    status: string
  }
  
  // Function that processes a table
  function get_completed_orders(orders: Table<Order>) -> Table<Order> {
    return orders -> filter(_, row => row.status == "completed")
  }
  
  function calculate_revenue(orders: Table<Order>) -> currency {
    let completed = get_completed_orders(orders)
    return sum(completed, "amount")
  }
  
  let orders = load_csv("orders.csv", Order)
  let revenue = calculate_revenue(orders)
  
  text "Total Revenue: {revenue}"
  show get_completed_orders(orders)
}
```

### Functions Returning Functions

Create function factories:

```wtlang
page FunctionFactories {
  title "Higher-Order Functions"
  
  table Product {
    name: string
    price: currency
  }
  
  // Function that returns a filter function
  function create_price_filter(min_price: currency) -> function(Table<Product>) -> Table<Product> {
    return filter(_, row => row.price >= min_price)
  }
  
  let products = load_csv("products.csv", Product)
  
  // Create specialized filters
  let filter_premium = create_price_filter($100)
  let filter_budget = create_price_filter($20)
  
  let premium_products = products -> filter_premium
  let budget_products = products -> filter_budget
  
  section "Premium Products" {
    show premium_products
  }
  
  section "Budget Products" {
    show budget_products
  }
}
```

## External Functions

Integrate Python functions for advanced processing.

### Declaring External Functions

```wtlang
// Declare a Python function
external function analyze_sentiment(text: string) -> float 
  from "analytics.sentiment"

external function classify_category(description: string) -> string
  from "ml.classifier"

page SentimentAnalysis {
  title "Product Review Sentiment"
  
  table Review {
    id: int
    product: string
    comment: string
    rating: int
  }
  
  let reviews = load_csv("reviews.csv", Review)
  
  // Use external function
  let with_sentiment = add_column(
    reviews,
    "sentiment",
    row => analyze_sentiment(row.comment)
  )
  
  show with_sentiment
}
```

### External Functions with Table Parameters

```wtlang
// Python function that processes entire tables
external function detect_anomalies(data: Table<Metric>) -> Table<Anomaly>
  from "analytics.anomaly_detection"

page AnomalyDetection {
  title "Anomaly Detection"
  
  table Metric {
    timestamp: date
    value: float
    metric_name: string
  }
  
  table Anomaly {
    timestamp: date
    metric_name: string
    severity: string
  }
  
  let metrics = load_csv("metrics.csv", Metric)
  
  // Call external function
  let anomalies = detect_anomalies(metrics)
  
  show anomalies
}
```

### Chaining with External Functions

```wtlang
external function enrich_customer_data(customers: Table<Customer>) -> Table<EnrichedCustomer>
  from "crm.enrichment"

page CustomerAnalysis {
  title "Enriched Customer Analysis"
  
  table Customer {
    id: int
    email: string
    signup_date: date
  }
  
  table EnrichedCustomer {
    id: int
    email: string
    signup_date: date
    lifetime_value: currency
    segment: string
  }
  
  let customers = load_csv("customers.csv", Customer)
  
  // Chain with external function
  let analysis = customers
    -> enrich_customer_data
    -> filter(_, row => row.lifetime_value > $1000)
    -> sort_desc(_, "lifetime_value")
  
  show analysis
}
```

---

# Advanced Topics

## Validation Rules

Define custom validation beyond simple constraints:

```wtlang
page ValidatedInput {
  title "Data Validation"
  
  // Custom validation function
  function is_valid_email(email: string) -> bool {
    // Simplified email validation
    return contains(email, "@") && contains(email, ".")
  }
  
  function is_valid_age(age: int) -> bool {
    return age >= 0 && age <= 150
  }
  
  table User {
    id: int [unique, non_null]
    email: string [non_null, validate(is_valid_email)]
    age: int [validate(is_valid_age)]
  }
  
  let users = table_from([])
  
  show_editable users
  
  button "Save" {
    // Validation happens automatically
    save_csv(users, "users.csv")
  }
}
```

## Import System

Split code across multiple files:

**src/types.wt:**
```wtlang
// Define shared types
table User {
  id: int [unique, non_null]
  name: string [non_null]
  email: string
}

table Order {
  id: int [unique, non_null]
  user_id: int [references User.id]
  amount: currency
  date: date
}
```

**src/utils.wt:**
```wtlang
import { User, Order } from "./types"

// Reusable functions
function get_user_orders(orders: Table<Order>, user_id: int) -> Table<Order> {
  return orders -> filter(_, row => row.user_id == user_id)
}

function calculate_user_total(orders: Table<Order>, user_id: int) -> currency {
  let user_orders = get_user_orders(orders, user_id)
  return sum(user_orders, "amount")
}
```

**src/main.wt:**
```wtlang
import { User, Order } from "./types"
import { get_user_orders, calculate_user_total } from "./utils"

page UserOrders {
  title "User Order History"
  
  let users = load_csv("users.csv", User)
  let orders = load_csv("orders.csv", Order)
  
  // Use imported functions
  let user_123_orders = get_user_orders(orders, 123)
  let user_123_total = calculate_user_total(orders, 123)
  
  text "Total for User 123: {user_123_total}"
  show user_123_orders
}
```

## Iteration

Loop over data when needed:

```wtlang
page IterationExample {
  title "Processing Multiple Categories"
  
  table Product {
    category: string
    name: string
    price: currency
  }
  
  let products = load_csv("products.csv", Product)
  let categories = distinct(products, "category")
  
  // Iterate over categories
  forall category in categories {
    section "Category: {category}" {
      let category_products = products 
        -> filter(_, row => row.category == category)
      
      let avg_price = average(category_products, "price")
      
      text "Average Price: {avg_price}"
      show category_products
    }
  }
}
```

---

# Complete Example

Here's a complete real-world example combining all concepts:

**src/types.wt:**
```wtlang
table Customer {
  customer_id: int [unique, non_null]
  name: string [non_null]
  email: string [non_null]
  signup_date: date
  status: string
}

table Product {
  product_id: int [unique, non_null]
  name: string [non_null]
  category: string
  price: currency [validate(x => x > 0)]
  cost: currency [validate(x => x > 0)]
}

table Order {
  order_id: int [unique, non_null]
  customer_id: int [references Customer.customer_id]
  product_id: int [references Product.product_id]
  quantity: int [validate(x => x > 0)]
  order_date: date
  status: string
}
```

**src/analytics.wt:**
```wtlang
import { Customer, Product, Order } from "./types"

function calculate_revenue(orders: Table<Order>, products: Table<Product>) -> currency {
  let enriched = join(
    orders,
    products,
    on: (o, p) => o.product_id == p.product_id
  )
  
  let with_revenue = add_column(
    enriched,
    "revenue",
    row => row.price * row.quantity
  )
  
  return sum(with_revenue, "revenue")
}

function top_customers(orders: Table<Order>, products: Table<Product>, customers: Table<Customer>, limit: int) -> Table {
  let order_products = join(orders, products, 
    on: (o, p) => o.product_id == p.product_id)
  
  let order_revenue = add_column(order_products, "revenue",
    row => row.price * row.quantity)
  
  let customer_revenue = group_by(order_revenue, "customer_id", {
    total_revenue: sum("revenue"),
    order_count: count("order_id")
  })
  
  let with_names = join(customer_revenue, customers,
    on: (rev, cust) => rev.customer_id == cust.customer_id)
  
  return with_names
    -> sort_desc(_, "total_revenue")
    -> limit(_, limit)
}
```

**src/main.wt:**
```wtlang
import { Customer, Product, Order } from "./types"
import { calculate_revenue, top_customers } from "./analytics"

external function predict_churn(customer_data: Table<Customer>) -> Table<ChurnPrediction>
  from "ml.churn"

page Dashboard {
  title "E-Commerce Dashboard"
  subtitle "Real-time business metrics"
  
  let customers = load_csv("data/customers.csv", Customer)
  let products = load_csv("data/products.csv", Product)
  let orders = load_csv("data/orders.csv", Order)
  
  // Calculate key metrics
  let total_revenue = calculate_revenue(orders, products)
  let active_customers = customers 
    -> filter(_, row => row.status == "active")
    -> count
  
  section "Key Metrics" {
    text "Total Revenue: {total_revenue}"
    text "Active Customers: {active_customers}"
  }
  
  section "Top Customers" {
    let top_10 = top_customers(orders, products, customers, 10)
    show top_10
  }
}

page OrderManagement {
  title "Order Management"
  
  let orders = load_csv("data/orders.csv", Order)
  let products = load_csv("data/products.csv", Product)
  let customers = load_csv("data/customers.csv", Customer)
  
  // Enrich orders with product and customer info
  let enriched_orders = orders
    -> join(_, products, on: (o, p) => o.product_id == p.product_id)
    -> join(_, customers, on: (o, c) => o.customer_id == c.customer_id)
    -> add_column(_, "total", row => row.price * row.quantity)
    -> sort_desc(_, "order_date")
  
  show_editable enriched_orders
  
  button "Save Changes" {
    save_csv(orders, "data/orders.csv")
  }
  
  button "Export Report" {
    export_excel(enriched_orders, "order_report.xlsx")
  }
}

page CustomerInsights {
  title "Customer Insights"
  
  let customers = load_csv("data/customers.csv", Customer)
  let orders = load_csv("data/orders.csv", Order)
  
  // ML-powered churn prediction
  let churn_predictions = predict_churn(customers)
  
  section "Churn Risk" {
    let high_risk = churn_predictions
      -> filter(_, row => row.churn_probability > 0.7)
      -> sort_desc(_, "churn_probability")
    
    text "High-risk customers: {count(high_risk)}"
    show high_risk
  }
  
  section "Customer Segmentation" {
    let order_counts = group_by(orders, "customer_id", {
      total_orders: count("order_id")
    })
    
    let segmented = add_column(order_counts, "segment", 
      row => if row.total_orders > 10 then "VIP"
             else if row.total_orders > 5 then "Regular"
             else "New")
    
    show segmented
  }
}
```

**Compile and run:**
```bash
wtc build src/main.wt --output dist/
cd dist
streamlit run Dashboard.py
```

---

## Next Steps

1. **Explore the Standard Library**: Check the [Standard Library Reference](standard-library.md) for all available functions
2. **Write Tests**: Learn about [Testing in WTLang](testing.md)
3. **Deploy Your App**: See the [Deployment Guide](deployment.md)
4. **Advanced Patterns**: Check out [Best Practices](best-practices.md)

## Getting Help

- **Documentation**: [docs.wtlang.org](https://docs.wtlang.org)
- **Examples**: [github.com/wtlang/examples](https://github.com/wtlang/examples)
- **Community**: [discord.gg/wtlang](https://discord.gg/wtlang)

Happy coding with WTLang! 🚀
