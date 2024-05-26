#![allow(dead_code)]

use crate::sqlbuilder::{DatabaseDialect, NvSelect};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FromTable {
    table: String,
    table_alias: Option<String>,
}

impl FromTable {
    pub fn new() -> Self {
        Self {
            table: String::new(),
            table_alias: None,
        }
    }

    pub fn with_alias(table: String, alias: Option<String>) -> Self {
        Self {
            table,
            table_alias: alias,
        }
    }

    pub fn build_table_name(&self) -> String {
        match &self.table_alias {
            Some(alias) => format!("{} AS {}", self.table, alias),
            None => self.table.clone(),
        }
    }
}


pub struct FromTableStatement<T> {
    parent: Option<*mut NvSelect<T>>,
    tables: Vec<FromTable>,
    subqueries: Vec<NvSelect<T>>,
    parameter_values: Arc<Vec<T>>,
    level: u32,
    current_parameter_index: u32,
    dialect: DatabaseDialect,
}

impl<T> FromTableStatement<T> {
    pub fn new(
        values: Arc<Vec<T>>,
        parent: *mut NvSelect<T>,
        parameter_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            parent: Some(parent),
            tables: Vec::new(),
            subqueries: Vec::new(),
            parameter_values: values,
            level,
            current_parameter_index: parameter_index,
            dialect,
        }
    }

    pub fn add_table(&mut self, table: FromTable) -> &mut Self {
        self.tables.push(table);
        self
    }

    pub fn add_table_with_alias(
        &mut self,
        table_name: String,
        table_alias: Option<String>,
    ) -> &mut Self {
        self.tables.push(FromTable::with_alias(table_name, table_alias));
        self
    }

    pub fn get_current_parameter_index(&self) -> u32 {
        self.current_parameter_index
    }

    pub fn update_current_param_index(&mut self, param_index: u32) {
        self.current_parameter_index = param_index;
    }

    pub fn get_current_parameter_index_from_parent(&self, select: &NvSelect<T>) -> u32 {
        select.get_current_param_index()
    }

    pub fn begin_subquery(&mut self, table_alias: String) -> &mut NvSelect<T> {
        let index = self.current_parameter_index;
        self.create_new_select_block(index, self.level + 1, table_alias);
        self.subqueries.last_mut().unwrap()
    }
    
    


    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    pub fn end_from_table_block(&mut self) -> &mut NvSelect<T> {
        match self.parent {
            Some(parent) => unsafe { &mut *parent },
            None => panic!("EndFromTableBlock() null-reference to parent"),
        }
    }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        let mut query = String::new();
        let mut first_element = true;
        for table in &self.tables {
            if !first_element {
                query.push_str(if pretty_print { ",\n" } else { ", " });
            }
            query.push_str(if pretty_print {
                 &self.generate_indentation(self.level + 1).clone()
            } else {
                ""
            });
            query.push_str(&table.build_table_name());
            first_element = false;
        }

        if !self.subqueries.is_empty() {
            for subquery in &self.subqueries {
                let alias = self.get_table_alias_from_parent(subquery);
                if !first_element {
                    query.push_str(if pretty_print { ",\n" } else { ", " });
                }
                query.push_str(if pretty_print {
                    &self.generate_indentation(self.level + 1)
                } else {
                    ""
                });
                query.push_str(&format!(
                    "(\n{}{})",
                    self.generate_select_query(subquery, pretty_print),
                    if alias.is_empty() {
                        "".to_string()
                    } else {
                        format!(" AS {}", alias)
                    }
                ));
            }
        }
        query
    }

    pub fn generate_select_block(&self, select: &NvSelect<T>) -> String {
        select.generate_query(false)
    }

    

    pub fn create_new_select_block(
        &mut self,
        index: u32,
        level: u32,
        table_alias: String,
    ) {
        self.subqueries.push(NvSelect::new_subquery_from(
            self.parameter_values.clone(),
            index,
            level,
            self as *mut _,
            table_alias,
            self.dialect,
        ));
    }

    pub fn generate_select_query(&self, select: &NvSelect<T>, pretty_print: bool) -> String {
        select.generate_query(pretty_print)
    }

    pub fn get_table_alias_from_parent(&self, select: &NvSelect<T>) -> String {
        select.table_alias().to_string()
    }
    fn generate_indentation(&self, level: u32) -> String {
        std::iter::repeat("  ").take(level as usize).collect::<String>().clone()
    }
}
