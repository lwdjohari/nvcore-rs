#![allow(dead_code)]

use crate::sqlbuilder::{DatabaseDialect, JoinDefMode, NvSelect, SqlJoinType, SqlOperator};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RecordKey {
    table: String,
    field: String,
    table_alias: Option<String>,
    initialize: bool,
}

impl RecordKey {
    pub fn new() -> Self {
        Self {
            table: String::new(),
            field: String::new(),
            table_alias: None,
            initialize: false,
        }
    }

    pub fn with_alias(table: String, field: String, alias: Option<String>) -> Self {
        Self {
            table,
            field,
            table_alias: alias,
            initialize: true,
        }
    }

    pub fn build_field(&self) -> String {
        match &self.table_alias {
            Some(alias) => format!("{}.{}", alias, self.field),
            None => format!("{}.{}", self.table, self.field),
        }
    }

    pub fn build_table_name(&self) -> String {
        match &self.table_alias {
            Some(alias) => format!("{} AS {}", self.table, alias),
            None => self.table.clone(),
        }
    }
}

pub struct JoinDef<T> {
    subquery_str: String,
    subsquery_str_alias: String,
    subquery_field_key: String,
    subquery_obj: Option<Arc<NvSelect<T>>>,
    left_table: RecordKey,
    right_table: RecordKey,
    join_type: SqlJoinType,
    join_mode: JoinDefMode,
    level: u32,
    dialect: DatabaseDialect,
}

