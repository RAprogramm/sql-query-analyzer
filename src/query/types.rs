use std::sync::OnceLock;

use compact_str::CompactString;
use serde::Serialize;
use smallvec::SmallVec;

/// Type alias for small column vectors (typically < 8 elements)
pub type ColumnVec = SmallVec<[CompactString; 8]>;

/// Parsed SQL query with metadata
#[derive(Debug, Clone, Serialize)]
pub struct Query {
    pub raw:          String,
    pub query_type:   QueryType,
    pub tables:       Vec<CompactString>,
    pub cte_names:    Vec<CompactString>,
    pub where_cols:   ColumnVec,
    pub join_cols:    ColumnVec,
    pub order_cols:   ColumnVec,
    pub group_cols:   ColumnVec,
    pub having_cols:  ColumnVec,
    pub window_funcs: Vec<WindowFunction>,
    pub limit:        Option<u64>,
    pub offset:       Option<u64>,
    pub has_union:    bool,
    pub has_distinct: bool,
    pub has_subquery: bool,
    #[serde(skip)]
    complexity_cell:  OnceLock<QueryComplexity>
}

impl Query {
    /// Get complexity (lazily calculated)
    pub fn complexity(&self) -> &QueryComplexity {
        self.complexity_cell
            .get_or_init(|| calculate_complexity(self))
    }
}

/// Window function information
#[derive(Debug, Clone, Serialize)]
pub struct WindowFunction {
    pub name:           CompactString,
    pub partition_cols: Vec<CompactString>,
    pub order_cols:     Vec<CompactString>
}

/// Query complexity metrics
#[derive(Debug, Clone, Serialize, Default)]
pub struct QueryComplexity {
    pub score:             u32,
    pub table_count:       u32,
    pub join_count:        u32,
    pub subquery_count:    u32,
    pub condition_count:   u32,
    pub aggregation_count: u32,
    pub window_count:      u32
}

/// Type of SQL query
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Truncate,
    Other
}

impl Default for Query {
    fn default() -> Self {
        Self {
            raw:             String::new(),
            query_type:      QueryType::Other,
            tables:          Vec::new(),
            cte_names:       Vec::new(),
            where_cols:      ColumnVec::new(),
            join_cols:       ColumnVec::new(),
            order_cols:      ColumnVec::new(),
            group_cols:      ColumnVec::new(),
            having_cols:     ColumnVec::new(),
            window_funcs:    Vec::new(),
            limit:           None,
            offset:          None,
            has_union:       false,
            has_distinct:    false,
            has_subquery:    false,
            complexity_cell: OnceLock::new()
        }
    }
}

impl Query {
    pub fn new(raw: String, query_type: QueryType) -> Self {
        Self {
            raw,
            query_type,
            ..Default::default()
        }
    }
}

impl std::fmt::Display for QueryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Select => write!(f, "SELECT"),
            Self::Insert => write!(f, "INSERT"),
            Self::Update => write!(f, "UPDATE"),
            Self::Delete => write!(f, "DELETE"),
            Self::Truncate => write!(f, "TRUNCATE"),
            Self::Other => write!(f, "OTHER")
        }
    }
}

/// Calculate complexity score for a query
pub fn calculate_complexity(query: &Query) -> QueryComplexity {
    let table_count = query.tables.len() as u32;
    let join_count = query.join_cols.len() as u32;
    let condition_count = query.where_cols.len() as u32 + query.having_cols.len() as u32;
    let window_count = query.window_funcs.len() as u32;
    let subquery_count = if query.has_subquery { 1 } else { 0 };
    let aggregation_count = query.group_cols.len() as u32;
    let score = table_count
        + join_count * 3
        + condition_count
        + window_count * 5
        + subquery_count * 4
        + aggregation_count * 2
        + if query.has_union { 3 } else { 0 }
        + if query.has_distinct { 1 } else { 0 };
    QueryComplexity {
        score,
        table_count,
        join_count,
        subquery_count,
        condition_count,
        aggregation_count,
        window_count
    }
}
