// Error system for WTLang compiler
// Each error has a unique code for easy identification and documentation

use std::fmt;

/// Error codes for WTLang compiler errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Lexical errors (E1xxx)
    E1001, // Unterminated string literal
    E1002, // Invalid number format
    E1003, // Invalid character
    E1004, // Unexpected end of file
    
    // Syntax errors (E2xxx)
    E2001, // Missing closing brace
    E2002, // Missing opening brace
    E2003, // Missing closing parenthesis
    E2004, // Missing opening parenthesis
    E2005, // Missing closing bracket
    E2006, // Missing opening bracket
    E2007, // Expected identifier
    E2008, // Expected type annotation
    E2009, // Expected expression
    E2010, // Expected statement
    E2011, // Unexpected token
    E2012, // Missing semicolon (if needed in future)
    E2013, // Missing comma in parameter list
    E2014, // Invalid function parameter
    E2015, // Missing arrow in function return type
    E2016, // Missing colon in type annotation
    
    // Semantic errors (E3xxx)
    E3001, // Undefined variable
    E3002, // Undefined function
    E3003, // Undefined table
    E3004, // Duplicate definition (variable redefinition)
    E3005, // Duplicate function definition
    E3006, // Duplicate table definition
    E3007, // Type mismatch in assignment
    E3008, // Type mismatch in function call
    E3009, // Type mismatch in return statement
    E3010, // Wrong number of arguments in function call
    E3011, // Uninitialized variable usage
    E3012, // Invalid field access (field doesn't exist)
    E3013, // Cannot access field on non-table type
    E3014, // Break outside loop
    E3015, // Continue outside loop
    E3016, // Return outside function
    E3017, // Missing return statement
    E3018, // Unreachable code after return
    
    // Table/Data errors (E4xxx)
    E4001, // Table type mismatch with CSV
    E4002, // Missing required table field
    E4003, // Extra field in table definition
    E4004, // Invalid table operation
    E4005, // Invalid filter definition
    E4006, // Filter on non-existent column
    
    // Import/External errors (E5xxx)
    E5001, // Cannot find external module
    E5002, // Invalid external function definition
    E5003, // External function not found in module
}

