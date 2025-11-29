# WTLang Target Platform Design Considerations

This document evaluates target platform alternatives for compiling WTLang and provides rationale for the chosen approach.

## 1. Primary Target Platform

### Alternatives Considered

**A. Plain HTML/CSS/JavaScript**
- Pros: Maximum control, no dependencies, works everywhere
- Cons: Requires building UI framework from scratch, complex state management, extensive development time

**B. React/Vue/Angular SPA**
- Pros: Rich ecosystem, component reusability, professional appearance
- Cons: Complex build pipeline, requires significant frontend expertise, state management complexity

**C. Streamlit (Chosen)**
- Pros: Python-based, designed for data apps, automatic UI generation, simple deployment
- Cons: Less customization, Python requirement, potential performance limits

**D. Dash (Plotly)**
- Pros: Similar to Streamlit, good for dashboards, Python-based
- Cons: More verbose than Streamlit, steeper learning curve

**E. Gradio**
- Pros: Very simple, great for ML demos
- Cons: Limited table manipulation features, less suitable for complex workflows

### Rationale
Streamlit was chosen as the primary target because it perfectly aligns with WTLang's goals:
1. **Rapid Development**: Streamlit's declarative API maps naturally to WTLang's structure
2. **Table-Centric**: Built-in `st.dataframe()` and `st.data_editor()` for interactive tables
3. **Python Ecosystem**: Seamless integration with pandas, which is ideal for table operations
4. **Deployment**: Simple deployment options (Streamlit Cloud, Docker)
5. **Interactivity**: Built-in widgets for filtering, sorting, and editing without custom JavaScript

The compilation strategy:
```
WTLang Table → pandas DataFrame → st.data_editor()
WTLang Functions → Python functions using pandas operations
WTLang Pages → Streamlit pages with st.navigation()
```

## 2. Secondary/Future Target Platforms

### Alternatives Considered

**A. Desktop Applications (Qt/Electron)**
- Pros: Offline capability, native performance, OS integration
- Cons: Distribution complexity, platform-specific builds

**B. Excel Add-in**
- Pros: Users already familiar with Excel, natural fit for tables
- Cons: Limited to Windows, VBA/JavaScript API constraints, deployment challenges

**C. Web Framework (Django/Flask) with HTMX**
- Pros: Traditional web architecture, SEO-friendly, more control
- Cons: More boilerplate, complex state management

**D. Jupyter Notebooks**
- Pros: Interactive, code exploration, data science integration
- Cons: Not suitable for end-user applications, requires notebook environment

**E. Progressive Web App (PWA)**
- Pros: Offline support, installable, cross-platform
- Cons: Requires full frontend implementation

### Rationale for Future Expansion
The architecture should support multiple backends by:
1. Creating an intermediate representation (IR) after parsing
2. Having platform-specific code generators
3. Keeping platform-agnostic core logic

Priority order for future platforms:
1. **PWA/React** for broader deployment options
2. **Desktop (Electron)** for offline/enterprise scenarios
3. **Jupyter integration** for data scientists

## 3. Backend/Runtime Environment

### Alternatives Considered

**A. Server-side Only**
- Pros: Security, centralized logic, easier deployment
- Cons: Network dependency, latency, server costs

**B. Client-side Only (WebAssembly)**
- Pros: No server needed, instant interaction, offline capable
- Cons: Limited backend integration, data persistence challenges

**C. Hybrid: Server for Logic, Client for UI (Chosen)**
- Pros: Best of both worlds, scalable, secure
- Cons: More complex architecture

### Rationale
With Streamlit as the target, the hybrid approach is natural:
- **Server-side**: WTLang logic compiles to Python running on Streamlit server
- **Client-side**: Streamlit's frontend handles rendering and basic interactions
- **Communication**: Streamlit's rerun mechanism manages state synchronization

This provides security (business logic stays server-side) while maintaining responsiveness through Streamlit's optimizations.

## 4. Data Persistence Strategy

### Alternatives Considered

**A. In-Memory Only**
- Pros: Simple, fast
- Cons: Data lost on refresh, not suitable for real applications

**B. Database Integration**
- Pros: Persistent, scalable, multi-user support
- Cons: Requires database setup, connection management

**C. File-based (CSV/Excel/Parquet)**
- Pros: Simple, portable, human-readable (CSV/Excel)
- Cons: Concurrent access issues, limited querying

**D. Streamlit Session State + Optional Persistence (Chosen)**
```python
# Generated code pattern
if 'tables' not in st.session_state:
    st.session_state.tables = load_data()

# User operations modify session state
st.session_state.tables['users'] = filtered_df

# Export functionality
if st.button("Save"):
    save_data(st.session_state.tables)
```
- Pros: Simple, flexible, works with Streamlit model
- Cons: Per-session isolation (can be pro or con)

### Rationale
Leveraging Streamlit's session state provides a simple persistence model for single-user sessions. For multi-user scenarios, WTLang can generate code that integrates with databases through Python libraries (SQLAlchemy, etc.). The export functionality handles long-term persistence to Excel/CSV.

## 5. External Function Integration

### Alternatives Considered

**A. REST API Calls**
- Pros: Language-agnostic, distributed systems friendly
- Cons: Network overhead, async complexity, error handling

**B. Python Import System (Chosen)**
```python
# WTLang: external function analyze from "analytics.ml"
# Generated Python:
from analytics.ml import analyze
```
- Pros: Direct integration, type checking, debugging support
- Cons: Requires Python environment, same-process only

