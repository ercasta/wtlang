# Code Generation Refactoring Design

## Executive Summary

This document analyzes the current code generation architecture in WTLang compiler and proposes refactoring strategies to:
1. Support multiple backend targets (beyond Streamlit)
2. Enable template-based code generation for easier customization
3. Separate code generation logic from compilation pipeline
4. Allow manual editing of code generation templates

**Current State**: Monolithic code generator hardcoded for Python/Streamlit target  
**Proposed State**: Modular, template-based multi-backend architecture

---

## 1. Current Architecture Analysis

### 1.1 Current Implementation

The code generator (`crates/wtlang-compiler/src/codegen.rs`) is a single struct that:
- Directly generates Python/Streamlit code as strings
- Hardcodes all target-specific patterns (imports, function mappings, widget calls)
- Mixes code generation logic with domain logic (e.g., filter handling)
- Contains 500+ lines of string concatenation and formatting

**Example of current approach:**
```rust
fn generate_statement(&mut self, stmt: &Statement) -> Result<String, String> {
    match stmt {
        Statement::Title(text) => {
            Ok(format!("{}st.title(\"{}\")\n", indent, self.escape_string(text)))
        },
        Statement::Button { label, body } => {
            let mut code = format!("{}if st.button(\"{}\"):\n", indent, self.escape_string(label));
            // ... more hardcoded Streamlit patterns
        }
    }
}
```

### 1.2 Current Strengths

1. **Simple & Direct**: Easy to understand, no abstraction overhead
2. **Fast Development**: Quick to add new features for single target
3. **Type Safety**: Rust ensures correctness at compile time
4. **Performance**: No runtime overhead, direct string generation

### 1.3 Current Limitations

1. **Single Target**: Only generates Python/Streamlit code
2. **Non-Extensible**: Adding new backend requires rewriting entire generator
3. **Non-Customizable**: Users cannot modify templates without recompiling
4. **Maintenance Burden**: Code generation patterns scattered throughout codebase
5. **Testing Difficulty**: Hard to test template variations
6. **Code Duplication**: Similar patterns repeated (e.g., indentation, escaping)

---

## 2. Architecture Design Alternatives

### Architecture A: Template-Based Generation with Embedded Templates

**Description**: Extract code patterns into template strings embedded in Rust code, use a template engine like Tera or Handlebars.

**Structure**:
```
crates/wtlang-compiler/src/
  codegen/
    mod.rs              # Main code generator interface
    context.rs          # Template context builders
    backends/
      mod.rs
      streamlit.rs      # Streamlit-specific templates & logic
      react.rs          # Future: React backend
      jupyter.rs        # Future: Jupyter backend
  templates/           # Embedded in binary via include_str!
    streamlit/
      page.py.tera
      imports.py.tera
      statement.py.tera
      show_filtered.py.tera
```

**Template Example** (`templates/streamlit/page.py.tera`):
```jinja2
import streamlit as st
import pandas as pd
from datetime import datetime

{% for module, functions in external_imports %}
from {{ module }} import {{ functions | join(", ") }}
{% endfor %}

{{ helper_functions }}

# Page: {{ page_name }}

{% for statement in statements %}
{{ statement }}
{% endfor %}
```

**Rust Code**:
```rust
use tera::{Tera, Context};

pub struct StreamlitBackend {
    templates: Tera,
}

impl StreamlitBackend {
    pub fn new() -> Self {
        let mut tera = Tera::default();
        tera.add_raw_template("page.py", include_str!("../../templates/streamlit/page.py.tera")).unwrap();
        // ... more templates
        StreamlitBackend { templates: tera }
    }
}

impl Backend for StreamlitBackend {
    fn generate_page(&self, page: &Page) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("page_name", &page.name);
        context.insert("statements", &self.prepare_statements(&page.statements)?);
        self.templates.render("page.py", &context)
            .map_err(|e| format!("Template error: {}", e))
    }
}
```

**Pros**:
- ✅ Templates are more readable than string concatenation
- ✅ Templates embedded in binary (no external files needed)
- ✅ Template engine handles escaping, whitespace, loops
- ✅ Easy to add new backends (just new template sets)
- ✅ Can version templates with compiler

**Cons**:
- ❌ Templates still not user-editable without recompiling
- ❌ Template syntax adds learning curve
- ❌ Debugging template errors can be harder
- ❌ Performance overhead (template parsing/rendering)
- ❌ Adds dependency on template engine

**Impact Assessment**:
- Development Time: **Medium** (2-3 weeks to refactor)
- Performance Impact: **Low** (negligible for typical projects)
- User Benefit: **Medium** (easier to read/modify templates in source)
- Extensibility: **High** (easy to add backends)

---

### Architecture B: External Template Files with Runtime Loading

**Description**: Store templates as external files, loaded at runtime from configurable directory.

**Structure**:
```
wtlang/
  templates/
    streamlit/
      page.py.hbs
      statement/
        title.py.hbs
        button.py.hbs
        section.py.hbs
      expression/
        binary_op.py.hbs
        function_call.py.hbs
      helpers/
        imports.py.hbs
        show_filtered.py.hbs
    react/              # Future backend
      page.tsx.hbs
    jupyter/            # Future backend
      notebook.ipynb.hbs
```

**Configuration** (`wtlang.toml`):
```toml
[codegen]
backend = "streamlit"
template_dir = "templates/streamlit"
custom_template_dir = "~/.wtlang/custom_templates"  # Optional user overrides
```

**Rust Code**:
```rust
pub struct CodeGenConfig {
    backend: String,
    template_dir: PathBuf,
    custom_template_dir: Option<PathBuf>,
}

pub struct TemplateBackend {
    config: CodeGenConfig,
    templates: Handlebars<'static>,
}

impl TemplateBackend {
    pub fn new(config: CodeGenConfig) -> Result<Self, String> {
        let mut hbs = Handlebars::new();
        
        // Load templates from directory
        let template_path = config.template_dir.join("**/*.hbs");
        hbs.register_templates_directory(&template_path, ".hbs")
            .map_err(|e| format!("Failed to load templates: {}", e))?;
        
        // Load user custom templates (overrides)
        if let Some(ref custom_dir) = config.custom_template_dir {
            if custom_dir.exists() {
                hbs.register_templates_directory(&custom_dir.join("**/*.hbs"), ".hbs")?;
            }
        }
        
        Ok(TemplateBackend { config, templates: hbs })
    }
    
    pub fn generate_statement(&self, stmt: &Statement) -> Result<String, String> {
        let template_name = format!("statement/{}", stmt.template_name());
        let context = stmt.to_context();
        self.templates.render(&template_name, &context)
            .map_err(|e| format!("Template error: {}", e))
    }
}
```

**Template Example** (`templates/streamlit/statement/title.py.hbs`):
```handlebars
{{indent}}st.title("{{escape text}}")
```

**Pros**:
- ✅ **User-editable templates**: Users can customize code generation
- ✅ Template changes don't require recompilation
- ✅ Version control friendly (templates are plain text files)
- ✅ Easy to share/distribute custom template sets
- ✅ Hot-reload during development
- ✅ Multiple backends just need different directories

**Cons**:
- ❌ Distribution complexity (need to ship templates with compiler)
- ❌ Template location/loading issues
- ❌ Breaking changes in templates could break compiler
- ❌ Users might edit templates incorrectly
- ❌ Security concerns (template injection)

**Impact Assessment**:
- Development Time: **Medium-High** (3-4 weeks)
- Performance Impact: **Low-Medium** (template loading overhead)
- User Benefit: **High** (full customization capability)
- Extensibility: **Very High** (trivial to add backends)

---

### Architecture C: Intermediate Representation (IR) Based

**Description**: Generate platform-agnostic IR, then have backend-specific IR→code translators.

**Structure**:
```
AST (wtlang-core)
  ↓
Semantic Analysis
  ↓
IR (Intermediate Representation)
  ↓
Backend-specific Code Generator
  ↓
Target Code (Python/TypeScript/etc.)
```

**IR Example**:
```rust
pub enum IRNode {
    Module {
        name: String,
        imports: Vec<Import>,
        statements: Vec<IRStatement>,
    },
    ShowTable {
        table_expr: Box<IRExpr>,
        filters: Vec<Filter>,
        editable: bool,
        key: String,
    },
    ConditionalBlock {
        condition: Box<IRExpr>,
        then_branch: Vec<IRStatement>,
        else_branch: Option<Vec<IRStatement>>,
    },
    // ... platform-agnostic operations
}

pub trait Backend {
    fn generate_module(&self, node: &IRNode) -> String;
    fn generate_show_table(&self, node: &IRNode) -> String;
    // ...
}
```

**Streamlit Backend**:
```rust
impl Backend for StreamlitBackend {
    fn generate_show_table(&self, node: &IRNode) -> String {
        if let IRNode::ShowTable { table_expr, filters, editable, key } = node {
            if filters.is_empty() {
                if *editable {
                    format!("st.data_editor({}, key=\"{}\")", 
                        self.generate_expr(table_expr), key)
                } else {
                    format!("st.dataframe({})", self.generate_expr(table_expr))
                }
            } else {
                // ... filter generation
            }
        }
    }
}
```

**React Backend** (future):
```rust
impl Backend for ReactBackend {
    fn generate_show_table(&self, node: &IRNode) -> String {
        if let IRNode::ShowTable { table_expr, filters, editable, key } = node {
            format!(
                "<DataTable data={{{}}} filters={{{}}} editable={{{}}} />",
                self.generate_expr(table_expr),
                self.generate_filters(filters),
                editable
            )
        }
    }
}
```

**Pros**:
- ✅ Clean separation of concerns
- ✅ Backend-agnostic optimizations possible
- ✅ Easy to add new backends
- ✅ IR can be inspected/optimized
- ✅ Multiple output formats from same IR
- ✅ Testable IR layer

**Cons**:
- ❌ **Significant development effort** (4-6 weeks)
- ❌ IR design complexity
- ❌ Two-stage compilation (AST→IR→Code)
- ❌ Performance overhead
- ❌ Over-engineering for current needs
- ❌ Doesn't address template customization

**Impact Assessment**:
- Development Time: **High** (1-2 months)
- Performance Impact: **Medium** (additional IR pass)
- User Benefit: **Low** (not directly user-facing)
- Extensibility: **Very High** (best for multiple backends)

