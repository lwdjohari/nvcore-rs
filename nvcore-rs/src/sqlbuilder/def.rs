#![allow(dead_code)]

// use chrono::{TimeZone, Utc};
use std::fmt;
use std::time::SystemTime;

// Define DefaultPostgresParamType to support comprehensive PostgreSQL data types
#[derive(Debug, PartialEq, Clone)]
pub enum DefaultPostgresParamType {
    SmallInt(i16),         // Small Int
    Int(i32),              // Integer
    BigInt(i64),         // Bigint
    Float(f32),            // Float
    Double(f64),           // Double
    String(String),        // Char, Varchar, NVarchar
    Bool(bool),            // Bool
    TimePoint(SystemTime), // Timestamp
}

impl fmt::Display for DefaultPostgresParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefaultPostgresParamType::SmallInt(value) => write!(f, "{}", value),
            DefaultPostgresParamType::Int(value) => write!(f, "{}", value),
            DefaultPostgresParamType::BigInt(value) => write!(f, "{}", value),
            DefaultPostgresParamType::Float(value) => write!(f, "{}", value),
            DefaultPostgresParamType::Double(value) => write!(f, "{}", value),
            DefaultPostgresParamType::String(value) => write!(f, "{}", value),
            DefaultPostgresParamType::Bool(value) => write!(f, "{}", value),
            DefaultPostgresParamType::TimePoint(value) => {
                let duration = value.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                write!(f, "{}", duration.as_secs())
            },
        }
    }
}


// Define DefaultOracleParamType to support comprehensive Oracle data types
#[derive(Debug, PartialEq, Clone)]
pub enum DefaultOracleParamType {
    Int(i32),       // Oracle NUMBER, INTEGER
    LongLong(i64),  // Oracle NUMBER (large integer)
    Float(f32),     // Oracle FLOAT
    Double(f64),    // Oracle DOUBLE PRECISION
    String(String), // Oracle VARCHAR2, CHAR, CLOB, NCHAR, NVARCHAR2, NCLOB,
    // BFILE, XMLType, ROWID, UROWID
    Bool(bool),            // Oracle BOOLEAN
    TimePoint(SystemTime), // Oracle DATE, TIMESTAMP, TIMESTAMP WITH TIME ZONE,
    // TIMESTAMP WITH LOCAL TIME ZONE
    RawBlob(Vec<u8>), // Oracle RAW, BLOB
}


// Define a trait for parameter types
pub trait ParameterType {}

impl ParameterType for DefaultPostgresParamType {}
impl ParameterType for DefaultOracleParamType {}

#[derive(Debug, PartialEq,Eq,Clone,Copy)]
pub enum DatabaseDialect {
    PostgreSQL,
    Oracle,
}

impl fmt::Display for DatabaseDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseDialect::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseDialect::Oracle => write!(f, "Oracle"),
        }
    }
}

#[derive(Debug)]
pub enum SortType {
    Ascending,
    Descending,
}

impl fmt::Display for SortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortType::Ascending => write!(f, "Ascending"),
            SortType::Descending => write!(f, "Descending"),
        }
    }
}

#[derive(Debug,PartialEq, Clone, Copy)]
pub enum SqlOperator {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Like,
    Between,
    In,
}

impl fmt::Display for SqlOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlOperator::Equal => write!(f, "="),
            SqlOperator::NotEqual => write!(f, "!="),
            SqlOperator::Less => write!(f, "<"),
            SqlOperator::LessOrEqual => write!(f, "<="),
            SqlOperator::Greater => write!(f, ">"),
            SqlOperator::GreaterOrEqual => write!(f, ">="),
            SqlOperator::Like => write!(f, "LIKE"),
            SqlOperator::Between => write!(f, "BETWEEN"),
            SqlOperator::In => write!(f, "IN"),
        }
    }
}

#[derive(Debug)]
pub enum LogicOperator {
    And,
    Or,
}

