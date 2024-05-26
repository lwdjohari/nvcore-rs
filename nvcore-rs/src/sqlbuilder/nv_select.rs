#![allow(dead_code)]

use crate::sqlbuilder::{
    DatabaseDialect, FieldDef, FieldDefMode, FromTableStatement, GroupByStatement, JoinStatement,
    OrderByStatement, SqlAggregateFunction, WhereStatement, JoinDef, Condition
};
use std::sync::Arc;


pub struct NvSelect<T> {
    current_param_index: u32,
    level: u32,
    join_blocks: Vec<JoinStatement<T>>,
    from_table: Option<Arc<FromTableStatement<T>>>,
    fields: Vec<FieldDef<T>>,
    parameter_values: Arc<Vec<T>>,
    subquery_from_parent: Option<*mut FromTableStatement<T>>,
    table_alias: String,
    where_: Option<Arc<WhereStatement<T>>>,
    order_by: Option<Arc<OrderByStatement<T>>>,
    group_by: Option<Arc<GroupByStatement<T>>>,
    subquery_where_parent: Option<*mut WhereStatement<T>>,
    // limit_offset: Option<Arc<LimitOffsetStatement<T>>>,
    dialect: DatabaseDialect,
}

impl<T> NvSelect<T> {
    pub fn new(dialect: DatabaseDialect) -> Self {
        Self {
            current_param_index: 1,
            level: 0,
            join_blocks: Vec::new(),
            from_table: None,
            fields: Vec::new(),
            parameter_values: Arc::new(Vec::new()),
            subquery_from_parent: None,
            table_alias: String::new(),
            where_: None,
            order_by: None,
            group_by: None,
            subquery_where_parent: None,
            // limit_offset: None,
            dialect,
        }
    }

    pub fn with_param_index(current_param_index: u32, dialect: DatabaseDialect) -> Self {
        Self {
            current_param_index,
            level: 0,
            join_blocks: Vec::new(),
            from_table: None,
            fields: Vec::new(),
            parameter_values: Arc::new(Vec::new()),
            subquery_from_parent: None,
            table_alias: String::new(),
            where_: None,
            order_by: None,
            group_by: None,
            subquery_where_parent: None,
            // limit_offset: None,
            dialect,
        }
    }