---

### Architecture D: Code Generator Registry with Plugins

**Description**: Plugin-based system where backends are registered dynamically.

**Structure**:
```rust
pub trait CodeGenBackend: Send + Sync {
    fn name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn generate_program(&self, program: &Program) -> Result<HashMap<String, String>, String>;
}

pub struct CodeGenRegistry {
    backends: HashMap<String, Box<dyn CodeGenBackend>>,
}

impl CodeGenRegistry {
    pub fn new() -> Self {
        let mut registry = CodeGenRegistry { backends: HashMap::new() };
        
        // Register built-in backends
        registry.register(Box::new(StreamlitBackend::new()));
        
        // Future: Load plugin backends from shared libraries
        // registry.load_plugins_from("~/.wtlang/plugins");
        
        registry
    }
    
    pub fn register(&mut self, backend: Box<dyn CodeGenBackend>) {
        self.backends.insert(backend.name().to_string(), backend);
    }
    
    pub fn generate(&self, backend_name: &str, program: &Program) 
        -> Result<HashMap<String, String>, String> {
        self.backends.get(backend_name)
            .ok_or_else(|| format!("Backend '{}' not found", backend_name))?
            .generate_program(program)
    }
}
```

**Pros**:
- ✅ Extensible via plugins
- ✅ Third-party backends possible
- ✅ Backend selection at runtime
- ✅ Clean interface/abstraction

**Cons**:
- ❌ Complex plugin infrastructure
- ❌ ABI compatibility challenges (Rust plugins are hard)
- ❌ Security concerns with dynamic loading
- ❌ Still doesn't solve template customization
- ❌ Overkill for current requirements

**Impact Assessment**:
- Development Time: **Very High** (2+ months)
- Performance Impact: **Low**
- User Benefit: **Low-Medium** (future-facing)
- Extensibility: **Extreme** (but complex)

---

## 3. Hybrid Recommendation: Template-Based with Configurable Backends

### 3.1 Recommended Architecture

**Combine the best of Architecture A and B**: 
- Templates stored externally (like B) 
- But with embedded fallbacks (like A)
- Multiple backends supported through template directories
- Simple trait-based backend system (like D, but simpler)

**Structure**:
```
wtlang/
  crates/
    wtlang-compiler/
      src/
        codegen/
          mod.rs                    # Main interface
          backend.rs                # Backend trait
          context.rs                # Template context builders
          template_loader.rs        # Template loading logic
          backends/
            mod.rs
            streamlit/
              mod.rs                # StreamlitBackend implementation
              helpers.rs            # Streamlit-specific helpers
              templates.rs          # Embedded template strings
  templates/                        # External templates (override built-in)
    streamlit/
      page.py.hbs
      statement/
        title.py.hbs
        subtitle.py.hbs
        text.py.hbs
        button.py.hbs
        section.py.hbs
        let.py.hbs
        assign.py.hbs
        if.py.hbs
        forall.py.hbs
      expression/
        literal.py.hbs
        identifier.py.hbs
        binary_op.py.hbs
        function_call.py.hbs
        field_access.py.hbs
        chain.py.hbs
      builtin/
        load_csv.py.hbs
        show.py.hbs
        show_editable.py.hbs
        save_csv.py.hbs
        where.py.hbs
        sort.py.hbs
      helpers/
        imports.py.hbs
        show_filtered_helper.py.hbs
      config.toml              # Backend configuration
```

### 3.2 Implementation Details

#### Backend Trait
```rust
// crates/wtlang-compiler/src/codegen/backend.rs

pub trait Backend {
    /// Backend identifier (e.g., "streamlit", "react")
    fn name(&self) -> &str;
    
    /// File extension for generated files
    fn file_extension(&self) -> &str;
    
    /// Generate complete program
    fn generate_program(&mut self, program: &Program) -> Result<HashMap<String, String>, String>;
    
    /// Additional files to generate (e.g., requirements.txt, package.json)
    fn auxiliary_files(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
```

#### Template Loader
```rust
// crates/wtlang-compiler/src/codegen/template_loader.rs

use handlebars::Handlebars;
use std::path::{Path, PathBuf};

pub struct TemplateLoader {
    hbs: Handlebars<'static>,
}

impl TemplateLoader {
    /// Load templates with fallback strategy:
    /// 1. User custom directory (~/.wtlang/templates/{backend})
    /// 2. Project local (./templates/{backend})
    /// 3. Embedded defaults (compiled into binary)
    pub fn new(backend_name: &str, embedded_templates: &[(String, String)]) 
        -> Result<Self, String> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        
        // Load embedded templates first (fallback)
        for (name, content) in embedded_templates {
            hbs.register_template_string(name, content)
                .map_err(|e| format!("Failed to register template '{}': {}", name, e))?;
        }
        
        // Try to load from project directory
        let project_templates = PathBuf::from("./templates").join(backend_name);
        if project_templates.exists() {
            Self::load_from_directory(&mut hbs, &project_templates)?;
        }
        
        // Try to load from user config directory (highest priority)
        if let Some(config_dir) = dirs::config_dir() {
            let user_templates = config_dir.join("wtlang/templates").join(backend_name);
            if user_templates.exists() {
                Self::load_from_directory(&mut hbs, &user_templates)?;
            }
        }
        
        Ok(TemplateLoader { hbs })
    }
    
    fn load_from_directory(hbs: &mut Handlebars, dir: &Path) -> Result<(), String> {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "hbs")) 
        {
            let path = entry.path();
            let relative = path.strip_prefix(dir)
                .map_err(|e| format!("Path error: {}", e))?;
            let template_name = relative.to_string_lossy()
                .trim_end_matches(".hbs")
                .replace('\\', "/");
            
            hbs.register_template_file(&template_name, path)
                .map_err(|e| format!("Failed to load template '{}': {}", template_name, e))?;
        }
        Ok(())
    }
    
    pub fn render(&self, template_name: &str, context: &impl serde::Serialize) 
        -> Result<String, String> {
        self.hbs.render(template_name, context)
            .map_err(|e| format!("Template rendering error ({}): {}", template_name, e))
    }
}
```

#### Streamlit Backend Implementation
```rust
// crates/wtlang-compiler/src/codegen/backends/streamlit/mod.rs

use crate::codegen::{Backend, TemplateLoader};
use wtlang_core::ast::*;
use serde::Serialize;

mod templates;  // Embedded templates as constants

pub struct StreamlitBackend {
    loader: TemplateLoader,
    key_counter: usize,
}

impl StreamlitBackend {
    pub fn new() -> Result<Self, String> {
        let embedded = templates::get_embedded_templates();
        let loader = TemplateLoader::new("streamlit", &embedded)?;
        Ok(StreamlitBackend { loader, key_counter: 0 })
    }
    
    fn render_statement(&mut self, stmt: &Statement, indent_level: usize) 
        -> Result<String, String> {
        let context = StatementContext::from_statement(stmt, indent_level, self)?;
        let template_name = match stmt {
            Statement::Title(_) => "statement/title",
            Statement::Subtitle(_) => "statement/subtitle",
            Statement::Text(_) => "statement/text",
            Statement::Button { .. } => "statement/button",
            Statement::Section { .. } => "statement/section",
            Statement::Let { .. } => "statement/let",
            Statement::Assign { .. } => "statement/assign",
            Statement::If { .. } => "statement/if",
            Statement::Forall { .. } => "statement/forall",
            Statement::FunctionCall(_) => "statement/function_call",
            Statement::Return(_) => "statement/return",
        };
        self.loader.render(template_name, &context)
    }
}

impl Backend for StreamlitBackend {
    fn name(&self) -> &str { "streamlit" }
    fn file_extension(&self) -> &str { "py" }
    
    fn generate_program(&mut self, program: &Program) -> Result<HashMap<String, String>, String> {
        // ... implementation using templates
    }
    
    fn auxiliary_files(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();
        files.insert(
            "requirements.txt".to_string(),
            "streamlit>=1.28.0\npandas>=2.0.0\n".to_string()
        );
        files
    }
}

#[derive(Serialize)]
struct StatementContext {
    indent: String,
    // ... fields vary by statement type
}

impl StatementContext {
    fn from_statement(stmt: &Statement, indent_level: usize, backend: &mut StreamlitBackend) 
        -> Result<Self, String> {
        // Build appropriate context for each statement type
        // ...
    }
}
```

#### Embedded Templates
```rust
// crates/wtlang-compiler/src/codegen/backends/streamlit/templates.rs

pub fn get_embedded_templates() -> Vec<(String, String)> {
    vec![
        (
            "page".to_string(),
            include_str!("../../../../../templates/streamlit/page.py.hbs").to_string()
        ),
        (
            "statement/title".to_string(),
            r#"{{indent}}st.title("{{escape text}}")"#.to_string()
        ),
        (
            "statement/button".to_string(),
            r#"{{indent}}if st.button("{{escape label}}"):
{{body}}"#.to_string()
        ),
        // ... more templates
    ]
}
```

#### Template Examples

**`templates/streamlit/page.py.hbs`**:
```handlebars
import streamlit as st
import pandas as pd
from datetime import datetime

{{#each external_imports}}
from {{this.module}} import {{#each this.functions}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
{{/each}}

{{#if needs_filter_helper}}
{{> helpers/show_filtered_helper}}
{{/if}}

# Page: {{page_name}}

{{#each statements}}
{{{this}}}
{{/each}}
```

**`templates/streamlit/statement/title.py.hbs`**:
```handlebars
{{indent}}st.title("{{escape text}}")
```

**`templates/streamlit/statement/button.py.hbs`**:
```handlebars
{{indent}}if st.button("{{escape label}}"):
{{#each body}}
{{{this}}}
{{/each}}
```

**`templates/streamlit/builtin/show_editable.py.hbs`**:
```handlebars
{{#if has_filters}}
{{indent}}_show_filtered({{table_expr}}, [{{#each filters}}('{{this.column}}', '{{this.mode}}'){{#unless @last}}, {{/unless}}{{/each}}], editable=True, key_prefix='f_{{key}}')
{{else}}
{{indent}}st.data_editor({{table_expr}}, key="editor_{{key}}", use_container_width=True)
{{/if}}
```

