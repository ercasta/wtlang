# WTLang Examples

This directory contains example WTLang programs demonstrating various language features.

## Examples

### 01_hello.wt
**Basic Hello World**
- Simple page with title and subtitle
- Text display

### 02_tables.wt
**Table Definitions and Display**
- Defining table structures with fields and constraints
- Loading data from CSV files
- Displaying tables
- Export functionality with buttons

### 03_chaining.wt
**Data Manipulation with Function Chaining**
- Using the `->` operator for function chaining
- Filtering and sorting data
- Aggregation functions (count, sum, average)
- Multiple sections within a page

### 04_multi_page.wt
**Multiple Pages and Conditional Logic**
- Creating multiple pages in one application
- Conditional statements with `if`
- Editable tables with `show_editable`
- Button actions for saving and exporting

## Compiling Examples

To compile any example:

```bash
# From the wtlang root directory
cargo build --release

# Compile an example
./target/release/wtc build examples/01_hello.wt --output output/01_hello
./target/release/wtc build examples/02_tables.wt --output output/02_tables
./target/release/wtc build examples/03_chaining.wt --output output/03_chaining
./target/release/wtc build examples/04_multi_page.wt --output output/04_multi_page
```

## Running the Generated Applications

After compilation, navigate to the output directory and run Streamlit:

```bash
cd output/01_hello
pip install -r requirements.txt
streamlit run Home.py
```

For multi-page applications (like 04_multi_page), Streamlit will automatically detect multiple pages and create a navigation sidebar.

## Test Data

Sample CSV files are provided in the `data/` directory:
- `users.csv` - Sample user data
- `products.csv` - Sample product data
- `orders.csv` - Sample order data

Copy these to your output directory when testing examples that load data.

## Quick Test Script

```bash
# Build and test the hello world example
cargo build --release
./target/release/wtc build examples/01_hello.wt --output output/test
cd output/test
python -m streamlit run Home.py
```
