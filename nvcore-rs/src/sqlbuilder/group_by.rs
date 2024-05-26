#![allow(dead_code)]

use crate::sqlbuilder::NvSelect;
// use crate::sqlbuilder::def::{DefaultPostgresParamType, DefaultOracleParamType};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GroupByMode {
    Field,
    FunctionCall,
    RawString,
}

impl std::fmt::Display for GroupByMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupByMode::Field => write!(f, "Field"),
            GroupByMode::FunctionCall => write!(f, "FunctionCall"),
            GroupByMode::RawString => write!(f, "RawString"),
        }
    }
}

pub struct GroupByClause {
    field_name: String,
    table_alias: Option<String>,
    start_parameter_index: u32,
    parameter_index: u32,
    level: u32,
    mode: GroupByMode,
}

impl GroupByClause {
    fn process_next_parameter_index(_mode: GroupByMode, start_parameter_index: u32) -> u32 {
        start_parameter_index
    }

    pub fn new(
        field_name: String,
        alias: Option<String>,
        mode: GroupByMode,
        parameter_index: u32,
        level: u32,
    ) -> Self {
        let parameter_index = Self::process_next_parameter_index(mode, parameter_index);
        Self {
            field_name,
            table_alias: alias,
            start_parameter_index: parameter_index,
            parameter_index,
            level,
            mode,
        }
    }

    pub fn next_parameter_index(&self) -> u32 {
        self.parameter_index
    }

    pub fn field_name(&self) -> &String {
        &self.field_name
    }

    pub fn table_alias(&self) -> &Option<String> {
        &self.table_alias
    }

    pub fn build_fieldname(&self) -> String {
        match &self.table_alias {
            Some(alias) => format!("{}.{}", alias, self.field_name),
            None => self.field_name.clone(),
        }
    }

    pub fn generate_query(&self) -> String {
        self.build_fieldname()
    }
}

pub struct GroupByStatement<T> {
    parent: Option<*mut NvSelect<T>>,
    sorts: Vec<GroupByClause>,
    level: u32,
    param_index: u32,
}

impl<T> GroupByStatement<T> {
    pub fn new() -> Self {
        Self {
            parent: None,
            sorts: Vec::new(),
            level: 0,
            param_index: 1,
        }
    }

    pub fn new_with_parent(parent: *mut NvSelect<T>, parameter_index: u32, level: u32) -> Self {
        Self {
            parent: Some(parent),
            sorts: Vec::new(),
            level,
            param_index: parameter_index,
        }
    }

    pub fn field(mut self, field_name: String, table_alias: Option<String>) -> Self {
        let clause = GroupByClause::new(
            field_name,
            table_alias,
            GroupByMode::Field,
            self.param_index,
            self.level,
        );
        self.param_index = clause.next_parameter_index();
        self.sorts.push(clause);
        self
    }

    pub fn generate_query(&self, _pretty_print: bool) -> String {
        let mut query = String::new();
        let mut is_first_element = true;

        for s in &self.sorts {
            if !is_first_element {
                query.push_str(", ");
            }
            query.push_str(&s.generate_query());
            is_first_element = false;
        }

        query
    }

    pub fn current_parameter_index(&self) -> u32 {
        self.param_index
    }

    pub fn end_group_by_block(&self) -> &NvSelect<T> {
        match self.parent {
            Some(parent) => unsafe { &*parent },
            None => panic!("null-reference to parent of NvSelect<T>"),
        }
    }
}