**`templates/streamlit/helpers/show_filtered_helper.py.hbs`**:
```python
def _show_filtered(df, filters, editable=False, key_prefix=''):
    """Show dataframe with optional filters"""
    # Create filter widgets (3 per row)
    filter_values = []
    num_filters = len(filters)
    for i in range(0, num_filters, 3):
        cols = st.columns(min(3, num_filters - i))
        for j, (col_name, mode) in enumerate(filters[i:i+3]):
            if mode == 'single':
                val = cols[j].selectbox(col_name, ['All'] + sorted(df[col_name].unique().astype(str).tolist()), key=f'{key_prefix}_f_{i+j}')
                filter_values.append((col_name, mode, val))
            else:  # multi
                val = cols[j].multiselect(col_name, sorted(df[col_name].unique().astype(str).tolist()), key=f'{key_prefix}_f_{i+j}')
                filter_values.append((col_name, mode, val))
    
    # Apply filters and track filtered/non-filtered rows
    mask = pd.Series([True] * len(df), index=df.index)
    for col_name, mode, val in filter_values:
        if mode == 'single' and val != 'All':
            mask = mask & (df[col_name].astype(str) == val)
        elif mode == 'multi' and val:
            mask = mask & df[col_name].astype(str).isin(val)
    
    filtered = df[mask]
    non_filtered = df[~mask]
    
    # Display
    if editable:
        edited = st.data_editor(filtered, key=f'{key_prefix}_editor', use_container_width=True)
        # Merge edited filtered rows with non-filtered rows
        return pd.concat([edited, non_filtered], ignore_index=True)
    else:
        st.dataframe(filtered)
        return None
```

**`templates/streamlit/config.toml`**:
```toml
[backend]
name = "streamlit"
description = "Python/Streamlit backend for web applications"
file_extension = "py"

[requirements]
streamlit = ">=1.28.0"
pandas = ">=2.0.0"

[features]
filtering = true
editable_tables = true
multipage = true
external_functions = true

[customization]
# Users can override these in their config
indent_spaces = 4
max_filters_per_row = 3
use_container_width = true
```

### 3.3 User Workflow

#### Basic Usage (No Customization)
```bash
# Default backend (Streamlit) with embedded templates
wtc compile myapp.wt
# Generates: myapp.py using built-in templates
```

#### Project-Level Customization
```bash
# Create project templates
mkdir -p templates/streamlit/statement
cp ~/.wtlang/default_templates/streamlit/statement/title.py.hbs templates/streamlit/statement/

# Edit template
# templates/streamlit/statement/title.py.hbs:
# {{indent}}st.title("📊 {{escape text}}")  # Added emoji!

wtc compile myapp.wt
# Now all titles have emoji prefix
```

#### User-Level Customization
```bash
# Install custom templates globally
mkdir -p ~/.config/wtlang/templates/streamlit/statement
cat > ~/.config/wtlang/templates/streamlit/statement/title.py.hbs <<EOF
{{indent}}st.markdown("## {{escape text}}")  # Use markdown instead of st.title
EOF

wtc compile myapp.wt
# All projects now use markdown titles by default
```

#### Backend Selection
```bash
# Use different backend (future)
wtc compile --backend react myapp.wt
wtc compile --backend jupyter myapp.wt
```

---

## 4. Implementation Plan

### Phase 1: Refactor Current Generator (Week 1-2)

**Goal**: Restructure existing code without changing functionality

1. **Extract Statement/Expression Generators** (2 days)
   - Create separate methods for each statement type
   - Extract expression generation logic
   - Add proper error handling

2. **Create Template Contexts** (2 days)
   - Define `PageContext`, `StatementContext`, `ExprContext` structs
   - Implement `Serialize` for all contexts
   - Build context from AST nodes

3. **Modularize Backend Logic** (3 days)
   - Create `Backend` trait
   - Extract Streamlit-specific code into `StreamlitBackend`
   - Implement registry pattern

4. **Add Tests** (2 days)
   - Unit tests for each generator method
   - Integration tests for full compilation
   - Snapshot tests for generated code

**Deliverable**: Working compiler with better structure, same output

### Phase 2: Implement Template System (Week 3-4)

**Goal**: Replace string generation with templates

1. **Add Template Engine** (2 days)
   - Add `handlebars` dependency
   - Create `TemplateLoader` with fallback logic
   - Implement template discovery

2. **Create Template Files** (4 days)
   - Write templates for all statement types
   - Create expression templates
   - Add helper templates (imports, filter helper, etc.)
   - Document template variables/helpers

3. **Embed Templates** (1 day)
   - Add `include_str!` for all templates
   - Create template registry
   - Test embedded fallback

4. **Integration** (2 days)
   - Wire template rendering into backend
   - Handle indentation properly
   - Test output matches current generator

5. **Documentation** (1 day)
   - Template customization guide
   - Template reference documentation
   - Example customizations

**Deliverable**: Template-based generator with identical output

### Phase 3: External Template Support (Week 5)

**Goal**: Enable user customization

1. **Template Loading Logic** (2 days)
   - Implement directory scanning
   - Add config directory support
   - Handle template overrides

2. **Configuration** (1 day)
   - Add backend configuration files
   - Support user config file (`~/.wtlangrc`)
   - CLI flags for template directory

3. **Error Handling** (1 day)
   - Better template error messages
   - Template validation
   - Fallback on template errors

4. **Testing & Polish** (1 day)
   - Test custom template loading
   - Error message improvements
   - Performance benchmarks

**Deliverable**: Fully customizable code generation

### Phase 4: Documentation & Examples (Week 6)

**Goal**: Help users leverage new system

1. **User Documentation** (2 days)
   - Template customization tutorial
   - Available template variables reference
   - Common customization recipes
   - Troubleshooting guide

2. **Example Templates** (2 days)
   - Create example customizations
   - Alternative styling templates
   - Advanced filter templates
   - Performance-optimized templates

3. **Developer Documentation** (1 day)
   - Backend development guide
   - Adding new backends tutorial
   - Template system architecture

**Deliverable**: Complete documentation suite

### Phase 5: Future Backend Preparation (Optional)

**Goal**: Validate architecture with second backend

1. **Create React Backend** (5 days)
   - Define React templates
   - Implement `ReactBackend`
   - Generate TypeScript/TSX code
   - Handle state management

2. **Jupyter Backend** (3 days)
   - Define notebook templates
   - Generate `.ipynb` JSON
   - Cell-based code organization

**Deliverable**: Multi-backend compiler

---

## 5. Impact Analysis

### 5.1 Breaking Changes

**None** - This is a pure refactoring. Output remains identical for default configuration.

### 5.2 Migration Path

No migration needed for users. Internally:
1. Current `codegen.rs` becomes `backends/streamlit/mod.rs`
2. String generation becomes template rendering
3. All tests should pass unchanged

### 5.3 Performance Impact

**Template Overhead**: 
- Initial template loading: ~10-50ms (one-time)
- Template rendering: ~0.1-0.5ms per template
- Total impact: <5% for typical projects (dozens of pages)

**Mitigation**:
- Template caching (already in Handlebars)
- Lazy template loading
- Pre-compiled templates option for production

**Benchmark** (estimated for 100-page project):
```
Current:     ~500ms compilation
Templated:   ~520ms compilation (+4%)
```

### 5.4 Maintenance Impact

**Positive**:
- ✅ Easier to modify code patterns (edit templates, not Rust)
- ✅ Less code duplication
- ✅ Better separation of concerns
- ✅ Easier testing (snapshot test templates)

**Negative**:
- ❌ Two places to maintain (Rust logic + templates)
- ❌ Template syntax errors possible
- ❌ More files to manage

**Net Impact**: Significant improvement in maintainability

### 5.5 User Impact

**Developers**:
- ✅ Can customize code generation without forking
- ✅ Company-specific code standards enforceable
- ✅ Experiment with output optimizations
- ✅ Debug generated code more easily

**End Users**:
- No direct impact (transparent change)

---

## 6. Alternative Future Enhancements

### 6.1 Code Generation Hooks

Allow users to inject custom code at specific points:

**Configuration** (`wtlang.toml`):
```toml
[codegen.hooks]
page_start = """
# Custom company header
# Generated by WTLang on {{date}}
# Proprietary and Confidential
"""

page_end = """
# Analytics tracking
track_page_view("{{page_name}}")
"""
```

### 6.2 Partial Template Override

Instead of full template replacement, allow partial overrides:

**`templates/streamlit/overrides.toml`**:
```toml
[statement.title]
prefix = "st.markdown(\"# "  # Use markdown instead of st.title
suffix = "\")"
wrapper = "with st.container():\n{content}"
```

### 6.3 Template Composition

Allow templates to compose/extend each other:

```handlebars
{{! templates/streamlit/statement/enhanced_button.py.hbs }}
{{#extend "statement/button"}}
  {{#before_button}}
  {{indent}}# Analytics: Button {{label}} clicked
  {{/before_button}}
{{/extend}}
```

### 6.4 Multi-File Templates

Generate auxiliary files from templates:

```
templates/streamlit/
  page.py.hbs           # Main page file
  page.test.py.hbs      # Pytest test file
  page.spec.md.hbs      # Documentation
```

### 6.5 Optimization Profiles

Different template sets for different scenarios:

```bash
wtc compile --profile debug myapp.wt    # Verbose, instrumented code
wtc compile --profile production myapp.wt  # Optimized, minified
wtc compile --profile demo myapp.wt    # Sample data, no DB calls
```

---

## 7. Security Considerations

### 7.1 Template Injection

**Risk**: Malicious templates could generate harmful code

**Mitigation**:
- Sandbox template rendering (Handlebars is safe by default)
- Validate template syntax before execution
- Sign official templates
- Warn on custom template usage

### 7.2 Path Traversal

**Risk**: Template loading from `../../../etc/passwd`

**Mitigation**:
- Canonicalize all paths
- Restrict template loading to specific directories
- Never load from arbitrary user input

### 7.3 Code Injection

**Risk**: User input in templates could inject Python code

**Mitigation**:
- Always escape user-provided text in templates
- Use Handlebars `{{escape}}` helper
- Never use raw/unescaped interpolation for user data

---

