#![allow(dead_code)]

use crate::sqlbuilder::NvSelect;
use crate::sqlbuilder::{DatabaseDialect, LogicOperator, SqlOperator};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConditionMode {
    Comparator,
    LogicalOperator,
    StartGroup,
    EndGroup,
    Subquery,
}

impl std::fmt::Display for ConditionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionMode::Comparator => write!(f, "Comparator"),
            ConditionMode::LogicalOperator => write!(f, "LogicalOperator"),
            ConditionMode::StartGroup => write!(f, "StartGroup"),
            ConditionMode::EndGroup => write!(f, "EndGroup"),
            ConditionMode::Subquery => write!(f, "Subquery"),
        }
    }
}

pub struct Condition<T> {
    field_name: String,
    values: Option<Arc<Vec<T>>>,
    where_subquery_parent: Option<*mut WhereStatement<T>>,
    subquery: Option<Arc<NvSelect<T>>>,
    operation: SqlOperator,
    value_size: u32,
    start_index: u32,
    param_index: u32,
    level: u32,
    logic_operator: LogicOperator,
    mode: ConditionMode,
    table_alias: String,
    dialect: DatabaseDialect,
}

impl<T> Condition<T> {
    fn process(start_index: u32, operation: SqlOperator, value_size: u32) -> u32 {
        match operation {
            SqlOperator::Between if value_size == 2 => start_index + 2,
            SqlOperator::In => start_index + value_size,
            _ => start_index + 1,
        }
    }

    pub fn new_comparator(
        field_name: String,
        op: SqlOperator,
        value_size: u32,
        param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        let param_index = Self::process(param_index, op, value_size);
        Self {
            field_name,
            values: None,
            where_subquery_parent: None,
            subquery: None,
            operation: op,
            value_size,
            start_index: param_index,
            param_index,
            level,
            logic_operator: LogicOperator::And,
            mode: ConditionMode::Comparator,
            table_alias: String::new(),
            dialect,
        }
    }

    pub fn new_logic(
        logic_operator: LogicOperator,
        mode: ConditionMode,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            field_name: String::new(),
            values: None,
            where_subquery_parent: None,
            subquery: None,
            operation: SqlOperator::Equal, // Placeholder
            value_size: 0,
            start_index: 0,
            param_index: 0,
            level,
            logic_operator,
            mode,
            table_alias: String::new(),
            dialect,
        }
    }

    pub fn new_subquery(
        parameter_values: Arc<Vec<T>>,
        parent: *mut WhereStatement<T>,
        field_name: String,
        subquery_name: String,
        op: SqlOperator,
        param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            field_name,
            values: Some(parameter_values.clone()),
            where_subquery_parent: Some(parent),
            subquery: Some(Arc::new(NvSelect::new_with_subquery(
                parameter_values,
                parent,
                param_index,
                level + 1,
                subquery_name.clone(),
                dialect,
            ))),
            operation: op,
            value_size: 0,
            start_index: param_index,
            param_index,
            level,
            logic_operator: LogicOperator::And,
            mode: ConditionMode::Subquery,
            table_alias: subquery_name,
            dialect,
        }
    }

    pub fn start_parameter_index(&self) -> u32 {
        self.start_index
    }

    pub fn next_parameter_index(&self) -> u32 {
        self.param_index
    }

    pub fn subquery(&self) -> &NvSelect<T> {
        self.subquery.as_ref().unwrap()
    }

    pub fn subquery_table_alias(&self) -> &String {
        &self.table_alias
    }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        let mut ss = String::new();
        let mut index = self.start_index;
        match self.mode {
            ConditionMode::StartGroup => {
                ss.push('(');
            }
            ConditionMode::LogicalOperator => {
                ss.push_str(&format!("{}", self.logic_operator));
            }
            ConditionMode::Subquery => {
                ss.push_str(&format!(
                    "{} {} ({})",
                    self.field_name,
                    self.operation,
                    self.__generate_query_from_subquery(pretty_print)
                ));
                if !self.table_alias.is_empty() {
                    ss.push_str(&format!(" AS {}", self.table_alias));
                }
                ss.push(' ');
            }
            ConditionMode::Comparator => {
                ss.push_str(&format!(
                    "{} {} {}",
                    self.field_name,
                    self.operation,
                    self.determine_parameter_format(index)
                ));
                index += 1;
            }
            ConditionMode::EndGroup => {
                ss.push(')');
            }
        }
        ss
    }

    fn __generate_query_from_subquery(&self, pretty_print: bool) -> String {
        // Placeholder for actual subquery generation logic
        String::new()
    }

    fn determine_parameter_format(&self, index: u32) -> String {
        // Placeholder for actual parameter format logic
        format!("param{}", index)
    }
}

