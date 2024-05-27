#![allow(dead_code)]

use crate::sqlbuilder::{DatabaseDialect, NvSelect};
use crate::utils::indent_space;
use std::sync::{Arc, RwLock};

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
    parent: RwLock<Option<Arc<NvSelect<T>>>>,
    tables: RwLock<Vec<FromTable>>,
    subqueries: RwLock<Vec<Arc<NvSelect<T>>>>,
    parameter_values: Arc<RwLock<Vec<T>>>,
    current_parameter_index: RwLock<u32>,
    level: u32,
    dialect: DatabaseDialect,
}

impl<T> FromTableStatement<T> {
    pub fn new(
        parameter_values: Arc<RwLock<Vec<T>>>,
        parent: Arc<NvSelect<T>>,
        parameter_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            parent: Some( parent).into(),
            tables: Vec::new().into(),
            subqueries: Vec::new().into(),
            parameter_values:  parameter_values,
            level: level,
            current_parameter_index: parameter_index.into(),
            dialect: dialect,
        })
    }

    pub fn add_table(self: Arc<Self>, table: FromTable) -> Arc<Self> {
        let mut write_guard = self.tables.write().unwrap();
        write_guard.push(table);
        self.clone()
    }

    pub fn add_table_with_alias(
        self: Arc<Self>,
        table_name: &String,
        table_alias: &Option<String>,
    ) -> Arc<Self> {
        {
        let mut write_guard = self.tables.write().unwrap();
        write_guard.push(FromTable::with_alias(table_name.clone(), table_alias.clone()));
        }
        self.clone()
    }

    pub fn get_current_parameter_index(&self) -> u32 {
        *self.current_parameter_index.read().unwrap()
    }

    pub fn update_current_parameter_index(&mut self, param_index: u32) {
        *self.current_parameter_index.write().unwrap() = param_index;
    }

    // pub fn get_current_parameter_index_from_parent(&self, select: &NvSelect<T>) -> u32 {
    //     select.get_current_param_index()
    // }

    pub fn begin_subquery(self:Arc<Self>, table_alias: String) -> Arc<NvSelect<T>> {
        let index = *self.current_parameter_index.read().unwrap();
        let level = self.level.clone();
        self.create_new_select_block(index, level + 1, table_alias)
    }

    pub fn is_empty(&self) -> bool {
        self.tables.read().unwrap().is_empty()
    }

    pub fn end_from_table_block( self:Arc<Self>) -> Arc<NvSelect<T>> {
        let parent_guard = self.parent.read().unwrap();
        if parent_guard.is_none() {
            panic!("EndFromTableBlock() null-reference to parent");
        }

        // Cloning the Arc inside the Option
        Arc::clone(parent_guard.as_ref().unwrap())
         
    }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        let mut query = String::new();
        let mut first_element = true;

        let table_guard = self.tables.read().unwrap();
        let sq_guard = self.subqueries.read().unwrap();
        
        let is_subqueries_empty = sq_guard.is_empty();

        let indent_level = self.level.clone() +1;
        let indentation = indent_space(indent_level.clone()); 
        for table in table_guard.iter() {
            if !first_element {
                query.push_str(if pretty_print { ",\n" } else { ", " });
            }
            query.push_str(if pretty_print {
                &indentation
            } else {
                ""
            });
            query.push_str(&table.build_table_name());
            first_element = false;
        }

        if !is_subqueries_empty {
            for subquery in sq_guard.iter() {
                let alias = self.get_table_alias_from_parent(subquery);
                if !first_element {
                    query.push_str(if pretty_print { ",\n" } else { ", " });
                }
                query.push_str(if pretty_print {
                    &indentation
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

     fn create_new_select_block( self:Arc<Self>, index: u32, level: u32, table_alias: String)->Arc<NvSelect<T>> {
        
        let subquery = NvSelect::new_subquery_from(
            self.parameter_values.clone(),
            index,
            level,
            self.clone(),
            table_alias,
            self.dialect.clone(),
        );

        let subquery_to_return = subquery.clone();

        let mut sq_guard = self.subqueries.write().unwrap();
        sq_guard.push(subquery);

        subquery_to_return
    }

    pub fn generate_select_query(&self, select: &NvSelect<T>, pretty_print: bool) -> String {
        select.generate_query(pretty_print)
    }

    pub fn get_table_alias_from_parent(&self, select: &NvSelect<T>) -> String {
        select.table_alias().to_string()
    }
    
}