## 8. Testing Strategy

### 8.1 Template Tests

**Snapshot Testing**:
```rust
#[test]
fn test_title_template() {
    let backend = StreamlitBackend::new().unwrap();
    let stmt = Statement::Title("Hello World".to_string());
    let output = backend.render_statement(&stmt, 0).unwrap();
    insta::assert_snapshot!(output);
}
```

### 8.2 Integration Tests

**Full Compilation Tests**:
```rust
#[test]
fn test_example_compilation() {
    let source = fs::read_to_string("examples/01_hello.wt").unwrap();
    let output = compile(&source, "streamlit").unwrap();
    
    // Check output is valid Python
    assert!(python_syntax_check(&output["Home.py"]));
    
    // Snapshot test
    insta::assert_snapshot!(output["Home.py"]);
}
```

### 8.3 Custom Template Tests

**User Template Loading**:
```rust
#[test]
fn test_custom_template_override() {
    let temp_dir = setup_custom_templates();
    let backend = StreamlitBackend::with_template_dir(temp_dir).unwrap();
    
    // Custom template should be used
    let output = backend.render_statement(...).unwrap();
    assert!(output.contains("CUSTOM"));
}
```

### 8.4 Multi-Backend Tests

**Consistency Tests**:
```rust
#[test]
fn test_all_backends_compile() {
    let source = "page Home { title \"Test\" }";
    
    for backend in ["streamlit", "react", "jupyter"] {
        let output = compile(source, backend);
        assert!(output.is_ok(), "Backend {} failed", backend);
    }
}
```

---

## 9. IR-Based Architecture: Extended Analysis for Tooling Ecosystem

### 9.1 Overview: Why IR Matters Beyond Code Generation

While the hybrid template approach (Section 3) provides excellent user customization and multi-backend support, an **Intermediate Representation (IR)** layer offers critical advantages for the broader tooling ecosystem:

1. **Platform-Independent Analysis**: Tools like LSP and debugger can operate on IR without understanding backend specifics
2. **Optimization Opportunities**: IR enables transformations that benefit all backends
3. **Consistent Semantics**: Single source of truth for program behavior
4. **Tool Interoperability**: Shared IR allows tools to exchange information

This section analyzes the benefits of IR for the complete WTLang tooling ecosystem and proposes a **three-layer architecture**: AST → IR → Template-based Code Generation.

---

### 9.2 IR Benefits for Language Server Protocol (LSP)

#### 9.2.1 Symbol Table and Type Information

**Current Challenge**: LSP duplicates semantic analysis from compiler

**With IR**:
```rust
// Compiler generates IR with complete symbol information
pub struct IRModule {
    symbols: SymbolTable,      // All declarations with types and scopes
    type_env: TypeEnvironment,  // Resolved types for all expressions
    ir_nodes: Vec<IRNode>,
}

// LSP can reuse this for:
impl LanguageServer {
    fn hover(&self, position: Position) -> Hover {
        let ir = self.compile_to_ir(document)?;
        let symbol = ir.symbols.lookup_at_position(position)?;
        
        Hover {
            contents: format!("{}: {}", symbol.name, symbol.ty),
            // Type info already resolved in IR
        }
    }
    
    fn autocomplete(&self, position: Position) -> Vec<CompletionItem> {
        let ir = self.compile_to_ir(document)?;
        // IR knows all visible symbols in scope
        ir.symbols.visible_at(position)
            .map(|sym| CompletionItem::from_symbol(sym))
            .collect()
    }
}
```

**Benefits**:
- ✅ No duplicate type inference logic
- ✅ Consistent type information between compiler and LSP
- ✅ Faster hover/completion (type info pre-computed)

#### 9.2.2 Semantic Understanding

**Use Case**: Find all references to a variable

**With IR**:
```rust
pub struct IRNode {
    source_location: SourceRange,  // Maps back to source code
    // ...
}

impl LanguageServer {
    fn references(&self, symbol_name: &str) -> Vec<Location> {
        let ir = self.compile_to_ir(document)?;
        
        // IR has explicit def-use chains
        ir.nodes.iter()
            .filter_map(|node| match node {
                IRNode::Identifier(name, loc) if name == symbol_name => Some(*loc),
                _ => None
            })
            .collect()
    }
}
```

**Benefits**:
- ✅ Accurate reference finding (respects scopes)
- ✅ Cross-page references (IR links pages)
- ✅ Rename refactoring safety (type-aware)

#### 9.2.3 Cross-Backend Validation

**Scenario**: User writes code valid for Streamlit but not React

**With IR**:
```rust
impl IRNode {
    fn validate_for_backend(&self, backend: &str) -> Vec<Diagnostic> {
        match (self, backend) {
            (IRNode::ShowEditable { filters, .. }, "react") if filters.len() > 5 => {
                vec![Diagnostic::warning(
                    "React backend supports max 5 filters, some will be ignored"
                )]
            }
            (IRNode::ExternalFunction { lang: "python", .. }, "jupyter") => {
                vec![Diagnostic::info(
                    "External Python functions run in same kernel in Jupyter"
                )]
            }
            _ => vec![]
        }
    }
}
```

**Benefits**:
- ✅ Backend-specific warnings in LSP
- ✅ Help users write portable code
- ✅ Show feature availability per backend

#### 9.2.4 Advanced Refactoring Support

**Example**: Extract function refactoring

**With IR**:
```rust
impl LanguageServer {
    fn extract_function(&self, selection: Range) -> WorkspaceEdit {
        let ir = self.compile_to_ir(document)?;
        
        // Analyze selected IR nodes
        let selected_nodes = ir.nodes_in_range(selection);
        
        // Determine required parameters (free variables)
        let free_vars = ir.compute_free_variables(selected_nodes);
        
        // Determine return type (last expression type)
        let return_type = ir.infer_type(selected_nodes.last());
        
        // Generate function signature
        let signature = FunctionSignature {
            name: "extracted_fn",
            params: free_vars.into_iter().map(|(name, ty)| Param { name, ty }).collect(),
            return_type,
        };
        
        // Create edits
        WorkspaceEdit {
            // Add function definition
            // Replace selection with function call
        }
    }
}
```

**Benefits**:
- ✅ Safe refactoring (type-aware)
- ✅ Automatic parameter inference
- ✅ Preserve semantics

---

### 9.3 IR Benefits for Debugger (Future)

#### 9.3.1 Source-to-Target Mapping

**Challenge**: User sets breakpoint in `.wt` file, need to translate to generated `.py` line

**With IR**:
```rust
pub struct IRNode {
    source_location: SourceRange,    // Original .wt file location
    target_location: Option<TargetLocation>,  // Generated code location
    // ...
}

pub struct SourceMap {
    ir_to_source: HashMap<IRNodeId, SourceRange>,
    ir_to_target: HashMap<IRNodeId, TargetLocation>,
}

impl Debugger {
    fn set_breakpoint(&mut self, wt_file: &str, line: usize) -> Result<(), String> {
        let ir = self.compile_to_ir(wt_file)?;
        
        // Find IR node at source line
        let ir_node = ir.node_at_line(line)?;
        
        // Map to generated code
        let target_loc = self.source_map.ir_to_target(ir_node)?;
        
        // Set breakpoint in actual Python debugger
        self.python_debugger.set_breakpoint(target_loc.file, target_loc.line)?;
        
        Ok(())
    }
    
    fn on_breakpoint_hit(&self, py_location: Location) -> SourceLocation {
        // Map back to .wt file
        let ir_node = self.source_map.target_to_ir(py_location);
        self.source_map.ir_to_source(ir_node)
    }
}
```

**Benefits**:
- ✅ Accurate breakpoint translation
- ✅ Step through .wt code, not generated code
- ✅ Show .wt file in debugger UI

#### 9.3.2 Expression Evaluation (Watch Windows)

**Challenge**: User wants to evaluate WTLang expression in debugger

**With IR**:
```rust
impl Debugger {
    fn evaluate_watch(&self, expression: &str) -> Result<Value, String> {
        // Parse and compile to IR
        let expr_ir = self.compile_expression_to_ir(expression)?;
        
        // Get current runtime state from Python debugger
        let runtime_state = self.python_debugger.get_locals()?;
        
        // Evaluate IR expression against runtime state
        let result = self.ir_evaluator.eval(&expr_ir, &runtime_state)?;
        
        Ok(result)
    }
}
```

**Example**:
```
// User types in watch window: "users |> where(u => u.age > 18)"
// IR understands pipeline semantics
// Translates to Python and evaluates
// Shows result in debugger
```

**Benefits**:
- ✅ Evaluate WTLang expressions at runtime
- ✅ No need to know generated code
- ✅ Full language support in watch windows

#### 9.3.3 Runtime Validation

**Use Case**: Detect schema mismatches at runtime

**With IR**:
```rust
pub struct IRNode {
    expected_type: Type,
    runtime_checks: Vec<Check>,
    // ...
}

// Generated code includes runtime checks
impl StreamlitBackend {
    fn generate_load_csv(&self, node: &IRNode) -> String {
        let checks = node.runtime_checks.iter()
            .map(|check| self.generate_check(check))
            .collect::<Vec<_>>()
            .join("\n");
        
        format!(
            r#"
{var} = pd.read_csv({path})
{checks}
if __debug__ and __wtlang_debug__:
    _wtlang_debugger.validate_type({var}, {expected_type})
            "#,
            var = node.var_name,
            path = node.file_path,
            checks = checks,
            expected_type = self.serialize_type(&node.expected_type),
        )
    }
}
```

**Benefits**:
- ✅ Debugger shows type mismatches in WTLang terms
- ✅ Better error messages
- ✅ Catch bugs earlier

#### 9.3.4 Time-Travel Debugging

**Advanced Feature**: Step backwards through execution

**With IR**:
```rust
// IR enables recording state changes
pub struct IRStateChange {
    ir_node: IRNodeId,
    variable: String,
    old_value: Value,
    new_value: Value,
    timestamp: Instant,
}

impl Debugger {
    fn step_back(&mut self) {
        // Get last state change
        let change = self.state_history.pop();
        
        // Show in UI which IR node caused this change
        let source_loc = self.ir.node_location(change.ir_node);
        self.ui.highlight(source_loc);
        
        // Restore state
        self.runtime.set_variable(change.variable, change.old_value);
    }
}
```

