#![allow(dead_code)]

use crate::sqlbuilder::{
    DatabaseDialect, FieldDef, FieldDefMode, FromTableStatement, SqlAggregateFunction,
    WhereStatement,
};

use crate::utils::indent_space;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};

pub struct NvSelect<T> {
    current_param_index: RwLock<u32>,
    // join_blocks: RwLock<Arc<Vec<JoinStatement<T>>>>,
    from_table: RwLock<Option<Arc<FromTableStatement<T>>>>,
    fields: RwLock<Vec<FieldDef<T>>>,
    parameter_values: RwLock<Arc<Vec<T>>>,
    subquery_from_parent: RwLock<Option<Arc<FromTableStatement<T>>>>,
    where_: RwLock<Option<Arc<WhereStatement<T>>>>,
    // order_by: RwLock<Option<Arc<OrderByStatement<T>>>>,
    // group_by: RwLock<Option<Arc<GroupByStatement<T>>>>,
    subquery_where_parent: RwLock<Option<Arc<WhereStatement<T>>>>,
    // limit_offset: Option<Arc<LimitOffsetStatement<T>>>,
    table_alias: String,
    level: u32,
    dialect: DatabaseDialect,
}

impl<T> NvSelect<T> {
    pub fn new(dialect: DatabaseDialect) -> Arc<Self> {
        Arc::new(Self {
            current_param_index: 1.into(),
            level: 0,
            // join_blocks: Arc::new(Vec::new()).into(),
            from_table: None.into(),
            fields: Vec::new().into(),
            parameter_values: Arc::new(Vec::new()).into(),
            subquery_from_parent: None.into(),
            table_alias: String::new(),
            where_: None.into(),
            // order_by: None.into(),
            // group_by: None.into(),
            subquery_where_parent: None.into(),
            // limit_offset: None,
            dialect: dialect,
        })
    }

    pub fn with_param_index(current_param_index: u32, dialect: DatabaseDialect) -> Arc<Self> {
        Arc::new(Self {
            current_param_index: current_param_index.into(),
            level: 0,
            // join_blocks: Arc::new(Vec::new()).into(),
            from_table: None.into(),
            fields: Vec::new().into(),
            parameter_values: Arc::new(Vec::new()).into(),
            subquery_from_parent: None.into(),
            table_alias: String::new().into(),
            where_: None.into(),
            // order_by: None.into(),
            // group_by: None.into(),
            subquery_where_parent: None.into(),
            // limit_offset: None,
            dialect: dialect,
        })
    }