**C. Message Queue (Celery/RabbitMQ)**
- Pros: Asynchronous, scalable, fault-tolerant
- Cons: Infrastructure overhead, complexity

### Rationale
Since Streamlit runs Python, direct imports are the most natural integration method. External functions are Python functions that accept and return pandas DataFrames. This keeps the mental model simple and provides excellent performance for data processing tasks.

## 6. Export Functionality

### Alternatives Considered

**A. Excel Export Only**
- Pros: Business-friendly, preserves formatting
- Cons: Requires openpyxl/xlsxwriter, file size

**B. CSV Only**
- Pros: Simple, universal
- Cons: Loses formatting, type information

**C. Multiple Formats (Chosen)**
```python
# Generated export code
if export_format == "excel":
    df.to_excel("output.xlsx")
elif export_format == "csv":
    df.to_csv("output.csv")
elif export_format == "parquet":
    df.to_parquet("output.parquet")
```
- Pros: Flexible, user choice
- Cons: Multiple dependencies

**D. Database Export**
- Pros: Integration with existing systems
- Cons: Requires configuration

### Rationale
Supporting multiple export formats (Excel, CSV, Parquet) gives users flexibility. Excel is the default for business users, CSV for interoperability, and Parquet for data engineering pipelines. All are well-supported by pandas.

## 7. Deployment Strategy

### Alternatives Considered

**A. Streamlit Cloud (Chosen for Quick Deployment)**
- Pros: Zero-config deployment, free tier, GitHub integration
- Cons: Limited resources, public by default

**B. Docker Containers**
- Pros: Reproducible, portable, works anywhere
- Cons: Requires Docker knowledge, infrastructure management

**C. Traditional Web Hosting (Heroku, AWS, etc.)**
- Pros: Scalable, customizable
- Cons: More configuration, higher cost

**D. On-Premises Installation**
- Pros: Data security, no cloud dependency
- Cons: User must manage infrastructure

### Rationale
WTLang should support multiple deployment strategies:
1. **Development**: Local Streamlit server (`streamlit run`)
2. **Quick Deploy**: Streamlit Cloud for prototypes and demos
3. **Production**: Docker containers for scalability and security
4. **Enterprise**: On-premises with Docker/Kubernetes

The compiler can generate appropriate configuration files (Dockerfile, requirements.txt, streamlit config) based on deployment target.

## 8. Multi-Page Applications

### Alternatives Considered

**A. Single Page with Tabs**
- Pros: Simple state management
- Cons: All code loads at once, less navigable

**B. Streamlit Multi-Page Apps (Chosen)**
```
app/
  ├── Home.py
  ├── pages/
  │   ├── 1_Users.py
  │   ├── 2_Orders.py
  │   └── 3_Reports.py
```
- Pros: Clean separation, lazy loading, standard pattern
- Cons: State management across pages requires careful handling

**C. URL Routing**
- Pros: Bookmarkable, RESTful
- Cons: Not native to Streamlit, requires custom implementation

### Rationale
Streamlit's native multi-page app structure maps perfectly to WTLang's page concept. Each WTLang page compiles to a separate Python file, maintaining clean separation of concerns and improving performance through lazy loading.

## 9. Real-time Collaboration

### Alternatives Considered

**A. No Collaboration (Chosen for v1)**
- Pros: Simple, no infrastructure
- Cons: Single-user only

**B. WebSocket-based Sync**
- Pros: Real-time updates
- Cons: Complex state reconciliation, requires infrastructure

**C. Operational Transformation (OT)**
- Pros: Google Docs-style collaboration
- Cons: Very complex, overkill for table editing

### Rationale
Version 1 focuses on single-user applications, which covers the majority of business use cases. Future versions could add collaboration through:
- Database-backed state with change tracking
- WebSocket layer for real-time updates
- Conflict resolution strategies

This keeps the initial implementation simple while leaving the door open for collaboration features.

## 10. Performance Optimization

### Alternatives Considered

**A. No Optimization**
- Pros: Simple implementation
- Cons: Poor user experience with large datasets

**B. Streamlit Caching (Chosen)**
```python
@st.cache_data
def load_large_table():
    return pd.read_csv("large_file.csv")

@st.cache_data
def expensive_computation(df):
    return df.groupby('category').agg({'sales': 'sum'})
```
- Pros: Easy to implement, significant performance gains
- Cons: Cache invalidation complexity

**C. Lazy Loading/Pagination**
- Pros: Handles unlimited data
- Cons: Complex UI, requires careful implementation

**D. Server-side Processing with Streaming**
- Pros: Scalable to very large datasets
- Cons: Infrastructure requirements

### Rationale
WTLang should generate code that automatically uses Streamlit's `@st.cache_data` decorator for:
- Data loading functions
- Expensive computations (aggregations, joins)
- Function chain results

For very large datasets, the compiler can generate paginated views using `st.data_editor`'s built-in virtual scrolling.

## Summary

The target platform strategy for WTLang:

**Primary Target: Streamlit**
- Natural fit for table-centric applications
- Simple deployment and development workflow
- Rich Python ecosystem for data manipulation
- Built-in interactive table widgets

**Architecture Principles:**
1. Compile to clean, readable Python code
2. Leverage pandas for table operations
3. Use Streamlit's native features (session state, caching, pages)
4. Generate deployment-ready configurations

**Future Extensibility:**
- Modular code generation to support multiple backends
- Platform-agnostic intermediate representation
- Pluggable export and integration mechanisms

This approach delivers immediate value while maintaining flexibility for future platform support as WTLang evolves.