**Benefits**:
- ✅ Understand program flow
- ✅ Debug complex pipelines
- ✅ See variable history

---

### 9.4 Additional IR Benefits

#### 9.4.1 Platform Normalization

**Problem**: Different backends have different capabilities

**With IR**:
```rust
pub enum IRNode {
    // High-level, platform-independent operations
    FilteredTable {
        table: Box<IRExpr>,
        filters: Vec<Filter>,
        editable: bool,
    },
    // Backend decides how to implement
}

impl StreamlitBackend {
    fn generate_filtered_table(&self, node: &IRNode) -> String {
        // Streamlit uses st.multiselect + st.data_editor
        // ...
    }
}

impl ReactBackend {
    fn generate_filtered_table(&self, node: &IRNode) -> String {
        // React uses <Select> + <DataGrid>
        // ...
    }
}

impl JupyterBackend {
    fn generate_filtered_table(&self, node: &IRNode) -> String {
        // Jupyter uses ipywidgets
        // ...
    }
}
```

**Benefits**:
- ✅ Write once, run anywhere
- ✅ Backend-specific optimizations
- ✅ Consistent semantics across platforms

#### 9.4.2 Optimization Opportunities

**Example 1: Pipeline Fusion**

**Source**:
```wtlang
users |> where(u => u.active) |> where(u => u.age > 18)
```

**Without IR**: Two separate filters in generated code

**With IR Optimization**:
```rust
impl IROptimizer {
    fn fuse_filters(&self, ir: IRNode) -> IRNode {
        match ir {
            IRNode::Chain(
                box IRNode::Where(table1, cond1),
                box IRNode::Where(table2, cond2)
            ) if table1 == table2 => {
                // Combine conditions
                IRNode::Where(
                    table1,
                    IRExpr::And(box cond1, box cond2)
                )
            }
            _ => ir
        }
    }
}
```

**Generated Code** (optimized):
```python
users[users.active & (users.age > 18)]  # Single filter!
```

**Example 2: Constant Folding**

**Source**:
```wtlang
let max_age: int = 100
let min_age: int = max_age - 82
```

**IR Optimization**:
```rust
IRNode::Let {
    name: "min_age",
    value: IRExpr::Literal(18),  // Computed at compile time!
}
```

**Example 3: Dead Code Elimination**

**Source**:
```wtlang
let temp: table = users |> where(u => u.active)
// temp is never used
show(users)
```

**IR Optimization**: Remove unreferenced variable

**Benefits**:
- ✅ Faster generated code
- ✅ Smaller output
- ✅ Better performance

#### 9.4.3 Static Analysis

**Type Safety Checks**:
```rust
impl IRValidator {
    fn validate(&self, ir: &IRNode) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        
        // Check: table operations on non-tables
        if let IRNode::Where(expr, _) = ir {
            if expr.get_type() != Type::Table(_) {
                diagnostics.push(Diagnostic::error(
                    "where() requires table, got {}",
                    expr.get_type()
                ));
            }
        }
        
        // Check: field access on wrong table type
        if let IRNode::FieldAccess(table_expr, field) = ir {
            if let Type::Table(schema) = table_expr.get_type() {
                if !schema.has_field(field) {
                    diagnostics.push(Diagnostic::error(
                        "Table has no field '{}'", field
                    ));
                }
            }
        }
        
        diagnostics
    }
}
```

**Dataflow Analysis**:
```rust
impl IRAnalyzer {
    fn check_uninitialized_variables(&self, ir: &IRModule) -> Vec<Diagnostic> {
        let mut initialized = HashSet::new();
        let mut diagnostics = vec![];
        
        for node in &ir.nodes {
            match node {
                IRNode::Let { name, value: Some(_), .. } => {
                    initialized.insert(name);
                }
                IRNode::Let { name, value: None, .. } => {
                    // Declaration without initialization
                }
                IRNode::Identifier(name, loc) if !initialized.contains(name) => {
                    diagnostics.push(Diagnostic::error_at(
                        *loc,
                        "Variable '{}' used before initialization", name
                    ));
                }
                _ => {}
            }
        }
        
        diagnostics
    }
}
```

**Benefits**:
- ✅ Catch errors earlier (compile time vs runtime)
- ✅ Better error messages
- ✅ More reliable code

#### 9.4.4 Incremental Compilation

**Scenario**: User edits one page in a multi-page app

**Without IR**: Recompile entire project

**With IR**:
```rust
pub struct CompilationCache {
    ir_cache: HashMap<PathBuf, (IRModule, Timestamp)>,
}

impl Compiler {
    fn compile_incremental(&mut self, changed_file: &Path) -> Result<(), String> {
        // Only recompile changed file to IR
        let new_ir = self.compile_file_to_ir(changed_file)?;
        
        // Check if IR actually changed (semantic change vs formatting)
        if let Some((old_ir, _)) = self.cache.ir_cache.get(changed_file) {
            if old_ir.semantically_equal(&new_ir) {
                // No semantic change, skip code generation
                return Ok(());
            }
        }
        
        // Update cache
        self.cache.ir_cache.insert(changed_file.to_path_buf(), (new_ir.clone(), now()));
        
        // Only regenerate code for this file
        self.backend.generate_module(&new_ir)?;
        
        Ok(())
    }
}
```

**Benefits**:
- ✅ Faster recompilation
- ✅ Better developer experience
- ✅ Efficient CI/CD builds

#### 9.4.5 Better Testing

**IR-Level Tests** (backend-independent):
```rust
#[test]
fn test_filter_optimization() {
    let source = r#"
        users |> where(u => u.active) |> where(u => u.age > 18)
    "#;
    
    let ir = compile_to_ir(source).unwrap();
    let optimized = optimize(ir);
    
    // Check IR structure, not generated code
    assert_matches!(optimized, IRNode::Where(_, IRExpr::And(..)));
}

#[test]
fn test_type_inference() {
    let source = "let x = 5 + 3";
    let ir = compile_to_ir(source).unwrap();
    
    assert_eq!(ir.get_type("x"), Type::Int);
}
```

**Cross-Backend Tests**:
```rust
#[test]
fn test_all_backends_semantically_equivalent() {
    let source = load_test_program("examples/01_hello.wt");
    let ir = compile_to_ir(source).unwrap();
    
    // Generate code for all backends
    let streamlit_code = StreamlitBackend::new().generate(&ir)?;
    let react_code = ReactBackend::new().generate(&ir)?;
    let jupyter_code = JupyterBackend::new().generate(&ir)?;
    
    // All should produce equivalent UI (test via snapshots or execution)
    // ...
}
```

**Benefits**:
- ✅ Test semantics, not syntax
- ✅ Verify optimizations correct
- ✅ Ensure backend consistency

---

### 9.5 Recommended Architecture: Three-Layer Hybrid

Based on the analysis above, we recommend a **three-layer architecture** that combines the best of IR-based analysis and template-based generation:

```
┌─────────────────────────────────────────────────────────────┐
│                        Source Code (.wt)                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
                    ┌─────────────────┐
                    │  Lexer + Parser │
                    └─────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                     AST (wtlang-core)                        │
│  - Syntax tree                                               │
│  - Source locations                                          │
└─────────────────────────────────────────────────────────────┘
                              ↓
                  ┌──────────────────────┐
                  │ Semantic Analysis    │
                  │ - Type checking      │
                  │ - Symbol resolution  │
                  │ - Scope analysis     │
                  └──────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│          IR (Intermediate Representation Layer)              │
│  - Platform-independent operations                           │
│  - Fully typed and annotated                                 │
│  - Symbol table + type environment                           │
│  - Source-to-IR mapping                                      │
│                                                               │
│  Used by:                                                    │
│  • LSP (hover, completion, references, refactoring)          │
│  • Debugger (breakpoints, watches, source maps)              │
│  • Optimizer (pipeline fusion, constant folding)             │
│  • Static Analyzer (unused vars, type errors)                │
└─────────────────────────────────────────────────────────────┘
                              ↓
                     ┌────────────────┐
                     │  IR Optimizer  │
                     │  (Optional)    │
                     └────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              Backend Code Generator + Templates              │
│  - IR → Template Context conversion                          │
│  - Template rendering (Handlebars)                           │
│  - Backend-specific idioms                                   │
│  - User-customizable templates                               │
│                                                               │
│  Backends:                                                   │
│  • StreamlitBackend (Python)                                 │
│  • ReactBackend (TypeScript/TSX)                             │
│  • JupyterBackend (Notebook JSON)                            │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Generated Code                            │
│  • Home.py, Dashboard.py (Streamlit)                         │
│  • App.tsx, components/* (React)                             │
│  • notebook.ipynb (Jupyter)                                  │
└─────────────────────────────────────────────────────────────┘
```

#### Layer 1: AST (Already Implemented)
- Preserves source structure
- Used for syntax highlighting, formatting
- Input to semantic analysis

#### Layer 2: IR (New - Core of this proposal)
```rust
// crates/wtlang-core/src/ir.rs

pub struct IRModule {
    pub name: String,
    pub imports: Vec<Import>,
    pub items: Vec<IRItem>,
    pub symbols: SymbolTable,
    pub type_env: TypeEnvironment,
}

pub enum IRItem {
    TableDef {
        name: String,
        schema: TableSchema,
        source_loc: SourceRange,
    },
    FunctionDef {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    PageDef {
        name: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
}

pub enum IRNode {
    // UI Operations
    ShowTable {
        table: Box<IRExpr>,
        filters: Vec<Filter>,
        editable: bool,
        key: String,
        source_loc: SourceRange,
        target_loc: Option<TargetLocation>,  // Filled during codegen
    },
    ShowText {
        text: String,
        style: TextStyle,
        source_loc: SourceRange,
    },
    
    // Control Flow
    Conditional {
        condition: Box<IRExpr>,
        then_branch: Vec<IRNode>,
        else_branch: Option<Vec<IRNode>>,
        source_loc: SourceRange,
    },
    Loop {
        variable: String,
        iterable: Box<IRExpr>,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    // Data Operations
    Pipeline {
        stages: Vec<PipelineStage>,
        result_type: Type,
        source_loc: SourceRange,
    },
    
    // Variables
    Binding {
        name: String,
        ty: Type,
        value: Option<Box<IRExpr>>,
        source_loc: SourceRange,
    },
    Assignment {
        name: String,
        value: Box<IRExpr>,
        source_loc: SourceRange,
    },
    
    // Expressions (simplified)
    Expr(IRExpr),
}

pub enum IRExpr {
    Literal { value: Literal, ty: Type },
    Variable { name: String, ty: Type },
    BinaryOp { op: BinOp, left: Box<IRExpr>, right: Box<IRExpr>, ty: Type },
    FunctionCall { name: String, args: Vec<IRExpr>, ty: Type },
    FieldAccess { object: Box<IRExpr>, field: String, ty: Type },
    // ... more
}

impl IRModule {
    /// Convert from AST
    pub fn from_ast(program: &Program, symbols: SymbolTable) -> Result<Self, String> {
        // AST lowering logic
    }
    
    /// Optimize IR
    pub fn optimize(&mut self) {
        // Apply optimization passes
    }
    
    /// Validate semantics
    pub fn validate(&self) -> Vec<Diagnostic> {
        // Type checking, unused variable detection, etc.
    }
}
```