impl<T> JoinDef<T> {
    pub fn new_record_key_both(
        left_table: RecordKey,
        right_table: RecordKey,
        join: SqlJoinType,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            subquery_str: String::new(),
            subsquery_str_alias: String::new(),
            subquery_field_key: String::new(),
            subquery_obj: None,
            left_table,
            right_table,
            join_type: join,
            join_mode: JoinDefMode::RecordKeyBoth,
            level,
            dialect,
        }
    }

    pub fn new_subquery_select_string(
        left_table: RecordKey,
        join: SqlJoinType,
        subquery: String,
        subquery_field_key: String,
        subquery_table_alias: String,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            subquery_str: subquery,
            subsquery_str_alias: subquery_table_alias,
            subquery_field_key,
            subquery_obj: None,
            left_table,
            right_table: RecordKey::new(),
            join_type: join,
            join_mode: JoinDefMode::SubquerySelectString,
            level,
            dialect,
        }
    }

    pub fn new_subquery_select_object(
        existing_table: RecordKey,
        subquery: NvSelect<T>,
        join: SqlJoinType,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            subquery_str: String::new(),
            subsquery_str_alias: String::new(),
            subquery_field_key: String::new(),
            subquery_obj: Some(Arc::new(subquery)),
            left_table: existing_table,
            right_table: RecordKey::new(),
            join_type: join,
            join_mode: JoinDefMode::SubquerySelectObject,
            level,
            dialect,
        }
    }

    pub fn join_type(&self) -> SqlJoinType {
        self.join_type
    }

    pub fn mode(&self) -> JoinDefMode {
        self.join_mode
    }

    pub fn left_table(&self) -> &RecordKey {
        &self.left_table
    }

    pub fn right_table(&self) -> &RecordKey {
        &self.right_table
    }

    pub fn subquery(&self) -> &NvSelect<T> {
        self.subquery_obj.as_ref().unwrap()
    }

    pub fn has_subquery_object(&self) -> bool {
        self.subquery_obj.is_some()
    }

    pub fn subquery_string(&self) -> &String {
        &self.subquery_str
    }

    pub fn subquery_alias_string(&self) -> &String {
        &self.subsquery_str_alias
    }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        match self.join_mode {
            JoinDefMode::RecordKeyBoth => self.generate_join_record_both(pretty_print),
            JoinDefMode::SubquerySelectString => self.generate_join_record_subquery_string(),
            JoinDefMode::SubquerySelectObject => self.generate_join_record_subquery_object(),
            JoinDefMode::SubqueryRawString => String::new()
        }
    }

    fn generate_join_record_both(&self, pretty_print: bool) -> String {
        match self.join_type {
            SqlJoinType::InnerJoin => {
                self.generate_inner_join(&self.left_table, &self.right_table, pretty_print)
            }
            SqlJoinType::LeftJoin => {
                self.generate_left_join(&self.left_table, &self.right_table, pretty_print)
            }
            SqlJoinType::RightJoin => self.generate_right_join(&self.left_table, &self.right_table),
            
            SqlJoinType::None => String::new()
        }
    }

    fn generate_join_record_subquery_string(&self) -> String {
        match self.join_type {
            SqlJoinType::InnerJoin => self.generate_inner_join_from_str(
                &self.left_table,
                &self.subquery_str,
                &self.subquery_field_key,
                &self.subsquery_str_alias,
                SqlOperator::Equal,
            ),
            SqlJoinType::LeftJoin => self.generate_left_join_from_str (
                &self.left_table,
                &self.subquery_str,
                &self.subquery_field_key,
                &self.subsquery_str_alias,
                SqlOperator::Equal,
            ),
            SqlJoinType::RightJoin => self.generate_right_join_from_str (
                &self.left_table,
                &self.subquery_str,
                &self.subquery_field_key,
                &self.subsquery_str_alias,
                SqlOperator::Equal,
            ),
            SqlJoinType::None => String::new()
        }
    }

    

    fn generate_left_join(
        &self,
        left_key: &RecordKey,
        right_key: &RecordKey,
        pretty_print: bool,
    ) -> String {
        let mut join = String::new();
        if pretty_print {
            join.push_str(&format!(
                "{}LEFT JOIN\n{}{} ON\n{}{} = {}",
                self.generate_indentation(self.level),
                self.generate_indentation(self.level + 1),
                right_key.build_table_name(),
                self.generate_indentation(self.level + 2),
                left_key.build_field(),
                right_key.build_field(),
            ));
        } else {
            join.push_str(&format!(
                "LEFT JOIN {} ON {} = {}",
                right_key.build_table_name(),
                left_key.build_field(),
                right_key.build_field(),
            ));
        }
        join
    }

    fn generate_left_join_from_str(
        &self,
        right_table: &RecordKey,
        left_table: &str,
        left_table_field_key: &str,
        left_table_alias: &str,
        op: SqlOperator,
    ) -> String {
        let mut join = String::new();
        join.push_str(&format!(
            "LEFT JOIN ({}) {} ON {} {} {}",
            left_table,
            if !left_table_alias.is_empty() {
                format!(" AS {}", left_table_alias)
            } else {
                String::new()
            },
            if !left_table_alias.is_empty() {
                format!("{}.{}", left_table_alias, left_table_field_key)
            } else {
                left_table_field_key.to_string()
            },
            op,
            right_table.build_field(),
        ));
        join
    }

    fn generate_right_join(&self, left_table: &RecordKey, right_table: &RecordKey) -> String {
        format!(
            "RIGHT JOIN {} ON {} = {}",
            right_table.build_table_name(),
            left_table.build_field(),
            right_table.build_field(),
        )
    }

    fn generate_right_join_from_str(
        &self,
        left_table: &RecordKey,
        right_table: &str,
        right_table_field_key: &str,
        right_table_alias: &str,
        op: SqlOperator,
    ) -> String {
        let mut join = String::new();
        join.push_str(&format!(
            "RIGHT JOIN ({}) {} ON {} {} {}",
            right_table,
            if !right_table_alias.is_empty() {
                format!(" AS {}", right_table_alias)
            } else {
                String::new()
            },
            left_table.build_field(),
            op,
            if !right_table_alias.is_empty() {
                format!("{}.{}", right_table_alias, right_table_field_key)
            } else {
                right_table_field_key.to_string()
            },
        ));
        join
    }

    fn generate_inner_join(
        &self,
        existing_select: &RecordKey,
        join_on_table: &RecordKey,
        pretty_print: bool,
    ) -> String {
        let mut join = String::new();
        if pretty_print {
            join.push_str(&format!(
                "{}INNER JOIN\n{}{} ON\n{}{} = {}",
                self.generate_indentation(self.level),
                self.generate_indentation(self.level + 1),
                join_on_table.build_table_name(),
                self.generate_indentation(self.level + 2),
                existing_select.build_field(),
                join_on_table.build_field(),
            ));
        } else {
            join.push_str(&format!(
                "INNER JOIN {} ON {} = {}",
                join_on_table.build_table_name(),
                existing_select.build_field(),
                join_on_table.build_field(),
            ));
        }
        join
    }

    fn generate_inner_join_from_str(
        &self,
        existing_select: &RecordKey,
        join_on_table: &str,
        join_table_field_key: &str,
        join_table_alias: &str,
        op: SqlOperator,
    ) -> String {
        let mut join = String::new();
        join.push_str(&format!(
            "INNER JOIN ({}) {} ON {} {} {}",
            join_on_table,
            if !join_table_alias.is_empty() {
                format!(" AS {}", join_table_alias)
            } else {
                String::new()
            },
            existing_select.build_field(),
            op,
            if !join_table_alias.is_empty() {
                format!("{}.{}", join_table_alias, join_table_field_key)
            } else {
                join_table_field_key.to_string()
            },
        ));
        join
    }

    fn generate_indentation(&self, level: u32) -> String {
        std::iter::repeat("  ")
            .take(level as usize)
            .collect::<String>()
    }

    pub fn generate_join_record_subquery_object(&self) -> String {
        match &self.subquery_obj {
            Some(subquery) => subquery.generate_query(false),
            None => String::new(),
        }
    }
}

