// Code generator for WTLang -> Python/Streamlit
use wtlang_core::ast::*;
use std::collections::HashMap;

pub struct CodeGenerator {
    indent_level: usize,
    table_defs: HashMap<String, TableDef>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            indent_level: 0,
            table_defs: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<HashMap<String, String>, String> {
        let mut output_files = HashMap::new();
        
        // First pass: collect table definitions
        for item in &program.items {
            if let ProgramItem::TableDef(table_def) = item {
                self.table_defs.insert(table_def.name.clone(), table_def.clone());
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
        
        // Imports
        code.push_str("import streamlit as st\n");
        code.push_str("import pandas as pd\n");
        code.push_str("from datetime import datetime\n");
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
            Statement::Show(expr) => {
                let expr_code = self.generate_expr(expr)?;
                Ok(format!("{}st.dataframe({})\n", indent, expr_code))
            },
            Statement::ShowEditable(expr) => {
                let expr_code = self.generate_expr(expr)?;
                Ok(format!("{}st.data_editor({})\n", indent, expr_code))
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
                        let mut args = Vec::new();
                        for arg in &call.args {
                            if matches!(arg, Expr::Identifier(name) if name == "_") {
                                args.push(left_code.clone());
                            } else {
                                args.push(self.generate_expr(arg)?);
                            }
                        }
                        Ok(format!("{}({})", call.name, args.join(", ")))
                    },
                    _ => Err("Chain right side must be a function call".to_string()),
                }
            },
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }

    fn generate_function_call(&mut self, call: &FunctionCall) -> Result<String, String> {
        // Map WTLang functions to pandas/Python equivalents
        let func_name = match call.name.as_str() {
            "load_csv" => {
                if call.args.len() < 1 {
                    return Err("load_csv requires at least 1 argument".to_string());
                }
                let file_arg = self.generate_expr(&call.args[0])?;
                return Ok(format!("pd.read_csv({})", file_arg));
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
            "filter" => {
                if call.args.len() < 2 {
                    return Err("filter requires 2 arguments".to_string());
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
}