#### Layer 3: Template-Based Backends (From Section 3)
- Converts IR to backend-specific code
- Uses templates for flexibility
- Generates auxiliary files

---

### 9.6 Implementation Strategy for IR + Templates

#### Phase 1: IR Foundation (Weeks 1-3)

**Week 1: IR Data Structures**
```rust
// Define IR types in wtlang-core/src/ir.rs
pub mod ir {
    mod types;      // IR type system
    mod nodes;      // IR node definitions
    mod module;     // IR module structure
    mod builder;    // AST → IR conversion
}
```

**Week 2: AST → IR Lowering**
```rust
impl IRBuilder {
    pub fn lower_program(&mut self, program: &Program) -> Result<IRModule, String> {
        let mut ir_items = Vec::new();
        
        for item in &program.items {
            match item {
                Item::TableDef(table) => {
                    ir_items.push(self.lower_table_def(table)?);
                }
                Item::PageDef(page) => {
                    ir_items.push(self.lower_page_def(page)?);
                }
                // ...
            }
        }
        
        Ok(IRModule {
            items: ir_items,
            symbols: self.symbols.clone(),
            type_env: self.type_env.clone(),
        })
    }
}
```

**Week 3: IR Validation & Testing**
- Implement IR validator
- Add IR-level tests
- Ensure semantic preservation (AST ≡ IR)

**Deliverable**: Compiler can generate IR from source

#### Phase 2: Update LSP to Use IR (Weeks 4-5)

**Week 4: LSP Integration**
```rust
// crates/wtlang-lsp/src/analysis.rs

impl LanguageServer {
    fn analyze_document(&mut self, uri: &Url) -> Result<IRModule, String> {
        let source = self.documents.get(uri)?;
        
        // Compile to IR (includes semantic analysis)
        let ir = compile_to_ir(source)?;
        
        // Cache IR for future requests
        self.ir_cache.insert(uri.clone(), ir.clone());
        
        Ok(ir)
    }
    
    fn handle_hover(&self, params: HoverParams) -> Option<Hover> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let ir = self.ir_cache.get(&uri)?;
        let symbol = ir.symbols.lookup_at_position(position)?;
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```wtlang\n{}: {}\n```\n{}",
                    symbol.name, symbol.ty, symbol.doc
                ),
            }),
            range: Some(symbol.range),
        })
    }
}
```

**Week 5: Enhanced LSP Features**
- Implement reference finding using IR
- Add rename refactoring
- Cross-file symbol resolution

**Deliverable**: LSP uses IR for all analysis

#### Phase 3: Refactor Codegen to Use IR (Weeks 6-7)

**Week 6: IR → Template Context**
```rust
// crates/wtlang-compiler/src/codegen/context.rs

#[derive(Serialize)]
pub struct PageContext {
    pub name: String,
    pub imports: Vec<ImportContext>,
    pub statements: Vec<String>,  // Pre-rendered
    pub helpers: Vec<String>,
}

impl PageContext {
    pub fn from_ir(ir_page: &IRItem, backend: &dyn Backend) -> Result<Self, String> {
        match ir_page {
            IRItem::PageDef { name, body, .. } => {
                let mut statements = Vec::new();
                for node in body {
                    statements.push(backend.render_node(node)?);
                }
                
                Ok(PageContext {
                    name: name.clone(),
                    imports: backend.compute_imports(body),
                    statements,
                    helpers: backend.required_helpers(body),
                })
            }
            _ => Err("Not a page".to_string())
        }
    }
}
```

**Week 7: Backend Implementations**
```rust
impl Backend for StreamlitBackend {
    fn generate_from_ir(&mut self, ir: &IRModule) -> Result<HashMap<String, String>, String> {
        let mut files = HashMap::new();
        
        for item in &ir.items {
            match item {
                IRItem::PageDef { name, .. } => {
                    let context = PageContext::from_ir(item, self)?;
                    let code = self.templates.render("page", &context)?;
                    files.insert(format!("{}.py", name), code);
                }
                _ => {}
            }
        }
        
        Ok(files)
    }
    
    fn render_node(&self, node: &IRNode) -> Result<String, String> {
        match node {
            IRNode::ShowTable { table, filters, editable, key, .. } => {
                let context = ShowTableContext {
                    table_expr: self.render_expr(table)?,
                    filters: filters.iter().map(|f| self.render_filter(f)).collect(),
                    editable: *editable,
                    key: key.clone(),
                    has_filters: !filters.is_empty(),
                };
                self.templates.render("builtin/show_editable", &context)
            }
            // ...
        }
    }
}
```

**Deliverable**: Code generation uses IR as input

#### Phase 4: IR Optimizations (Week 8)

**Implement Optimization Passes**:
```rust
pub fn optimize_ir(ir: IRModule) -> IRModule {
    let ir = constant_folding(ir);
    let ir = pipeline_fusion(ir);
    let ir = dead_code_elimination(ir);
    let ir = inline_simple_functions(ir);
    ir
}
```

**Deliverable**: Optimized code generation

#### Phase 5: Source Mapping for Debugger (Week 9)

**Generate Source Maps**:
```rust
pub struct SourceMap {
    mappings: Vec<Mapping>,
}

pub struct Mapping {
    ir_node_id: usize,
    source_file: PathBuf,
    source_range: SourceRange,
    target_file: PathBuf,
    target_range: TargetRange,
}

impl Backend {
    fn generate_with_source_map(&mut self, ir: &IRModule) 
        -> Result<(HashMap<String, String>, SourceMap), String> {
        // Generate code and track mappings
    }
}
```

**Deliverable**: Source maps for future debugger

#### Phase 6: Documentation & Polish (Week 10)

- Document IR design
- Update architecture diagrams
- Write IR optimization guide
- Create debugging guide

**Deliverable**: Complete documentation

---

### 9.7 Comparison: Hybrid Template vs IR + Templates

| Aspect | Hybrid Template (Section 3) | IR + Templates (Section 9) |
|--------|----------------------------|----------------------------|
| **Code Generation** | ✅ Template-based | ✅ Template-based |
| **User Customization** | ✅ Full template editing | ✅ Full template editing |
| **Multi-Backend** | ✅ Easy to add | ✅ Easy to add |
| **LSP Type Info** | ⚠️ Duplicate analysis | ✅ Shared IR |
| **LSP Refactoring** | ❌ Limited | ✅ Advanced |
| **Debugger Support** | ❌ Hard to implement | ✅ Source maps built-in |
| **Optimizations** | ❌ Per-backend | ✅ Cross-backend |
| **Static Analysis** | ⚠️ Limited | ✅ Comprehensive |
| **Incremental Compilation** | ❌ No | ✅ Yes |
| **Development Time** | 6 weeks | 10 weeks |
| **Complexity** | Medium | High |
| **Long-term Benefits** | Good | Excellent |

---

### 9.8 Decision Matrix

#### Choose Hybrid Template (Section 3) if:
- ❌ No plans for debugger soon
- ❌ LSP features stay basic (hover + completion only)
- ❌ Single backend is enough
- ✅ Want faster implementation (6 weeks vs 10 weeks)
- ✅ Simpler architecture preferred

#### Choose IR + Templates (Section 9) if:
- ✅ Planning debugger implementation
- ✅ Want advanced LSP (refactoring, navigation)
- ✅ Multiple backends needed
- ✅ Want optimization opportunities
- ✅ Long-term investment in tooling
- ✅ Static analysis important

---

### 9.9 Recommended Path: Incremental Adoption

**Best of Both Worlds**: Start with Hybrid, migrate to IR incrementally

#### Stage 1: Hybrid Template (Now)
- Implement Section 3 architecture (6 weeks)
- Get user customization and multi-backend support
- Ship and gather feedback

#### Stage 2: Add Minimal IR (Later - 3 months)
- Create basic IR types
- Use for LSP only (not codegen yet)
- Keep existing template-based codegen

#### Stage 3: Migrate Codegen to IR (Later - 6 months)
- Refactor backends to consume IR
- Add optimizations
- Implement source maps

#### Stage 4: Advanced Features (Later - 12 months)
- Debugger using source maps
- Advanced refactoring
- Static analysis

**Benefits of Incremental Approach**:
- ✅ Ship value quickly (6 weeks)
- ✅ Learn from user feedback before big investment
- ✅ Lower risk (can stop after each stage)
- ✅ Distribute development cost over time

---

## 10. Conclusion

### 10.1 Updated Recommendation Summary

**For Immediate Implementation (Step 18)**: 
Adopt the **IR + Templates architecture** (Section 9.5) with **incremental rollout** (Section 9.9).

**Rationale**:
1. ✅ **LSP Benefits**: IR enables advanced features (refactoring, cross-file analysis) that users expect
2. ✅ **Future Debugger**: Source maps and type information essential for debugging experience
3. ✅ **Optimizations**: Pipeline fusion and other opts improve generated code quality
4. ✅ **Multi-Backend**: IR normalizes platform differences better than templates alone
5. ✅ **Static Analysis**: Catch more errors at compile time, better user experience
6. ✅ **Template Flexibility**: Still get all customization benefits from Section 3
7. ✅ **Incremental Approach**: Can start with basic IR, add features over time

