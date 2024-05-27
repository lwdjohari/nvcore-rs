#![allow(dead_code)]

use crate::sqlbuilder::NvSelect;
use crate::sqlbuilder::{determine_parameter_format, DatabaseDialect, LogicOperator, SqlOperator};
use std::sync::{Arc, RwLock};

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
    where_subquery_parent: RwLock<Option<Arc<WhereStatement<T>>>>,
    subquery: RwLock<Option<Arc<NvSelect<T>>>>,
    operation: SqlOperator,
    value_size: u32,
    start_index: u32,
    param_index: RwLock<u32>,
    level: u32,
    logic_operator: LogicOperator,
    mode: ConditionMode,
    table_alias: String,
    dialect: DatabaseDialect,
}

impl<T> Condition<T> {
    fn process(start_index: u32, operation: &SqlOperator, value_size: u32) -> u32 {
        match operation {
            SqlOperator::Between if value_size == 2 => start_index + 2,
            SqlOperator::In => start_index + value_size,
            _ => start_index + 1,
        }
    }

    pub fn new_comparator(
        field_name: &String,
        op: &SqlOperator,
        value_size: u32,
        param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        let param_index = Self::process(param_index, op, value_size);
        Arc::new(Self {
            field_name: field_name.clone(),
            values: None,
            where_subquery_parent: None.into(),
            subquery: None.into(),
            operation: op.clone(),
            value_size,
            start_index: param_index.clone(),
            param_index: param_index.clone().into(),
            level,
            logic_operator: LogicOperator::And,
            mode: ConditionMode::Comparator,
            table_alias: String::new(),
            dialect,
        })
    }

    pub fn new_logic(
        logic_operator: LogicOperator,
        mode: ConditionMode,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            field_name: String::new(),
            values: None.into(),
            where_subquery_parent: None.into(),
            subquery: None.into(),
            operation: SqlOperator::Equal, // Placeholder
            value_size: 0,
            start_index: 0,
            param_index: 0.into(),
            level,
            logic_operator,
            mode,
            table_alias: String::new(),
            dialect,
        })
    }

    pub fn new_subquery(
        parameter_values: Arc<Vec<T>>,
        parent: Arc<WhereStatement<T>>,
        field_name: String,
        subquery_name: String,
        op: SqlOperator,
        param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            field_name,
            values: Some(parameter_values.clone()),
            where_subquery_parent: Some(parent.clone()).into(),
            subquery: Some(NvSelect::new_subquery_where(
                parameter_values,
                parent,
                param_index,
                level + 1,
                subquery_name.clone(),
                dialect,
            ))
            .into(),
            operation: op,
            value_size: 0,
            start_index: param_index.clone(),
            param_index: param_index.clone().into(),
            level,
            logic_operator: LogicOperator::And,
            mode: ConditionMode::Subquery,
            table_alias: subquery_name,
            dialect,
        })
    }

    pub fn start_parameter_index(&self) -> u32 {
        self.start_index
    }

    pub fn next_parameter_index(&self) -> u32 {
        *self.param_index.read().unwrap()
    }

    pub fn subquery(&self) -> Arc<NvSelect<T>> {
        // self.subquery.read().unwrap().unwrap().clone()
        let sq_guard: std::sync::RwLockReadGuard<Option<Arc<NvSelect<T>>>> =
            self.subquery.read().unwrap();
        if !sq_guard.is_some() {
            panic!("Subquery not initialize!");
        }

        Arc::clone(sq_guard.as_ref().unwrap())
    }

    pub fn subquery_table_alias(&self) -> &String {
        &self.table_alias
    }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        let mut ss = String::new();
        let index = self.start_index;
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
                    self.generate_query_from_subquery(pretty_print)
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
                    determine_parameter_format(&self.dialect, index)
                ));
            }
            ConditionMode::EndGroup => {
                ss.push(')');
            }
        }
        ss
    }

    pub fn generate_query_from_subquery(&self, pretty_print: bool) -> String {
        let sq_guard = self.subquery.read().unwrap();
        if !sq_guard.is_some() {
            return String::new();
        }

        let sq_ref = sq_guard.as_ref().unwrap();
        sq_ref.generate_query(pretty_print)
    }

    // pub fn generate_query_from_subquery(&self, pretty_print: bool) -> String {
    //     match &self.subquery {
    //         Some(subquery) => subquery.generate_query(pretty_print),
    //         None => String::new(),
    //     }
    // }
}

pub struct WhereStatement<T> {
    parent: RwLock<Option<Arc<NvSelect<T>>>>,
    values: RwLock<Arc<Vec<T>>>,
    conditions: RwLock<Vec<Arc<Condition<T>>>>,
    level: u32,
    current_param_index: RwLock<u32>,
    dialect: DatabaseDialect,
}

impl<T> WhereStatement<T> {
    pub fn new(dialect: DatabaseDialect) -> Arc<Self> {
        Arc::new(Self {
            parent: None.into(),
            values: Arc::new(Vec::new()).into(),
            conditions: Vec::new().into(),
            level: 0,
            current_param_index: 1.into(),
            dialect,
        })
    }