pub struct JoinStatement<T> {
    parent: Option<*mut NvSelect<T>>,
    joins: Vec<JoinDef<T>>,
    current_parameter_index: u32,
    level: u32,
    dialect: DatabaseDialect,
}

impl<T> JoinStatement<T> {
    pub fn new(
        parent: *mut NvSelect<T>,
        parameter_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            parent: Some(parent),
            joins: Vec::new(),
            current_parameter_index: parameter_index,
            level,
            dialect,
        }
    }

    pub fn end_join_block(&mut self) -> &NvSelect<T> {
        match self.parent {
            Some(parent) => unsafe { &mut *parent },
            None => panic!("EndFromTableBlock() null-reference to parent"),
        }
    }

    pub fn generate_select_block(&self, select: &NvSelect<T>) -> String {
        select.generate_query(false)
    }

    pub fn get_join_clauses(&self) -> &Vec<JoinDef<T>> {
        &self.joins
    }

    pub fn is_empty(&self) -> bool {
        self.joins.is_empty()
    }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        let mut query = String::new();
        let mut is_first_element = true;
        for clause in &self.joins {
            if !is_first_element {
                query.push_str(if pretty_print { "\n" } else { " " });
            }
            query.push_str(&clause.generate_query(pretty_print));
            is_first_element = false;
        }
        query
    }

    pub fn left_join(&mut self, left_table: RecordKey, right_table: RecordKey) -> &mut Self {
        self.joins.push(JoinDef::new_record_key_both(
            left_table,
            right_table,
            SqlJoinType::LeftJoin,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn left_join_with_query(
        &mut self,
        right_table: RecordKey,
        left_table: String,
        left_table_field_key: String,
        left_table_alias: String,
        op: SqlOperator,
    ) -> &mut Self {
        self.joins.push(JoinDef::new_subquery_select_string(
            right_table,
            SqlJoinType::LeftJoin,
            left_table,
            left_table_field_key,
            left_table_alias,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn right_join(&mut self, left_table: RecordKey, right_table: RecordKey) -> &mut Self {
        self.joins.push(JoinDef::new_record_key_both(
            left_table,
            right_table,
            SqlJoinType::RightJoin,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn right_join_with_query(
        &mut self,
        left_table: RecordKey,
        right_table: String,
        right_table_field_key: String,
        right_table_alias: Option<String>,
        op: SqlOperator,
    ) -> &mut Self {
        self.joins.push(JoinDef::new_subquery_select_string(
            left_table,
            SqlJoinType::RightJoin,
            right_table,
            right_table_field_key,
            right_table_alias.unwrap_or_default(),
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn inner_join(
        &mut self,
        existing_select: RecordKey,
        join_on_table: RecordKey,
    ) -> &mut Self {
        self.joins.push(JoinDef::new_record_key_both(
            existing_select,
            join_on_table,
            SqlJoinType::InnerJoin,
            self.level,
            self.dialect,
        ));
        self
    }

    pub fn inner_join_with_query(
        &mut self,
        existing_select: RecordKey,
        join_on_table: String,
        join_table_field_key: String,
        join_table_alias: Option<String>,
        op: SqlOperator,
    ) -> &mut Self {
        self.joins.push(JoinDef::new_subquery_select_string(
            existing_select,
            SqlJoinType::InnerJoin,
            join_on_table,
            join_table_field_key,
            join_table_alias.unwrap_or_default(),
            self.level,
            self.dialect,
        ));
        self
    }
}