    pub fn new_subquery(
        values: Arc<Vec<T>>,
        current_param_index: u32,
        level: u32,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            current_param_index: current_param_index.into(),
            level,
            // join_blocks: Arc::new(Vec::new()).into(),
            from_table: None.into(),
            fields: Vec::new().into(),
            parameter_values: values.into(),
            subquery_from_parent: None.into(),
            table_alias: String::new(),
            where_: None.into(),
            // order_by: None.into(),
            // group_by: None.into(),
            subquery_where_parent: None.into(),
            // limit_offset: None,
            dialect: dialect,
        })
    }

    pub fn new_subquery_from(
        values: Arc<Vec<T>>,
        current_param_index: u32,
        level: u32,
        from_obj: Arc<FromTableStatement<T>>,
        table_alias: String,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            current_param_index: current_param_index.into(),
            level,
            // join_blocks: Arc::new(Vec::new()).into(),
            from_table: None.into(),
            fields: Vec::new().into(),
            parameter_values: values.into(),
            subquery_from_parent: Some(from_obj).into(),
            table_alias,
            where_: None.into(),
            // order_by: None.into(),
            // group_by: None.into(),
            subquery_where_parent: None.into(),
            // limit_offset: None,
            dialect: dialect,
        })
    }

    pub fn new_subquery_where(
        values: Arc<Vec<T>>,
        where_obj: Arc<WhereStatement<T>>,
        current_param_index: u32,
        level: u32,
        table_alias: String,
        dialect: DatabaseDialect,
    ) -> Arc<Self> {
        Arc::new(Self {
            current_param_index: current_param_index.into(),
            level,
            // join_blocks: Arc::new(Vec::new()).into(),
            from_table: None.into(),
            fields: Vec::new().into(),
            parameter_values: values.into(),
            subquery_from_parent: None.into(),
            table_alias,
            where_: None.into(),
            // order_by: None.into(),
            // group_by: None.into(),
            subquery_where_parent: Some(where_obj).into(),
            // limit_offset: None,
            dialect: dialect,
        })
    }

    pub fn dialect(&self) -> DatabaseDialect {
        self.dialect
    }

    pub fn get_current_param_index(&self) -> u32 {
        *self.current_param_index.read().unwrap()
    }

    pub fn update_current_param_index(self: Arc<Self>, current_param_index: u32) {
        let mut write_guard = self.current_param_index.write().unwrap();
        *write_guard = current_param_index;
    }

    pub fn table_alias(&self) -> &str {
        &self.table_alias
    }

    pub fn get_block_level(&self) -> u32 {
        self.level
    }

    pub fn field(
        self: Arc<Self>,
        field: String,
        table_alias: Option<String>,
        field_alias: Option<String>,
        aggregate_fn: SqlAggregateFunction,
        enclose_field_name: bool,
    ) -> Arc<Self> {
        let mut field_guard = self.fields.write().unwrap();

        let d = self.dialect.clone();
        let l = self.level.clone();

        // let field_mut = Arc::get_mut(&mut *field_guard)
        //     .expect("There should be no other references to the Arc at this point");

        field_guard.push(FieldDef::new_field_def(
            d,
            field,
            table_alias,
            enclose_field_name,
            aggregate_fn,
            field_alias,
            l,
            FieldDefMode::FieldWType,
        ));

        self.clone()
    }

    pub fn f(
        self: Arc<Self>,
        field: String,
        table_alias: Option<String>,
        field_alias: Option<String>,
        aggregate_fn: SqlAggregateFunction,
        enclose_field_name: bool,
    ) -> Arc<Self> {
        let mut field_guard = self.fields.write().unwrap();

        field_guard.push(FieldDef::new_field_def(
            self.dialect,
            field,
            table_alias,
            enclose_field_name,
            aggregate_fn,
            field_alias,
            self.level,
            FieldDefMode::FieldRaw,
        ));

        self.clone()
    }

    pub fn end_subquery_inside_from(self: Arc<Self>) -> Arc<FromTableStatement<T>> {
        let from_parent = self.subquery_from_parent.read().unwrap().is_some();
        if !from_parent {
            panic!("Call this only from .From().AddSubquery().EndFromSubquery()")
        }

        let mut from_parent_rw = self.subquery_from_parent.write().unwrap().unwrap();
        from_parent_rw.update_current_param_index(*self.current_param_index.read().unwrap());
        from_parent_rw.clone()
    }

    pub fn end_subquery_inside_where_condition(self: Arc<Self>) -> Arc<WhereStatement<T>> {
        let where_parent = self.subquery_where_parent.read().unwrap().is_some();
        if !where_parent {
            panic!("Call this only from .From().AddSubquery().EndFromSubquery()")
        }

        let mut where_parent_rw = self.subquery_where_parent.write().unwrap().unwrap();
        // where_parent_rw.update_current_param_index(*self.current_param_index.read().unwrap());
        where_parent_rw.clone()
    }

    pub fn from(self: Arc<Self>) -> Arc<FromTableStatement<T>> {
        let is_from_parent = self.from_table.read().unwrap().is_some();

        if !is_from_parent {
            *self.from_table.write().unwrap() = Some(FromTableStatement::new(
                self.parameter_values.read().unwrap().clone(),
                self,
                self.current_param_index.read().unwrap().clone(),
                self.level,
                self.dialect,
            ))
            .into();
        }

        self.from_table.read().unwrap().unwrap().clone()
    }

    pub fn where_(self: Arc<Self>) -> Arc<WhereStatement<T>> {
        let is_where_parent = self.where_.read().unwrap().is_some();

        if is_where_parent {
            *self.where_.write().unwrap() = Some(WhereStatement::new_with_parent(
                self.parameter_values.read().unwrap().clone(),
                self,
                self.current_param_index.read().unwrap().clone(),
                self.level,
                self.dialect,
            ));
        }

        self.where_.read().unwrap().unwrap()
    }

    // pub fn join(&mut self) -> &mut JoinStatement<T> {
    //     self.join_blocks.push(JoinStatement::new(
    //         self as *mut _,
    //         self.current_param_index,
    //         self.level,
    //         self.dialect,
    //     ));
    //     self.join_blocks.last_mut().unwrap()
    // }

    // pub fn order_by(&mut self) -> &mut OrderByStatement<T> {
    //     if self.order_by.is_none() {
    //         self.order_by = Some(Arc::new(OrderByStatement::new_with_parent(
    //             self as *mut _,
    //             self.level,
    //         )));
    //     }
    //     Arc::get_mut(self.order_by.as_mut().unwrap()).unwrap()
    // }

    // pub fn group_by(&mut self) -> &mut GroupByStatement<T> {
    //     if self.group_by.is_none() {
    //         self.group_by = Some(Arc::new(GroupByStatement::new_with_parent(
    //             self as *mut _,
    //             self.current_param_index,
    //             self.level,
    //         )));
    //     }
    //     Arc::get_mut(self.group_by.as_mut().unwrap()).unwrap()
    // }

    pub fn fn_static(
        self: Arc<Self>,
        fn_name: String,
        param_values: Arc<Vec<String>>,
        field_alias: Option<String>,
    ) -> Arc<Self> {
        self.fields
            .write()
            .unwrap()
            .push(FieldDef::new_static_function(
                self.dialect,
                fn_name,
                param_values,
                self.level,
                field_alias,
            ));
        self
    }

    pub fn fn_dynamic(
        self: Arc<Self>,
        fn_name: String,
        parameter_list_format: String,
        param_values: Arc<Vec<T>>,
        static_param_values: Arc<Vec<String>>,
        field_alias: Option<String>,
    ) -> Arc<Self> {
        self.fields
            .write()
            .unwrap()
            .push(FieldDef::new_dynamic_function(
                self.dialect,
                fn_name,
                parameter_list_format,
                self.parameter_values.read().unwrap().clone(),
                param_values,
                static_param_values,
                self.current_param_index.read().unwrap().clone(),
                self.level,
                field_alias,
            ));

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

        let fields_guard = self.fields.read().unwrap();
        let from_table_guard = self.from_table.read().unwrap();
        let where_statement_guard = self.where_.read().unwrap();

        // SELECT
        if pretty_print {
            query.push_str(&indent_space(self.level));
            query.push_str("SELECT \n");
        } else {
            query.push_str("SELECT ");
        }

        let mut first_element = true;
        for field in fields_guard.iter() {
            if !first_element {
                query.push_str(if pretty_print { ",\n" } else { ", " });
            }
            if pretty_print {
                query.push_str(&indent_space(self.level + 1));
            }
            query.push_str(&field.generate_query());
            first_element = false;
        }

        // FROM
        if let Some(from_table) = from_table_guard.as_ref() {
            if !from_table.is_empty() {
                if pretty_print {
                    query.push_str(&format!(
                        "\n{}FROM \n{}",
                        indent_space(self.level),
                        from_table.generate_query(pretty_print)
                    ));
                } else {
                    query.push_str(&format!(
                        " FROM {}",
                        from_table.generate_query(pretty_print)
                    ));
                }
            }
        }

        // // JOIN
        // if !self.join_blocks.is_empty() {
        //     query.push_str(if pretty_print { "\n" } else { " " });
        //     for join_block in &self.join_blocks {
        //         query.push_str(&join_block.generate_query(pretty_print));
        //     }
        // }

        // WHERE
        if let Some(where_) = where_statement_guard.as_ref() {
            if pretty_print {
                query.push_str(&format!(
                    "\n{}WHERE\n{}{}",
                    indent_space(self.level),
                    indent_space(self.level + 1),
                    where_.generate_query(pretty_print, false)
                ));
            } else {
                query.push_str(&format!(
                    " WHERE {}",
                    where_.generate_query(pretty_print, false)
                ));
            }
        }

        // // GROUP BY
        // if let Some(ref group_by) = self.group_by {
        //     if pretty_print {
        //         query.push_str(&format!(
        //             "\n{}GROUP BY\n{}{}",
        //             indent_space(self.level),
        //             indent_space(self.level + 1),
        //             group_by.generate_query(pretty_print)
        //         ));
        //     } else {
        //         query.push_str(&format!(
        //             " GROUP BY {}",
        //             group_by.generate_query(pretty_print)
        //         ));
        //     }
        // }

        // // ORDER BY
        // if let Some(ref order_by) = self.order_by {
        //     if pretty_print {
        //         query.push_str(&format!(
        //             "\n{}ORDER BY\n{}{}",
        //             indent_space(self.level),
        //             indent_space(self.level + 1),
        //             order_by.generate_query(pretty_print)
        //         ));
        //     } else {
        //         query.push_str(&format!(
        //             " ORDER BY {}",
        //             order_by.generate_query(pretty_print)
        //         ));
        //     }
        // }

        query
    }

    pub fn values(&self) -> Arc<Vec<T>> {
        self.parameter_values.read().unwrap().clone()
    }
}

// impl<T> NvSelect<T> {
//     pub fn generate_tuples_holder<FieldTypes>(&self) -> Vec<FieldTypes> {
//         Vec::new()
//     }
// }

// // Implement additional methods required for `JoinDef`, `JoinStatement`, `FromTableStatement`, and `Condition` classes
// impl<T> JoinDef<T> {}

// impl<T> JoinStatement<T> {}

// impl<T> Condition<T> {

// }
