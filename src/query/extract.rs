mod expr;
mod set_expr;
mod table;

use compact_str::CompactString;
pub use expr::extract_columns_from_expr;
use indexmap::IndexSet;
pub use set_expr::extract_from_set_expr;

use super::types::WindowFunction;

/// Context for extracting query metadata
pub struct ExtractionContext<'a> {
    pub tables:       &'a mut IndexSet<CompactString>,
    pub where_cols:   &'a mut IndexSet<CompactString>,
    pub join_cols:    &'a mut IndexSet<CompactString>,
    pub group_cols:   &'a mut IndexSet<CompactString>,
    pub having_cols:  &'a mut IndexSet<CompactString>,
    pub window_funcs: &'a mut Vec<WindowFunction>,
    pub has_union:    &'a mut bool,
    pub has_distinct: &'a mut bool,
    pub has_subquery: &'a mut bool
}
