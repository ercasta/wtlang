// Code generator for WTLang -> Python/Streamlit
use wtlang_core::ast::*;
use std::collections::HashMap;

pub struct CodeGenerator {
    indent_level: usize,
    table_defs: HashMap<String, TableDef>,
    external_functions: HashMap<String, ExternalFunction>,
    key_counter: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            indent_level: 0,
            table_defs: HashMap::new(),
            external_functions: HashMap::new(),
            key_counter: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<HashMap<String, String>, String> {
        let mut output_files = HashMap::new();
        
        // First pass: collect table definitions and external functions
        for item in &program.items {
            match item {
                ProgramItem::TableDef(table_def) => {
                    self.table_defs.insert(table_def.name.clone(), table_def.clone());
                }
                ProgramItem::ExternalFunction(ext_fn) => {
                    self.external_functions.insert(ext_fn.name.clone(), ext_fn.clone());
                }
                _ => {}
            }
        }
        
        // Second pass: generate pages
        for item in &program.items {
            if let ProgramItem::Page(page) = item {
                let code = self.generate_page(page)?;
                output_files.insert(format!("{}.py", page.name), code);
            }
        }
        
        Ok(output_files)
    }

    fn generate_page(&mut self, page: &Page) -> Result<String, String> {
        let mut code = String::new();
        
        // Standard imports
        code.push_str("import streamlit as st\n");
        code.push_str("import pandas as pd\n");
        code.push_str("from datetime import datetime\n");
        
        // External function imports
        // Group by module to generate clean imports
        let mut modules: HashMap<String, Vec<String>> = HashMap::new();
        for (func_name, ext_fn) in &self.external_functions {
            modules.entry(ext_fn.module.clone())
                .or_insert_with(Vec::new)
                .push(func_name.clone());
        }
        
        // Generate import statements
        for (module, functions) in modules {
            if functions.len() == 1 {
                code.push_str(&format!("from {} import {}\n", module, functions[0]));
            } else {
                code.push_str(&format!("from {} import {}\n", module, functions.join(", ")));
            }
        }
        
        code.push_str("\n");
        
        // Helper function for filtered show/show_editable
        code.push_str("def _show_filtered(df, filters, editable=False, key_prefix=''):\n");
        code.push_str("    \"\"\"Show dataframe with optional filters\"\"\"\n");
        code.push_str("    # Create filter widgets (3 per row)\n");
        code.push_str("    filter_values = []\n");
        code.push_str("    num_filters = len(filters)\n");
        code.push_str("    for i in range(0, num_filters, 3):\n");
        code.push_str("        cols = st.columns(min(3, num_filters - i))\n");
        code.push_str("        for j, (col_name, mode) in enumerate(filters[i:i+3]):\n");
        code.push_str("            if mode == 'single':\n");
        code.push_str("                val = cols[j].selectbox(col_name, ['All'] + sorted(df[col_name].unique().astype(str).tolist()), key=f'{key_prefix}_f_{i+j}')\n");
        code.push_str("                filter_values.append((col_name, mode, val))\n");
        code.push_str("            else:  # multi\n");
        code.push_str("                val = cols[j].multiselect(col_name, sorted(df[col_name].unique().astype(str).tolist()), key=f'{key_prefix}_f_{i+j}')\n");
        code.push_str("                filter_values.append((col_name, mode, val))\n");
        code.push_str("    \n");
        code.push_str("    # Apply filters and track filtered/non-filtered rows\n");
        code.push_str("    mask = pd.Series([True] * len(df), index=df.index)\n");
        code.push_str("    for col_name, mode, val in filter_values:\n");
        code.push_str("        if mode == 'single' and val != 'All':\n");
        code.push_str("            mask = mask & (df[col_name].astype(str) == val)\n");
        code.push_str("        elif mode == 'multi' and val:\n");
        code.push_str("            mask = mask & df[col_name].astype(str).isin(val)\n");
        code.push_str("    \n");
        code.push_str("    filtered = df[mask]\n");
        code.push_str("    non_filtered = df[~mask]\n");
        code.push_str("    \n");
        code.push_str("    # Display\n");
        code.push_str("    if editable:\n");
        code.push_str("        edited = st.data_editor(filtered, key=f'{key_prefix}_editor', use_container_width=True)\n");
        code.push_str("        # Merge edited filtered rows with non-filtered rows\n");
        code.push_str("        return pd.concat([edited, non_filtered], ignore_index=True)\n");
        code.push_str("    else:\n");
        code.push_str("        st.dataframe(filtered)\n");
        code.push_str("        return None\n");
        code.push_str("\n");
        
        // Page configuration
        code.push_str(&format!("# Page: {}\n", page.name));
        code.push_str("\n");
        
        // Generate statements
        for stmt in &page.statements {
            code.push_str(&self.generate_statement(stmt)?);
        }
        
        Ok(code)
    }