**Implementation Timeline**:
- **Weeks 1-3**: IR foundation (types, AST lowering, validation)
- **Weeks 4-5**: LSP integration with IR
- **Weeks 6-7**: Refactor codegen to use IR + templates
- **Week 8**: IR optimizations
- **Week 9**: Source map generation
- **Week 10**: Documentation and polish

**Total**: 10 weeks for complete IR + Templates system

### 10.2 Architecture Overview

```
Source (.wt)
    ↓
AST (syntax tree)
    ↓
IR (semantic representation)
    ├→ LSP (analysis, refactoring)
    ├→ Optimizer (transformations)
    ├→ Validator (static checks)
    └→ Code Generators
           ├→ StreamlitBackend (uses templates)
           ├→ ReactBackend (uses templates)
           └→ JupyterBackend (uses templates)
                ↓
        Generated Code + Source Maps
```

### 10.3 Success Metrics

**Implement Hybrid Architecture (Section 3)**:
1. ✅ Template-based generation for maintainability
2. ✅ External templates for user customization
3. ✅ Embedded fallbacks for zero-config experience
4. ✅ Backend trait for future extensibility
5. ✅ Phased implementation to manage risk

**Rationale**:
- Balances immediate needs (better maintainability) with future goals (multiple backends)
- Enables user customization without complexity of full plugin system
- Manageable development timeline (6 weeks)
- Low risk (phased approach, no breaking changes)
- High return on investment (easier maintenance + user flexibility)

### 9.2 Next Steps

1. **Review & Approve** this design document
2. **Create detailed issues** for each phase
3. **Set up branch** for refactoring work
4. **Begin Phase 1** (restructure existing code)
5. **Iterate** based on feedback from early phases

### 9.3 Success Metrics

### 10.3 Success Metrics

- ✅ All existing tests pass with new architecture
- ✅ Generated code identical to current output (when optimizations disabled)
- ✅ LSP provides type-aware hover and completions using IR
- ✅ At least 2 backends implemented (Streamlit + one other)
- ✅ Users can customize templates without recompiling
- ✅ Source maps generated for future debugger integration
- ✅ IR optimizations improve performance by >10%
- ✅ Documentation complete and clear
- ✅ Performance within 15% of current implementation (10% for IR, 5% for templates)

### 10.4 Risk Mitigation

**Risk**: IR adds complexity
- **Mitigation**: Incremental rollout, start with LSP only

**Risk**: 10-week timeline is long
- **Mitigation**: Ship template system after week 7, add IR optimizations later

**Risk**: Performance overhead
- **Mitigation**: Benchmark each phase, optimize hot paths, make optimizations optional

**Risk**: Breaking changes
- **Mitigation**: Maintain compatibility layer, use feature flags

### 10.5 Next Steps

1. ✅ **Review & Approve** this updated design document
2. ✅ **Create detailed issues** for each phase in Section 9.6
3. ✅ **Set up development branch** for IR implementation
4. ✅ **Begin Phase 1** (IR foundation - weeks 1-3)
5. ✅ **Continuous feedback** from early adopters
6. ✅ **Iterate** based on learnings from each phase

---

*Document Version: 2.0*  
*Author: GitHub Copilot*  
*Date: December 4, 2025*  
*Last Updated: Step 18 - Added IR analysis for tooling ecosystem*

### Statement Templates

| Statement Type | Template Path | Variables |
|---------------|---------------|-----------|
| Title | `statement/title.py.hbs` | `text`, `indent` |
| Subtitle | `statement/subtitle.py.hbs` | `text`, `indent` |
| Text | `statement/text.py.hbs` | `text`, `indent` |
| Button | `statement/button.py.hbs` | `label`, `body`, `indent` |
| Section | `statement/section.py.hbs` | `title`, `body`, `indent` |
| Let | `statement/let.py.hbs` | `name`, `value?`, `indent` |
| Assign | `statement/assign.py.hbs` | `name`, `value`, `indent` |
| If | `statement/if.py.hbs` | `condition`, `then_branch`, `else_branch?`, `indent` |
| Forall | `statement/forall.py.hbs` | `var`, `iterable`, `body`, `indent` |

### Expression Templates

| Expression Type | Template Path | Variables |
|----------------|---------------|-----------|
| Literal | `expression/literal.py.hbs` | `value`, `type` |
| Identifier | `expression/identifier.py.hbs` | `name` |
| Binary Op | `expression/binary_op.py.hbs` | `left`, `op`, `right` |
| Function Call | `expression/function_call.py.hbs` | `name`, `args[]` |
| Field Access | `expression/field_access.py.hbs` | `object`, `field` |
| Chain | `expression/chain.py.hbs` | `left`, `right` |

### Helper Templates

| Helper | Template Path | Purpose |
|--------|---------------|---------|
| Imports | `helpers/imports.py.hbs` | Generate import statements |
| Filter Helper | `helpers/show_filtered_helper.py.hbs` | Filtered table display function |

---

## Appendix A: Template Reference (Legacy - Pre-IR)

*Note: This appendix documents the template-only architecture described in Sections 1-8. For the recommended IR + Templates architecture, see Appendices D-G.*

---

## Appendix B: Handlebars Helpers

Custom Handlebars helpers to add:

```rust
// Register custom helpers
handlebars.register_helper("escape", Box::new(escape_string));
handlebars.register_helper("indent", Box::new(indent_lines));
handlebars.register_helper("join_args", Box::new(join_args));

fn escape_string(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0)
        .ok_or(RenderError::new("escape requires parameter"))?;
    let value = param.value().as_str()
        .ok_or(RenderError::new("escape requires string"))?;
    
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    
    out.write(&escaped)?;
    Ok(())
}
```

---

## Appendix C: Example Customizations

### Example 1: Corporate Branding

**`~/.config/wtlang/templates/streamlit/statement/title.py.hbs`**:
```handlebars
{{indent}}st.markdown("""
<div style="background: linear-gradient(90deg, #1e3a8a 0%, #3b82f6 100%); 
            padding: 20px; border-radius: 10px;">
    <h1 style="color: white; margin: 0;">{{escape text}}</h1>
</div>
""", unsafe_allow_html=True)
```

### Example 2: Logging

**`templates/streamlit/page.py.hbs`** (override):
```handlebars
import streamlit as st
import pandas as pd
from datetime import datetime
import logging

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

logger.info("Page {{page_name}} loaded at " + datetime.now().isoformat())

{{#each external_imports}}
from {{this.module}} import {{#each this.functions}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
{{/each}}

# ... rest of template
```

### Example 3: Performance Optimization

**`templates/streamlit/builtin/load_csv.py.hbs`** (cached version):
```handlebars
@st.cache_data(ttl=3600)
def _load_{{table_name}}():
    return pd.read_csv({{file_path}})

{{indent}}{{var_name}} = _load_{{table_name}}()
```

---

*Document Version: 2.0*  
*Author: GitHub Copilot*  
*Date: December 4, 2025*  
*Last Updated: Step 18 - Added comprehensive IR analysis for tooling ecosystem*

---

## Appendix D: IR Type System Specification

### Core IR Types

```rust
// crates/wtlang-core/src/ir/types.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Basic types
    Int,
    Float,
    String,
    Bool,
    
    /// Table with schema
    Table(TableSchema),
    
    /// Filter specification
    Filter {
        table_type: Box<Type>,
        mode: FilterMode,
    },
    
    /// Function type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    /// Unit type (no value)
    Unit,
    
    /// Type variable (for inference)
    Var(TypeVar),
    
    /// Error type (for error recovery)
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableSchema {
    pub fields: Vec<Field>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    Int,
    Float,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Constraint {
    Unique(String),        // Field name
    NonNull(String),       // Field name
    PrimaryKey(String),    // Field name
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterMode {
    Single,
    Multi,
}
```

### IR Node Definitions

```rust
// crates/wtlang-core/src/ir/nodes.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRModule {
    pub name: String,
    pub imports: Vec<Import>,
    pub items: Vec<IRItem>,
    pub symbols: SymbolTable,
    pub type_env: TypeEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRItem {
    TableDef {
        name: String,
        schema: TableSchema,
        source_loc: SourceRange,
    },
    
    FunctionDef {
        name: String,
        params: Vec<Param>,
        return_type: Type,
        body: Vec<IRNode>,
        is_external: bool,
        external_info: Option<ExternalInfo>,
        source_loc: SourceRange,
    },
    
    PageDef {
        name: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalInfo {
    pub language: String,
    pub module: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRNode {
    // UI Display
    ShowTable {
        table: Box<IRExpr>,
        filters: Vec<FilterSpec>,
        editable: bool,
        key: String,
        source_loc: SourceRange,
        target_loc: Option<TargetLocation>,
    },
    
    ShowText {
        text: String,
        style: TextStyle,
        source_loc: SourceRange,
    },
    
    Button {
        label: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    Section {
        title: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    // Control Flow
    Conditional {
        condition: Box<IRExpr>,
        then_branch: Vec<IRNode>,
        else_branch: Option<Vec<IRNode>>,
        source_loc: SourceRange,
    },
    
    Loop {
        variable: String,
        iterable: Box<IRExpr>,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    // Data Operations
    Pipeline {
        stages: Vec<PipelineStage>,
        result_type: Type,
        source_loc: SourceRange,
    },
    
    // Variables
    Binding {
        name: String,
        ty: Type,
        value: Option<Box<IRExpr>>,
        mutable: bool,
        source_loc: SourceRange,
    },
    
    Assignment {
        target: String,
        value: Box<IRExpr>,
        source_loc: SourceRange,
    },
    
    // Expression statement
    ExprStmt {
        expr: Box<IRExpr>,
        source_loc: SourceRange,
    },
    
    // Return
    Return {
        value: Option<Box<IRExpr>>,
        source_loc: SourceRange,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSpec {
    pub column: String,
    pub mode: FilterMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextStyle {
    Title,
    Subtitle,
    Normal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStage {
    Where {
        predicate: Box<IRExpr>,
    },
    Sort {
        columns: Vec<SortColumn>,
    },
    Aggregate {
        column: String,
        operation: AggOp,
    },
    // More stages...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortColumn {
    pub name: String,
    pub ascending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggOp {
    Sum,
    Mean,
    Count,
    Min,
    Max,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRExpr {
    Literal {
        value: Literal,
        ty: Type,
    },
    
    Variable {
        name: String,
        ty: Type,
    },
    
    BinaryOp {
        op: BinOp,
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        ty: Type,
    },
    
    UnaryOp {
        op: UnOp,
        operand: Box<IRExpr>,
        ty: Type,
    },
    
    FunctionCall {
        function: String,
        args: Vec<IRExpr>,
        ty: Type,
    },
    
    FieldAccess {
        object: Box<IRExpr>,
        field: String,
        ty: Type,
    },
    
    TableConstructor {
        fields: Vec<(String, IRExpr)>,
        ty: Type,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRange {
    pub file: PathBuf,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}
```