pub struct WhereStatement<T> {
    parent: Option<*mut NvSelect<T>>,
    values: Arc<Vec<T>>,
    conditions: Vec<Condition<T>>,
    level: u32,
    current_param_index: u32,
    dialect: DatabaseDialect,
}

impl<T> WhereStatement<T> {
    pub fn new(dialect: DatabaseDialect) -> Self {
        Self {
            parent: None,
            values: Arc::new(Vec::new()),
            conditions: Vec::new(),
            level: 0,
            current_param_index: 1,
            dialect,
        }
    }

    pub fn new_with_parent(
        values: Arc<Vec<T>>,
        parent: *mut NvSelect<T>,
        current_param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            parent: Some(parent),
            values,
            conditions: Vec::new(),
            level,
            current_param_index,
            dialect,
        }
    }

    pub fn update_current_parameter_index(&mut self, parameter_index: u32) {
        self.current_param_index = parameter_index;
    }

    pub fn get_current_parameter_index(&self) -> u32 {
        self.current_param_index
    }

    pub fn values(&self) -> Arc<Vec<T>> {
        self.values.clone()
    }

    pub fn end_where_block(&mut self) -> &NvSelect<T> {
        match self.parent {
            Some(parent) => unsafe { &*parent },
            None => panic!("null-reference to parent of NvSelect<T>"),
        }
    }

    pub fn generate_query(&self, pretty_print: bool, append_where_keyword: bool) -> String {
        let mut where_clause = String::new();
        if append_where_keyword {
            where_clause.push_str("WHERE ");
        }

        for c in &self.conditions {
            where_clause.push_str(&c.generate_query(pretty_print));
        }

        where_clause
    }

    pub fn add_condition<U>(&mut self, field_name: String, op: SqlOperator, value: U) -> &mut Self
    where
        U: Into<T>,
    {
        let condition = Condition::new_comparator(
            field_name,
            op,
            1,
            self.current_param_index,
            self.level + 1,
            self.dialect,
        );
        self.current_param_index = condition.next_parameter_index();
        self.values.push(value.into());
        self.conditions.push(condition);
        self
    }

    pub fn add_condition_between<U>(
        &mut self,
        field_name: String,
        value1: U,
        value2: U,
    ) -> &mut Self
    where
        U: Into<T>,
    {
        self.values.push(value1.into());
        self.values.push(value2.into());
        let condition = Condition::new_comparator(
            field_name,
            SqlOperator::Between,
            2,
            self.current_param_index,
            self.level + 1,
            self.dialect,
        );
        self.current_param_index = condition.next_parameter_index();
        self.conditions.push(condition);
        self
    }

    pub fn add_condition_in<U>(&mut self, field_name: String, values: Vec<U>) -> &mut Self
    where
        U: Into<T>,
    {
        let size = values.len() as u32;
        let condition = Condition::new_comparator(
            field_name,
            SqlOperator::In,
            size,
            self.current_param_index,
            self.level + 1,
            self.dialect,
        );
        self.current_param_index = condition.next_parameter_index();
        for value in values {
            self.values.push(value.into());
        }
        self.conditions.push(condition);
        self
    }

    pub fn and(&mut self) -> &mut Self {
        self.conditions.push(Condition::new_logic(
            LogicOperator::And,
            ConditionMode::LogicalOperator,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn or(&mut self) -> &mut Self {
        self.conditions.push(Condition::new_logic(
            LogicOperator::Or,
            ConditionMode::LogicalOperator,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn start_group(&mut self) -> &mut Self {
        self.conditions.push(Condition::new_logic(
            LogicOperator::Or,
            ConditionMode::StartGroup,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn end_group(&mut self) -> &mut Self {
        self.conditions.push(Condition::new_logic(
            LogicOperator::Or,
            ConditionMode::EndGroup,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn add_subquery(
        &mut self,
        field_name: String,
        op: SqlOperator,
        subquery_name: String,
    ) -> &NvSelect<T> {
        let condition = Condition::new_subquery(
            self.values.clone(),
            self as *mut _,
            field_name,
            subquery_name,
            op,
            self.current_param_index,
            self.level + 1,
            self.dialect,
        );
        self.current_param_index = condition.next_parameter_index();
        self.conditions.push(condition);
        self.conditions.last().unwrap().subquery()
    }
}