    fn generate_statement(&mut self, stmt: &Statement) -> Result<String, String> {
        let indent = self.get_indent();
        
        match stmt {
            Statement::Title(text) => {
                Ok(format!("{}st.title(\"{}\")\n", indent, self.escape_string(text)))
            },
            Statement::Subtitle(text) => {
                Ok(format!("{}st.subheader(\"{}\")\n", indent, self.escape_string(text)))
            },
            Statement::Text(text) => {
                // Handle string interpolation
                let formatted = self.format_string_interpolation(text);
                Ok(format!("{}st.write({})\n", indent, formatted))
            },
            Statement::Button { label, body } => {
                let mut code = format!("{}if st.button(\"{}\"):\n", indent, self.escape_string(label));
                self.indent_level += 1;
                for s in body {
                    code.push_str(&self.generate_statement(s)?);
                }
                self.indent_level -= 1;
                Ok(code)
            },
            Statement::Section { title, body } => {
                let mut code = format!("{}with st.container():\n", indent);
                self.indent_level += 1;
                code.push_str(&format!("{}st.markdown(\"### {}\")\n", self.get_indent(), self.escape_string(title)));
                for s in body {
                    code.push_str(&self.generate_statement(s)?);
                }
                self.indent_level -= 1;
                Ok(code)
            },
            Statement::Let { name, value } => {
                let value_code = self.generate_expr(value)?;
                Ok(format!("{}{} = {}\n", indent, name, value_code))
            },
            Statement::If { condition, then_branch, else_branch } => {
                let cond_code = self.generate_expr(condition)?;
                let mut code = format!("{}if {}:\n", indent, cond_code);
                self.indent_level += 1;
                for s in then_branch {
                    code.push_str(&self.generate_statement(s)?);
                }
                self.indent_level -= 1;
                
                if let Some(else_stmts) = else_branch {
                    code.push_str(&format!("{}else:\n", indent));
                    self.indent_level += 1;
                    for s in else_stmts {
                        code.push_str(&self.generate_statement(s)?);
                    }
                    self.indent_level -= 1;
                }
                Ok(code)
            },
            Statement::Forall { var, iterable, body } => {
                let iter_code = self.generate_expr(iterable)?;
                let mut code = format!("{}for {} in {}:\n", indent, var, iter_code);
                self.indent_level += 1;
                for s in body {
                    code.push_str(&self.generate_statement(s)?);
                }
                self.indent_level -= 1;
                Ok(code)
            },
            Statement::FunctionCall(call) => {
                let call_code = self.generate_function_call(call)?;
                Ok(format!("{}{}\n", indent, call_code))
            },
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntLiteral(n) => Ok(n.to_string()),
            Expr::FloatLiteral(f) => Ok(f.to_string()),
            Expr::StringLiteral(s) => Ok(format!("\"{}\"", self.escape_string(s))),
            Expr::BoolLiteral(b) => Ok(if *b { "True" } else { "False" }.to_string()),
            Expr::Identifier(name) => Ok(name.clone()),
            Expr::FunctionCall(call) => self.generate_function_call(call),
            Expr::BinaryOp { op, left, right } => {
                let left_code = self.generate_expr(left)?;
                let right_code = self.generate_expr(right)?;
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::Multiply => "*",
                    BinaryOp::Divide => "/",
                    BinaryOp::Modulo => "%",
                    BinaryOp::Equal => "==",
                    BinaryOp::NotEqual => "!=",
                    BinaryOp::LessThan => "<",
                    BinaryOp::LessThanEqual => "<=",
                    BinaryOp::GreaterThan => ">",
                    BinaryOp::GreaterThanEqual => ">=",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                };
                Ok(format!("({} {} {})", left_code, op_str, right_code))
            },
            Expr::UnaryOp { op, operand } => {
                let operand_code = self.generate_expr(operand)?;
                let op_str = match op {
                    UnaryOp::Not => "not",
                    UnaryOp::Negate => "-",
                };
                Ok(format!("({} {})", op_str, operand_code))
            },
            Expr::FieldAccess { object, field } => {
                let obj_code = self.generate_expr(object)?;
                Ok(format!("{}[{:?}]", obj_code, field))
            },
            Expr::Index { object, index } => {
                let obj_code = self.generate_expr(object)?;
                let idx_code = self.generate_expr(index)?;
                Ok(format!("{}.iloc[{}]", obj_code, idx_code))
            },
            Expr::Chain { left, right } => {
                // Function chaining: left -> right
                // Right should be a function call with _ as first argument
                let left_code = self.generate_expr(left)?;
                
                // Replace _ in right with left_code
                match right.as_ref() {
                    Expr::FunctionCall(call) => {
                        // Create a new function call with _ replaced by the left side
                        let mut new_call = call.clone();
                        for arg in &mut new_call.args {
                            if matches!(arg, Expr::Identifier(name) if name == "_") {
                                *arg = Expr::Identifier(left_code.clone());
                            }
                        }
                        // Use generate_function_call to handle special functions properly
                        self.generate_function_call(&new_call)
                    },
                    _ => Err("Chain right side must be a function call".to_string()),
                }
            },
            Expr::FilterLiteral(_) => {
                // Filter literals are only used as part of filter arrays, not standalone
                Err("Filter literals can only be used within show/show_editable filter arrays".to_string())
            },
            Expr::ArrayLiteral(_) => {
                // Array literals for filters are handled specially in show/show_editable
                Err("Array literals must be handled in context (e.g., for filters)".to_string())
            },
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }

    fn generate_function_call(&mut self, call: &FunctionCall) -> Result<String, String> {
        // Map WTLang functions to pandas/Python equivalents
        let func_name = match call.name.as_str() {
            "load_csv" => {
                if call.args.len() < 1 {
                    return Err("load_csv requires at least 1 argument (file path)".to_string());
                }
                let file_arg = self.generate_expr(&call.args[0])?;
                
                // Check if a table type was specified as second argument
                if call.args.len() >= 2 {
                    if let Expr::Identifier(table_name) = &call.args[1] {
                        if let Some(table_def) = self.table_defs.get(table_name) {
                            // Generate code with validation
                            let field_names: Vec<String> = table_def.fields.iter()
                                .map(|f| format!("\"{}\"", f.name))
                                .collect();
                            let expected_cols = format!("[{}]", field_names.join(", "));
                            
                            return Ok(format!(
                                "(_df := pd.read_csv({}), \
                                st.error(f'Invalid CSV: expected columns {}, got {{list(_df.columns)}}') \
                                if not set({}).issubset(set(_df.columns)) else None, \
                                _df)[2]",
                                file_arg, expected_cols, expected_cols
                            ));
                        }
                    }
                }
                
                // No table type specified, just load the CSV
                return Ok(format!("pd.read_csv({})", file_arg));
            },
            "show" => {
                if call.args.len() < 1 {
                    return Err("show requires at least 1 argument (table to display)".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                
                // If filters are provided as second argument (array of filters)
                if call.args.len() > 1 {
                    return self.generate_show_with_filters(&df_arg, &call.args[1], false);
                }
                
                return Ok(format!("st.dataframe({})", df_arg));
            },
            "show_editable" => {
                if call.args.len() < 1 {
                    return Err("show_editable requires at least 1 argument (table to edit)".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                
                // If filters are provided as second argument (array of filters)
                if call.args.len() > 1 {
                    return self.generate_show_with_filters(&df_arg, &call.args[1], true);
                }
                
                // show_editable returns the edited dataframe (tables are immutable)
                return Ok(format!("st.data_editor({}, key=\"editor_{}\", use_container_width=True)", 
                    df_arg, 
                    self.get_unique_key()));
            },
            "save_csv" => {
                if call.args.len() < 2 {
                    return Err("save_csv requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                let file_arg = self.generate_expr(&call.args[1])?;
                return Ok(format!("{}.to_csv({}, index=False)", df_arg, file_arg));
            },
            "export_excel" => {
                if call.args.len() < 2 {
                    return Err("export_excel requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                let file_arg = self.generate_expr(&call.args[1])?;
                return Ok(format!("{}.to_excel({}, index=False)", df_arg, file_arg));
            },
            "where" => {
                if call.args.len() < 2 {
                    return Err("where requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                // Second argument should be a lambda, for now just pass through
                return Ok(format!("{}.query({})", df_arg, self.generate_expr(&call.args[1])?));
            },
            "sort" => {
                if call.args.len() < 2 {
                    return Err("sort requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                let col_arg = self.generate_expr(&call.args[1])?;
                return Ok(format!("{}.sort_values({})", df_arg, col_arg));
            },
            "sort_desc" => {
                if call.args.len() < 2 {
                    return Err("sort_desc requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                let col_arg = self.generate_expr(&call.args[1])?;
                return Ok(format!("{}.sort_values({}, ascending=False)", df_arg, col_arg));
            },
            "sum" => {
                if call.args.len() < 2 {
                    return Err("sum requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                let col_arg = self.generate_expr(&call.args[1])?;
                return Ok(format!("{}[{}].sum()", df_arg, col_arg));
            },
            "count" => {
                if call.args.len() < 1 {
                    return Err("count requires 1 argument".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                return Ok(format!("len({})", df_arg));
            },
            "average" | "mean" => {
                if call.args.len() < 2 {
                    return Err("average requires 2 arguments".to_string());
                }
                let df_arg = self.generate_expr(&call.args[0])?;
                let col_arg = self.generate_expr(&call.args[1])?;
                return Ok(format!("{}[{}].mean()", df_arg, col_arg));
            },
            _ => call.name.as_str(),
        };
        
        let args: Result<Vec<String>, String> = call.args.iter()
            .map(|arg| self.generate_expr(arg))
            .collect();
        
        Ok(format!("{}({})", func_name, args?.join(", ")))
    }

    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('"', "\\\"")
         .replace('\n', "\\n")
         .replace('\t', "\\t")
    }

    fn format_string_interpolation(&self, text: &str) -> String {
        // Simple f-string conversion for {var} syntax
        if text.contains('{') {
            format!("f\"{}\"", text.replace('"', "\\\""))
        } else {
            format!("\"{}\"", self.escape_string(text))
        }
    }

    fn get_indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn get_unique_key(&mut self) -> usize {
        let key = self.key_counter;
        self.key_counter += 1;
        key
    }

    fn generate_show_with_filters(&mut self, df_expr: &str, filters_expr: &Expr, is_editable: bool) -> Result<String, String> {
        // Parse the filters array
        let filters = match filters_expr {
            Expr::ArrayLiteral(filter_exprs) => {
                let mut filters = Vec::new();
                for filter_expr in filter_exprs {
                    match filter_expr {
                        Expr::FilterLiteral(filter_def) => {
                            filters.push(filter_def.clone());
                        }
                        _ => return Err("Filter arrays must contain only filter literals".to_string()),
                    }
                }
                filters
            }
            _ => return Err("Second argument to show/show_editable must be an array of filters".to_string()),
        };

        if filters.is_empty() {
            // No filters, just show the dataframe
            if is_editable {
                return Ok(format!("st.data_editor({}, key=\"editor_{}\", use_container_width=True)", 
                    df_expr, self.get_unique_key()));
            } else {
                return Ok(format!("st.dataframe({})", df_expr));
            }
        }

        let key = self.get_unique_key();
        
        // Build filter list as Python code
        let filter_list: Vec<String> = filters.iter().map(|f| {
            let mode = match f.mode {
                FilterMode::Single => "single",
                FilterMode::Multi => "multi",
            };
            format!("('{}', '{}')", f.column, mode)
        }).collect();
        
        // Call the helper function
        Ok(format!(
            "_show_filtered({}, [{}], editable={}, key_prefix='f_{}')",
            df_expr,
            filter_list.join(", "),
            if is_editable { "True" } else { "False" },
            key
        ))
    }
}