    pub fn new_with_parent(
        values: Arc<Vec<T>>,
        parent: Arc<NvSelect<T>>,
        current_param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            parent: Some(parent).into(),
            values: values.into(),
            conditions: Vec::new().into(),
            level,
            current_param_index: current_param_index.into(),
            dialect,
        })
    }

    pub fn update_current_parameter_index(&mut self, parameter_index: u32) {
        *self.current_param_index.write().unwrap() = parameter_index;
    }

    pub fn get_current_parameter_index(&self) -> u32 {
        *self.current_param_index.read().unwrap()
    }

    pub fn values(self: Arc<Self>) -> Arc<Vec<T>> {
        self.values.read().unwrap().clone()
    }

    pub fn end_where_block(self: Arc<Self>) -> Arc<NvSelect<T>> {
        let parent_guard = self.parent.read().unwrap();
        if parent_guard.is_none() {
            panic!("EndWhereBlock() null-reference to parent");
        }

        // Cloning the Arc inside the Option
        Arc::clone(parent_guard.as_ref().unwrap())
    }

    pub fn generate_query(&self, pretty_print: bool, append_where_keyword: bool) -> String {
        let mut where_clause = String::new();
        if append_where_keyword {
            where_clause.push_str("WHERE ");
        }

        let conditions_guard = self.conditions.read().unwrap();

        for c in conditions_guard.iter() {
            where_clause.push_str(&c.generate_query(pretty_print));
        }

        where_clause
    }

    pub fn add_condition(
        self: Arc<Self>,
        field_name: &String,
        op: &SqlOperator,
        value: T,
    ) -> Arc<Self> {
        {
            let condition: Arc<Condition<T>> = Condition::new_comparator(
                field_name,
                op,
                1,
                *self.current_param_index.read().unwrap(),
                self.level + 1,
                self.dialect,
            );

            *self.current_param_index.write().unwrap() = condition.next_parameter_index();
            self.conditions.write().unwrap().push(condition);
        }
        // {
        //     let mut pvalues_guard = self.values.write().unwrap();
        //     let pvalues_ref = Arc::get_mut(&mut pvalues_guard)
        //         .unwrap();

        //     pvalues_ref.push(value);
        // }
        self.clone()
    }

    pub fn add_condition_between(
        self: Arc<Self>,
        field_name: &String,
        value1: T,
        value2: T,
    ) -> Arc<Self> {
        let condition = Condition::new_comparator(
            field_name,
            &SqlOperator::Between,
            2,
            self.current_param_index.read().unwrap().clone(),
            self.level + 1,
            self.dialect,
        );

        *self.current_param_index.write().unwrap() = condition.next_parameter_index();
        self.conditions.write().unwrap().push(condition);

        let mut pvalues_guard = self.values.write().unwrap();
        let pvalues_ref = Arc::get_mut(&mut pvalues_guard)
            .expect("There should be no other references to the Arc at this point");

        pvalues_ref.push(value1);
        pvalues_ref.push(value2);

        self.clone()
    }

    pub fn and(self: Arc<Self>) -> Arc<Self> {
        self.conditions.write().unwrap().push(Condition::new_logic(
            LogicOperator::And,
            ConditionMode::LogicalOperator,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn or(self: Arc<Self>) -> Arc<Self> {
        self.conditions.write().unwrap().push(Condition::new_logic(
            LogicOperator::Or,
            ConditionMode::LogicalOperator,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn start_group(self: Arc<Self>) -> Arc<Self> {
        self.conditions.write().unwrap().push(Condition::new_logic(
            LogicOperator::Or,
            ConditionMode::StartGroup,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn end_group(self: Arc<Self>) -> Arc<Self> {
        self.conditions.write().unwrap().push(Condition::new_logic(
            LogicOperator::Or,
            ConditionMode::EndGroup,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn add_subquery(
        self: Arc<Self>,
        field_name: String,
        op: SqlOperator,
        subquery_name: String,
    ) -> Arc<NvSelect<T>> {
        let param_index = self.current_param_index.read().unwrap().clone();
        let level = self.level + 1;
        let dialect = self.dialect;

        let condition = Condition::new_subquery(
            self.values.read().unwrap().clone(),
            self.clone(),
            field_name,
            subquery_name,
            op,
            param_index,
            level,
            dialect,
        );

        let cond_ref = condition.as_ref();
        let next_index = cond_ref.next_parameter_index();
        let subquery = cond_ref.subquery();
        *self.current_param_index.write().unwrap() = next_index;
        self.conditions.write().unwrap().push(condition);
        subquery
    }
}

impl<T: Clone> WhereStatement<T> {
    pub fn add_condition_in(self: Arc<Self>, field_name: &String, values: &Vec<T>) -> Arc<Self> {
        let size = values.len() as u32;
        let condition = Condition::new_comparator(
            field_name,
            &SqlOperator::In,
            size,
            self.current_param_index.read().unwrap().clone(),
            self.level + 1,
            self.dialect.clone(),
        );
        *self.current_param_index.write().unwrap() = condition.next_parameter_index();
        self.conditions.write().unwrap().push(condition);

        let mut pvalues_guard = self.values.write().unwrap();
        let pvalues_ref = Arc::get_mut(&mut pvalues_guard)
            .expect("There should be no other references to the Arc at this point");

        for value in values {
            pvalues_ref.push(value.clone());
        }
        self.clone()
    }
}