impl fmt::Display for LogicOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicOperator::And => write!(f, "AND"),
            LogicOperator::Or => write!(f, "OR"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SqlAggregateFunction {
    None,
    Distinct,
    Count,
    Avg,
    Sum,
    ToUpper,
    ToLower,
    ToIso8601DateTime,
    ToIso8601Date,
    ToIso8601Time,
}

impl fmt::Display for SqlAggregateFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlAggregateFunction::None => write!(f, "None"),
            SqlAggregateFunction::Distinct => write!(f, "Distinct"),
            SqlAggregateFunction::Count => write!(f, "Count"),
            SqlAggregateFunction::Avg => write!(f, "Avg"),
            SqlAggregateFunction::Sum => write!(f, "Sum"),
            SqlAggregateFunction::ToUpper => write!(f, "ToUpper"),
            SqlAggregateFunction::ToLower => write!(f, "ToLower"),
            SqlAggregateFunction::ToIso8601DateTime => write!(f, "ToIso8601DateTime"),
            SqlAggregateFunction::ToIso8601Date => write!(f, "ToIso8601Date"),
            SqlAggregateFunction::ToIso8601Time => write!(f, "ToIso8601Time"),
        }
    }
}

#[derive(Debug)]
pub enum SqlJoinType {
    None,
    InnerJoin,
    LeftJoin,
    RightJoin,
}

impl fmt::Display for SqlJoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlJoinType::None => write!(f, "None"),
            SqlJoinType::InnerJoin => write!(f, "InnerJoin"),
            SqlJoinType::LeftJoin => write!(f, "LeftJoin"),
            SqlJoinType::RightJoin => write!(f, "RightJoin"),
        }
    }
}

#[derive(Debug)]
pub enum JoinDefMode {
    RecordKeyBoth,
    SubquerySelectString,
    SubqueryRawString,
    SubquerySelectObject,
}

impl fmt::Display for JoinDefMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinDefMode::RecordKeyBoth => write!(f, "RecordKeyBoth"),
            JoinDefMode::SubquerySelectString => write!(f, "SubquerySelectString"),
            JoinDefMode::SubqueryRawString => write!(f, "SubqueryRawString"),
            JoinDefMode::SubquerySelectObject => write!(f, "SubquerySelectObject"),
        }
    }
}

// Function to determine the parameter format based on the dialect
pub fn determine_parameter_format(dialect: &DatabaseDialect, parameter_index: u32) -> String {
    match dialect {
        DatabaseDialect::PostgreSQL => format!("${}", parameter_index),
        DatabaseDialect::Oracle => format!(":{}", parameter_index),
    }
}

// Function to convert SqlOperator to a string representation
pub fn sql_operator_to_string(op: SqlOperator) -> String {
    match op {
        SqlOperator::Equal => "=".to_string(),
        SqlOperator::NotEqual => "!=".to_string(),
        SqlOperator::Less => "<".to_string(),
        SqlOperator::LessOrEqual => "<=".to_string(),
        SqlOperator::Greater => ">".to_string(),
        SqlOperator::GreaterOrEqual => ">=".to_string(),
        SqlOperator::Like => "LIKE".to_string(),
        SqlOperator::Between => "BETWEEN".to_string(),
        SqlOperator::In => "IN".to_string(),
    }
}

// Function to convert LogicOperator to a string representation
pub fn logic_operator_to_string(logic: LogicOperator) -> String {
    match logic {
        LogicOperator::And => "AND".to_string(),
        LogicOperator::Or => "OR".to_string(),
    }
}

// Function to generate indentation for pretty printing
pub fn generate_indentation(level: u32, indent_char: char, number_per_print: u32) -> String {
    let n = (number_per_print * level) as usize;
    std::iter::repeat(indent_char).take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_dialect_display() {
        assert_eq!(DatabaseDialect::PostgreSQL.to_string(), "PostgreSQL");
        assert_eq!(DatabaseDialect::Oracle.to_string(), "Oracle");
    }
}
