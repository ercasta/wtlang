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

## 9. Conclusion

### 9.1 Recommendation Summary

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

- ✅ All existing tests pass with new architecture
- ✅ Generated code identical to current output (default config)
- ✅ Users can customize templates without recompiling
- ✅ At least 2 backends implemented (Streamlit + one other)
- ✅ Documentation complete and clear
- ✅ Performance within 10% of current implementation

---

## Appendix A: Template Reference

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

*Document Version: 1.0*  
*Author: GitHub Copilot*  
*Date: December 4, 2025*
