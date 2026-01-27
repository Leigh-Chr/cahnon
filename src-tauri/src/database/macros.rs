//! Shared macros for database update operations.

/// Appends a SET clause and parameter for an optional field in a dynamic UPDATE query.
///
/// # Variants
/// - `add_field!(set_clauses, params, field, "column")` — String/clone value
/// - `add_field!(set_clauses, params, field, "column", int)` — Integer/Copy value
/// - `add_field!(set_clauses, params, field, "column", bool)` — Boolean → i32 value
/// - `add_field!(set_clauses, params, field, "column", float)` — Float/Copy value
macro_rules! add_field {
    ($set_clauses:expr, $params:expr, $field:expr, $column:literal) => {
        if let Some(val) = &$field {
            $set_clauses.push(format!("{} = ?{}", $column, $params.len() + 1));
            $params.push(Box::new(val.clone()));
        }
    };
    ($set_clauses:expr, $params:expr, $field:expr, $column:literal, int) => {
        if let Some(val) = $field {
            $set_clauses.push(format!("{} = ?{}", $column, $params.len() + 1));
            $params.push(Box::new(val));
        }
    };
    ($set_clauses:expr, $params:expr, $field:expr, $column:literal, bool) => {
        if let Some(val) = $field {
            $set_clauses.push(format!("{} = ?{}", $column, $params.len() + 1));
            $params.push(Box::new(val as i32));
        }
    };
    ($set_clauses:expr, $params:expr, $field:expr, $column:literal, float) => {
        if let Some(val) = $field {
            $set_clauses.push(format!("{} = ?{}", $column, $params.len() + 1));
            $params.push(Box::new(val));
        }
    };
}

pub(crate) use add_field;