    pub fn new_subquery(
        values: Arc<Vec<T>>,
        current_param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            current_param_index,
            level,
            join_blocks: Vec::new(),
            from_table: None,
            fields: Vec::new(),
            parameter_values: values,
            subquery_from_parent: None,
            table_alias: String::new(),
            where_: None,
            order_by: None,
            group_by: None,
            subquery_where_parent: None,
            // limit_offset: None,
            dialect,
        }
    }

    pub fn new_subquery_from(
        values: Arc<Vec<T>>,
        current_param_index: u32,
        level: u32,
        from_obj: *mut FromTableStatement<T>,
        table_alias: String,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            current_param_index,
            level,
            join_blocks: Vec::new(),
            from_table: None,
            fields: Vec::new(),
            parameter_values: values,
            subquery_from_parent: Some(from_obj),
            table_alias,
            where_: None,
            order_by: None,
            group_by: None,
            subquery_where_parent: None,
            // limit_offset: None,
            dialect,
        }
    }

    pub fn new_subquery_where(
        values: Arc<Vec<T>>,
        where_obj: *mut WhereStatement<T>,
        current_param_index: u32,
        level: u32,
        table_alias: String,
        dialect: DatabaseDialect,
    ) -> Self {
        Self {
            current_param_index,
            level,
            join_blocks: Vec::new(),
            from_table: None,
            fields: Vec::new(),
            parameter_values: values,
            subquery_from_parent: None,
            table_alias,
            where_: None,
            order_by: None,
            group_by: None,
            subquery_where_parent: Some(where_obj),
            // limit_offset: None,
            dialect,
        }
    }

    pub fn dialect(&self) -> DatabaseDialect {
        self.dialect
    }

    pub fn get_current_param_index(&self) -> u32 {
        self.current_param_index
    }

    pub fn update_current_param_index(&mut self, current_param_index: u32) {
        self.current_param_index = current_param_index;
    }

    pub fn table_alias(&self) -> &str {
        &self.table_alias
    }

    pub fn get_block_level(&self) -> u32 {
        self.level
    }

    pub fn field(
        &mut self,
        field: String,
        table_alias: Option<String>,
        field_alias: Option<String>,
        aggregate_fn: SqlAggregateFunction,
        enclose_field_name: bool,
    ) -> &mut Self {
        self.fields.push(FieldDef::new_field_def(
            self.dialect,
            field,
            table_alias,
            enclose_field_name,
            aggregate_fn,
            field_alias,
            self.level,
            FieldDefMode::FieldWType,
        ));
        self
    }

    pub fn f(
        &mut self,
        field: String,
        table_alias: Option<String>,
        field_alias: Option<String>,
        aggregate_fn: SqlAggregateFunction,
        enclose_field_name: bool,
    ) -> &mut Self {
        self.fields.push(FieldDef::new_field_def(
            self.dialect,
            field,
            table_alias,
            enclose_field_name,
            aggregate_fn,
            field_alias,
            self.level,
            FieldDefMode::FieldRaw,
        ));
        self
    }

    pub fn end_subquery_inside_from(&mut self) -> &mut FromTableStatement<T> {
        match self.subquery_from_parent {
            Some(parent) => unsafe {
                (*parent).update_current_param_index(self.current_param_index);
                &mut *parent
            },
            None => panic!("Call this only from .From().AddSubquery().EndFromSubquery()"),
        }
    }

    pub fn end_subquery_inside_where_condition(&mut self) -> &mut WhereStatement<T> {
        match self.subquery_where_parent {
            Some(parent) => unsafe {
                (*parent).update_current_parameter_index(self.current_param_index);
                &mut *parent
            },
            None => panic!("Call this only from .Where().AddSubquery().EndFromSubquery()"),
        }
    }

    pub fn from(&mut self) -> &mut FromTableStatement<T> {
        if self.from_table.is_none() {
            self.from_table = Some(Arc::new(FromTableStatement::new(
                self.parameter_values.clone(),
                self as *mut _,
                self.current_param_index,
                self.level,
                self.dialect,
            )));
        }
        Arc::get_mut(self.from_table.as_mut().unwrap()).unwrap()
    }

    pub fn where_(&mut self) -> &mut WhereStatement<T> {
        if self.where_.is_none() {
            self.where_ = Some(Arc::new(WhereStatement::new_with_parent(
                self.parameter_values.clone(),
                self as *mut _,
                self.current_param_index,
                self.level,
                self.dialect,
            )));
        }
        Arc::get_mut(self.where_.as_mut().unwrap()).unwrap()
    }

    pub fn join(&mut self) -> &mut JoinStatement<T> {
        self.join_blocks.push(JoinStatement::new(
            self as *mut _,
            self.current_param_index,
            self.level,
            self.dialect,
        ));
        self.join_blocks.last_mut().unwrap()
    }

    pub fn order_by(&mut self) -> &mut OrderByStatement<T> {
        if self.order_by.is_none() {
            self.order_by = Some(Arc::new(OrderByStatement::new_with_parent(
                self as *mut _,
                self.level,
            )));
        }
        Arc::get_mut(self.order_by.as_mut().unwrap()).unwrap()
    }

    pub fn group_by(&mut self) -> &mut GroupByStatement<T> {
        if self.group_by.is_none() {
            self.group_by = Some(Arc::new(GroupByStatement::new_with_parent(
                self as *mut _,
                self.current_param_index,
                self.level,
            )));
        }
        Arc::get_mut(self.group_by.as_mut().unwrap()).unwrap()
    }

    pub fn fn_static(
        &mut self,
        fn_name: String,
        param_values: Vec<String>,
        field_alias: Option<String>,
    ) -> &mut Self {
        self.fields.push(FieldDef::new_static_function(
            self.dialect,
            fn_name,
            param_values,
            self.level,
            field_alias,
        ));
        self
    }

    pub fn fn_dynamic(
        &mut self,
        fn_name: String,
        parameter_list_format: String,
        param_values: Vec<T>,
        static_param_values: Vec<String>,
        field_alias: Option<String>,
    ) -> &mut Self {
        self.fields.push(FieldDef::new_dynamic_function(
            self.dialect,
            fn_name,
            parameter_list_format,
            self.parameter_values.clone(),
            param_values,
            static_param_values,
            self.current_param_index,
            self.level,
            field_alias,
        ));
        self.current_param_index = self.fields.last().unwrap().get_current_parameter_index();
        self
    }

    // pub fn limit_offset(&mut self) -> &mut LimitOffsetStatement<T> {
    //     if self.limit_offset.is_none() {
    //         self.limit_offset = Some(Arc::new(LimitOffsetStatement::new(
    //             self as *mut _,
    //             self.parameter_values.clone(),
    //             self.current_param_index,
    //             self.level,
    //             self.dialect,
    //         )));
    //     }
    //     Arc::get_mut(self.limit_offset.as_mut().unwrap()).unwrap()
    // }

    pub fn generate_query(&self, pretty_print: bool) -> String {
        let mut query = String::new();

        // SELECT
        if pretty_print {
            query.push_str(&self.generate_indentation(self.level));
            query.push_str("SELECT \n");
        } else {
            query.push_str("SELECT ");
        }

        let mut first_element = true;
        for field in &self.fields {
            if !first_element {
                query.push_str(if pretty_print { ",\n" } else { ", " });
            }
            if pretty_print {
                query.push_str(&self.generate_indentation(self.level + 1));
            }
            query.push_str(&field.generate_query());
            first_element = false;
        }

        // FROM
        if let Some(ref from_table) = self.from_table {
            if !from_table.is_empty() {
                if pretty_print {
                    query.push_str(&format!(
                        "\n{}FROM \n{}",
                        self.generate_indentation(self.level),
                        from_table.generate_query(pretty_print)
                    ));
                } else {
                    query.push_str(&format!(" FROM {}", from_table.generate_query(pretty_print)));
                }
            }
        }

        // JOIN
        if !self.join_blocks.is_empty() {
            query.push_str(if pretty_print { "\n" } else { " " });
            for join_block in &self.join_blocks {
                query.push_str(&join_block.generate_query(pretty_print));
            }
        }

        // WHERE
        if let Some(ref where_) = self.where_ {
            if pretty_print {
                query.push_str(&format!(
                    "\n{}WHERE\n{}{}",
                    self.generate_indentation(self.level),
                    self.generate_indentation(self.level + 1),
                    where_.generate_query(pretty_print, false)
                ));
            } else {
                query.push_str(&format!(
                    " WHERE {}",
                    where_.generate_query(pretty_print, false)
                ));
            }
        }

        // GROUP BY
        if let Some(ref group_by) = self.group_by {
            if pretty_print {
                query.push_str(&format!(
                    "\n{}GROUP BY\n{}{}",
                    self.generate_indentation(self.level),
                    self.generate_indentation(self.level + 1),
                    group_by.generate_query(pretty_print)
                ));
            } else {
                query.push_str(&format!(
                    " GROUP BY {}",
                    group_by.generate_query(pretty_print)
                ));
            }
        }

        // ORDER BY
        if let Some(ref order_by) = self.order_by {
            if pretty_print {
                query.push_str(&format!(
                    "\n{}ORDER BY\n{}{}",
                    self.generate_indentation(self.level),
                    self.generate_indentation(self.level + 1),
                    order_by.generate_query(pretty_print)
                ));
            } else {
                query.push_str(&format!(
                    " ORDER BY {}",
                    order_by.generate_query(pretty_print)
                ));
            }
        }

        query
    }

    pub fn values(&self) -> Arc<Vec<T>> {
        self.parameter_values.clone()
    }

    fn generate_indentation(&self, level: u32) -> String {
        "  ".repeat(level as usize)
    }
}

impl<T> NvSelect<T> {
    pub fn generate_tuples_holder<FieldTypes>(&self) -> Vec<FieldTypes> {
        Vec::new()
    }
}

// Implement additional methods required for `JoinDef`, `JoinStatement`, `FromTableStatement`, and `Condition` classes
impl<T> JoinDef<T> {
    
}

impl<T> JoinStatement<T> {
    
}

impl<T> Condition<T> {
    pub fn generate_query_from_subquery(&self, pretty_print: bool) -> String {
        match &self.subquery {
            Some(subquery) => subquery.generate_query(pretty_print),
            None => String::new(),
        }
    }
}