impl ErrorCode {
    /// Get the error code as a string (e.g., "E1001")
    pub fn code(&self) -> &'static str {
        match self {
            // Lexical errors
            ErrorCode::E1001 => "E1001",
            ErrorCode::E1002 => "E1002",
            ErrorCode::E1003 => "E1003",
            ErrorCode::E1004 => "E1004",
            
            // Syntax errors
            ErrorCode::E2001 => "E2001",
            ErrorCode::E2002 => "E2002",
            ErrorCode::E2003 => "E2003",
            ErrorCode::E2004 => "E2004",
            ErrorCode::E2005 => "E2005",
            ErrorCode::E2006 => "E2006",
            ErrorCode::E2007 => "E2007",
            ErrorCode::E2008 => "E2008",
            ErrorCode::E2009 => "E2009",
            ErrorCode::E2010 => "E2010",
            ErrorCode::E2011 => "E2011",
            ErrorCode::E2012 => "E2012",
            ErrorCode::E2013 => "E2013",
            ErrorCode::E2014 => "E2014",
            ErrorCode::E2015 => "E2015",
            ErrorCode::E2016 => "E2016",
            
            // Semantic errors
            ErrorCode::E3001 => "E3001",
            ErrorCode::E3002 => "E3002",
            ErrorCode::E3003 => "E3003",
            ErrorCode::E3004 => "E3004",
            ErrorCode::E3005 => "E3005",
            ErrorCode::E3006 => "E3006",
            ErrorCode::E3007 => "E3007",
            ErrorCode::E3008 => "E3008",
            ErrorCode::E3009 => "E3009",
            ErrorCode::E3010 => "E3010",
            ErrorCode::E3011 => "E3011",
            ErrorCode::E3012 => "E3012",
            ErrorCode::E3013 => "E3013",
            ErrorCode::E3014 => "E3014",
            ErrorCode::E3015 => "E3015",
            ErrorCode::E3016 => "E3016",
            ErrorCode::E3017 => "E3017",
            ErrorCode::E3018 => "E3018",
            
            // Table/Data errors
            ErrorCode::E4001 => "E4001",
            ErrorCode::E4002 => "E4002",
            ErrorCode::E4003 => "E4003",
            ErrorCode::E4004 => "E4004",
            ErrorCode::E4005 => "E4005",
            ErrorCode::E4006 => "E4006",
            
            // Import/External errors
            ErrorCode::E5001 => "E5001",
            ErrorCode::E5002 => "E5002",
            ErrorCode::E5003 => "E5003",
        }
    }
    
    /// Get a brief description of what this error means
    pub fn description(&self) -> &'static str {
        match self {
            // Lexical errors
            ErrorCode::E1001 => "Unterminated string literal",
            ErrorCode::E1002 => "Invalid number format",
            ErrorCode::E1003 => "Invalid character",
            ErrorCode::E1004 => "Unexpected end of file",
            
            // Syntax errors
            ErrorCode::E2001 => "Missing closing brace",
            ErrorCode::E2002 => "Missing opening brace",
            ErrorCode::E2003 => "Missing closing parenthesis",
            ErrorCode::E2004 => "Missing opening parenthesis",
            ErrorCode::E2005 => "Missing closing bracket",
            ErrorCode::E2006 => "Missing opening bracket",
            ErrorCode::E2007 => "Expected identifier",
            ErrorCode::E2008 => "Expected type annotation",
            ErrorCode::E2009 => "Expected expression",
            ErrorCode::E2010 => "Expected statement",
            ErrorCode::E2011 => "Unexpected token",
            ErrorCode::E2012 => "Missing semicolon",
            ErrorCode::E2013 => "Missing comma in parameter list",
            ErrorCode::E2014 => "Invalid function parameter",
            ErrorCode::E2015 => "Missing arrow in function return type",
            ErrorCode::E2016 => "Missing colon in type annotation",
            
            // Semantic errors
            ErrorCode::E3001 => "Undefined variable",
            ErrorCode::E3002 => "Undefined function",
            ErrorCode::E3003 => "Undefined table",
            ErrorCode::E3004 => "Variable already defined",
            ErrorCode::E3005 => "Function already defined",
            ErrorCode::E3006 => "Table already defined",
            ErrorCode::E3007 => "Type mismatch in assignment",
            ErrorCode::E3008 => "Type mismatch in function call",
            ErrorCode::E3009 => "Type mismatch in return statement",
            ErrorCode::E3010 => "Wrong number of arguments",
            ErrorCode::E3011 => "Variable used before initialization",
            ErrorCode::E3012 => "Field does not exist",
            ErrorCode::E3013 => "Cannot access field on non-table type",
            ErrorCode::E3014 => "Break statement outside loop",
            ErrorCode::E3015 => "Continue statement outside loop",
            ErrorCode::E3016 => "Return statement outside function",
            ErrorCode::E3017 => "Missing return statement",
            ErrorCode::E3018 => "Unreachable code after return",
            
            // Table/Data errors
            ErrorCode::E4001 => "Table structure mismatch with CSV",
            ErrorCode::E4002 => "Missing required table field",
            ErrorCode::E4003 => "Extra field in table definition",
            ErrorCode::E4004 => "Invalid table operation",
            ErrorCode::E4005 => "Invalid filter definition",
            ErrorCode::E4006 => "Filter on non-existent column",
            
            // Import/External errors
            ErrorCode::E5001 => "Cannot find external module",
            ErrorCode::E5002 => "Invalid external function definition",
            ErrorCode::E5003 => "External function not found in module",
        }
    }
    
    /// Get help text for fixing this error
    pub fn help(&self) -> Option<&'static str> {
        match self {
            ErrorCode::E1001 => Some("Add a closing quote (\") to terminate the string literal"),
            ErrorCode::E1002 => Some("Check the number format - use digits only, with optional decimal point"),
            ErrorCode::E2001 => Some("Add a closing brace (}) to match the opening brace"),
            ErrorCode::E2003 => Some("Add a closing parenthesis ())"),
            ErrorCode::E2005 => Some("Add a closing bracket (])"),
            ErrorCode::E2007 => Some("Provide a valid identifier (variable or function name)"),
            ErrorCode::E2016 => Some("Use colon (:) syntax for type annotations: let name: type"),
            ErrorCode::E3001 => Some("Declare the variable before using it with 'let variable_name'"),
            ErrorCode::E3004 => Some("Use a different name or remove one of the definitions"),
            ErrorCode::E3007 => Some("Ensure the value type matches the variable's declared type"),
            ErrorCode::E3011 => Some("Initialize the variable before using it, or use conditional initialization"),
            _ => None,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Location information for an error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Location { line, column, file: None }
    }
    
    pub fn with_file(line: usize, column: usize, file: String) -> Self {
        Location { line, column, file: Some(file) }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref file) = self.file {
            write!(f, "{}:{}:{}", file, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

/// Severity level of a diagnostic message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

/// A diagnostic message (error, warning, info, or hint)
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: ErrorCode,
    pub message: String,
    pub location: Location,
    pub context: Option<String>,  // Source code snippet showing the error
}

impl Diagnostic {
    pub fn new(severity: Severity, code: ErrorCode, message: String, location: Location) -> Self {
        Diagnostic {
            severity,
            code,
            message,
            location,
            context: None,
        }
    }
    
    pub fn error(code: ErrorCode, message: String, location: Location) -> Self {
        Diagnostic::new(Severity::Error, code, message, location)
    }
    
    pub fn warning(code: ErrorCode, message: String, location: Location) -> Self {
        Diagnostic::new(Severity::Warning, code, message, location)
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Format the diagnostic for display
    pub fn format(&self) -> String {
        let mut output = String::new();
        
        // Main error line: severity[code]: message
        output.push_str(&format!(
            "{}[{}]: {}\n",
            self.severity,
            self.code,
            self.message
        ));
        
        // Location
        output.push_str(&format!("  --> {}\n", self.location));
        
        // Context (source snippet)
        if let Some(ref context) = self.context {
            output.push_str("   |\n");
            output.push_str(&format!(" {} | {}\n", self.location.line, context));
            output.push_str("   |\n");
        }
        
        // Help text
        if let Some(help) = self.code.help() {
            output.push_str(&format!("  = help: {}\n", help));
        }
        
        output
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Collection of diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        DiagnosticBag {
            diagnostics: Vec::new(),
        }
    }
    
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    pub fn add_error(&mut self, code: ErrorCode, message: String, location: Location) {
        self.add(Diagnostic::error(code, message, location));
    }
    
    pub fn add_warning(&mut self, code: ErrorCode, message: String, location: Location) {
        self.add(Diagnostic::warning(code, message, location));
    }
    
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
    
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).count()
    }
    
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count()
    }
    
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
    
    /// Format all diagnostics for display
    pub fn format_all(&self) -> String {
        let mut output = String::new();
        
        for diagnostic in &self.diagnostics {
            output.push_str(&diagnostic.format());
            output.push('\n');
        }
        
        // Summary
        let errors = self.error_count();
        let warnings = self.warning_count();
        
        if errors > 0 || warnings > 0 {
            output.push_str(&format!(
                "Found {} error(s) and {} warning(s)\n",
                errors, warnings
            ));
        }
        
        output
    }
}