### Symbol Table

```rust
// crates/wtlang-core/src/ir/symbols.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub kind: SymbolKind,
    pub declaration_loc: SourceRange,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Variable,
    Function,
    Table,
    Page,
    Parameter,
}

impl SymbolTable {
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Search current scope and parents
    }
    
    pub fn lookup_at_position(&self, pos: Position) -> Option<&Symbol> {
        // Find symbol at given source position
    }
    
    pub fn visible_at(&self, pos: Position) -> Vec<&Symbol> {
        // All symbols visible at position (for autocomplete)
    }
}
```

### Type Environment

```rust
// crates/wtlang-core/src/ir/type_env.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEnvironment {
    bindings: HashMap<String, Type>,
    constraints: Vec<TypeConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConstraint {
    pub var: TypeVar,
    pub must_be: Type,
}

impl TypeEnvironment {
    pub fn infer_type(&mut self, expr: &IRExpr) -> Result<Type, TypeError> {
        // Type inference logic
    }
    
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), TypeError> {
        // Unification for type inference
    }
}
```

---

## Appendix E: IR Optimization Passes

### Pass 1: Constant Folding

```rust
// crates/wtlang-compiler/src/ir/optimize/constant_folding.rs

pub fn fold_constants(module: IRModule) -> IRModule {
    let mut optimizer = ConstantFolder::new();
    optimizer.visit_module(module)
}

struct ConstantFolder;

impl ConstantFolder {
    fn fold_expr(&self, expr: IRExpr) -> IRExpr {
        match expr {
            IRExpr::BinaryOp { op: BinOp::Add, left, right, ty } => {
                match (*left, *right) {
                    (IRExpr::Literal { value: Literal::Int(a), .. },
                     IRExpr::Literal { value: Literal::Int(b), .. }) => {
                        IRExpr::Literal {
                            value: Literal::Int(a + b),
                            ty,
                        }
                    }
                    (left, right) => IRExpr::BinaryOp {
                        op: BinOp::Add,
                        left: Box::new(self.fold_expr(left)),
                        right: Box::new(self.fold_expr(right)),
                        ty,
                    }
                }
            }
            // More operators...
            _ => expr,
        }
    }
}
```

### Pass 2: Pipeline Fusion

```rust
// crates/wtlang-compiler/src/ir/optimize/pipeline_fusion.rs

pub fn fuse_pipelines(module: IRModule) -> IRModule {
    let mut optimizer = PipelineFuser::new();
    optimizer.visit_module(module)
}

struct PipelineFuser;

impl PipelineFuser {
    fn fuse_node(&self, node: IRNode) -> IRNode {
        match node {
            IRNode::Pipeline { stages, result_type, source_loc } => {
                let fused_stages = self.fuse_where_stages(stages);
                let fused_stages = self.fuse_sort_stages(fused_stages);
                IRNode::Pipeline {
                    stages: fused_stages,
                    result_type,
                    source_loc,
                }
            }
            _ => node,
        }
    }
    
    fn fuse_where_stages(&self, stages: Vec<PipelineStage>) -> Vec<PipelineStage> {
        let mut result = Vec::new();
        let mut where_predicates = Vec::new();
        
        for stage in stages {
            match stage {
                PipelineStage::Where { predicate } => {
                    where_predicates.push(predicate);
                }
                other => {
                    if !where_predicates.is_empty() {
                        // Combine all where predicates with AND
                        let combined = where_predicates.into_iter()
                            .reduce(|acc, pred| {
                                Box::new(IRExpr::BinaryOp {
                                    op: BinOp::And,
                                    left: acc,
                                    right: pred,
                                    ty: Type::Bool,
                                })
                            })
                            .unwrap();
                        
                        result.push(PipelineStage::Where { predicate: combined });
                        where_predicates = Vec::new();
                    }
                    result.push(other);
                }
            }
        }
        
        // Handle trailing where predicates
        if !where_predicates.is_empty() {
            let combined = where_predicates.into_iter()
                .reduce(|acc, pred| {
                    Box::new(IRExpr::BinaryOp {
                        op: BinOp::And,
                        left: acc,
                        right: pred,
                        ty: Type::Bool,
                    })
                })
                .unwrap();
            result.push(PipelineStage::Where { predicate: combined });
        }
        
        result
    }
}
```

### Pass 3: Dead Code Elimination

```rust
// crates/wtlang-compiler/src/ir/optimize/dead_code.rs

pub fn eliminate_dead_code(module: IRModule) -> IRModule {
    let mut eliminator = DeadCodeEliminator::new();
    eliminator.visit_module(module)
}

struct DeadCodeEliminator {
    used_symbols: HashSet<String>,
}

impl DeadCodeEliminator {
    fn find_used_symbols(&mut self, nodes: &[IRNode]) {
        for node in nodes {
            self.visit_node(node);
        }
    }
    
    fn visit_node(&mut self, node: &IRNode) {
        match node {
            IRNode::ExprStmt { expr, .. } => {
                self.mark_expr_symbols(expr);
            }
            IRNode::ShowTable { table, .. } => {
                self.mark_expr_symbols(table);
            }
            IRNode::Binding { name, value, .. } => {
                if let Some(v) = value {
                    self.mark_expr_symbols(v);
                }
                // Don't mark `name` as used unless referenced elsewhere
            }
            // ...
            _ => {}
        }
    }
    
    fn mark_expr_symbols(&mut self, expr: &IRExpr) {
        match expr {
            IRExpr::Variable { name, .. } => {
                self.used_symbols.insert(name.clone());
            }
            IRExpr::BinaryOp { left, right, .. } => {
                self.mark_expr_symbols(left);
                self.mark_expr_symbols(right);
            }
            // ...
            _ => {}
        }
    }
    
    fn remove_unused_bindings(&self, nodes: Vec<IRNode>) -> Vec<IRNode> {
        nodes.into_iter()
            .filter(|node| {
                if let IRNode::Binding { name, .. } = node {
                    self.used_symbols.contains(name)
                } else {
                    true
                }
            })
            .collect()
    }
}
```

### Pass 4: Inline Simple Functions

```rust
// crates/wtlang-compiler/src/ir/optimize/inline.rs

pub fn inline_simple_functions(module: IRModule) -> IRModule {
    let mut inliner = FunctionInliner::new();
    inliner.visit_module(module)
}

struct FunctionInliner {
    inline_candidates: HashMap<String, Vec<IRNode>>,
}

impl FunctionInliner {
    fn should_inline(&self, func: &IRItem) -> bool {
        if let IRItem::FunctionDef { body, is_external, .. } = func {
            !is_external && body.len() <= 3  // Only inline small functions
        } else {
            false
        }
    }
    
    fn inline_call(&self, call: IRExpr) -> IRExpr {
        // Replace function call with inlined body
        // Substitute parameters with arguments
        // ...
    }
}
```

---

## Appendix F: Source Map Format

### Source Map Structure

```rust
// crates/wtlang-compiler/src/sourcemap.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub version: u32,  // Always 3 (compatible with JS source maps)
    pub file: String,  // Generated file name
    pub sources: Vec<String>,  // Source file paths
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub generated_line: usize,
    pub generated_column: usize,
    pub source_index: usize,
    pub source_line: usize,
    pub source_column: usize,
    pub name_index: Option<usize>,
}

impl SourceMap {
    pub fn to_json(&self) -> String {
        // Serialize to JSON format
        serde_json::to_string_pretty(self).unwrap()
    }
    
    pub fn to_vlq(&self) -> String {
        // Convert to VLQ (Variable Length Quantity) encoding
        // Compatible with browser devtools
    }
}
```

### Example Source Map

```json
{
  "version": 3,
  "file": "Home.py",
  "sources": ["Home.wt"],
  "mappings": [
    {
      "generated_line": 10,
      "generated_column": 0,
      "source_index": 0,
      "source_line": 3,
      "source_column": 2
    },
    {
      "generated_line": 15,
      "generated_column": 4,
      "source_index": 0,
      "source_line": 5,
      "source_column": 4
    }
  ]
}
```

---

## Appendix G: Backend Interface Specification

### Backend Trait

```rust
// crates/wtlang-compiler/src/codegen/backend.rs

pub trait Backend {
    /// Backend identifier (e.g., "streamlit", "react", "jupyter")
    fn name(&self) -> &str;
    
    /// File extension for generated files
    fn file_extension(&self) -> &str;
    
    /// Generate complete program from IR
    fn generate_from_ir(&mut self, ir: &IRModule) 
        -> Result<BackendOutput, String>;
    
    /// Render single IR node to code
    fn render_node(&self, node: &IRNode) -> Result<String, String>;
    
    /// Render IR expression to code
    fn render_expr(&self, expr: &IRExpr) -> Result<String, String>;
    
    /// Additional files to generate (e.g., requirements.txt, package.json)
    fn auxiliary_files(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    
    /// Backend-specific validation
    fn validate_ir(&self, ir: &IRModule) -> Vec<Diagnostic> {
        Vec::new()
    }
    
    /// Required imports for given IR module
    fn compute_imports(&self, ir: &IRModule) -> Vec<Import>;
    
    /// Helper functions needed
    fn required_helpers(&self, nodes: &[IRNode]) -> Vec<String>;
}

#[derive(Debug)]
pub struct BackendOutput {
    pub files: HashMap<String, String>,
    pub source_map: Option<SourceMap>,
    pub warnings: Vec<Diagnostic>,
}
```

---

*End of Document*