impl fmt::Display for DiagnosticBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_all())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::E1001.code(), "E1001");
        assert_eq!(ErrorCode::E3001.code(), "E3001");
    }

    #[test]
    fn test_error_code_description() {
        assert_eq!(ErrorCode::E1001.description(), "Unterminated string literal");
        assert_eq!(ErrorCode::E3001.description(), "Undefined variable");
    }

    #[test]
    fn test_location_display() {
        let loc = Location::new(10, 5);
        assert_eq!(loc.to_string(), "10:5");
        
        let loc_with_file = Location::with_file(10, 5, "test.wt".to_string());
        assert_eq!(loc_with_file.to_string(), "test.wt:10:5");
    }

    #[test]
    fn test_diagnostic_creation() {
        let loc = Location::new(5, 10);
        let diag = Diagnostic::error(
            ErrorCode::E3001,
            "Variable 'x' is not defined".to_string(),
            loc.clone()
        );
        
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, ErrorCode::E3001);
        assert_eq!(diag.location, loc);
    }

    #[test]
    fn test_diagnostic_bag() {
        let mut bag = DiagnosticBag::new();
        
        bag.add_error(ErrorCode::E3001, "Undefined variable".to_string(), Location::new(1, 1));
        bag.add_warning(ErrorCode::E3004, "Variable shadowing".to_string(), Location::new(2, 1));
        
        assert_eq!(bag.error_count(), 1);
        assert_eq!(bag.warning_count(), 1);
        assert!(bag.has_errors());
    }
}
