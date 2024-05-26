#![allow(dead_code)]

use crate::sqlbuilder::NvSelect;
use crate::sqlbuilder::SortType;

pub struct OrderByClause {
    field_name: String,
    table_alias: Option<String>,
    level: u32,
    sort_type: SortType,
    define_sort_type: bool,
}

impl OrderByClause {
    pub fn new(field_name: String, sort: SortType, define_sort_type: bool, level: u32) -> Self {
        Self {
            field_name,
            table_alias: None,
            sort_type: sort,
            define_sort_type,
            level,
        }
    }

    pub fn new_with_alias(
        field_name: String,
        alias: Option<String>,
        sort: SortType,
        define_sort_type: bool,
        level: u32,
    ) -> Self {
        Self {
            field_name,
            table_alias: alias,
            sort_type: sort,
            define_sort_type,
            level,
        }
    }

    pub fn build_fieldname(&self) -> String {
        match &self.table_alias {
            Some(alias) => format!("{}.{}", alias, self.field_name),
            None => self.field_name.clone(),
        }
    }

    pub fn generate_query(&self) -> String {
        let mut query = self.build_fieldname();
        if self.define_sort_type {
            query.push_str(match self.sort_type {
                SortType::Ascending => " ASC",
                SortType::Descending => " DESC",
            });
        }
        query
    }
}

pub struct OrderByStatement<T> {
    parent: Option<*mut NvSelect<T>>,
    sorts: Vec<OrderByClause>,
    level: u32,
}

impl<T> OrderByStatement<T> {
    pub fn new() -> Self {
        Self {
            parent: None,
            sorts: Vec::new(),
            level: 0,
        }
    }

    pub fn new_with_parent(parent: *mut NvSelect<T>, level: u32) -> Self {
        Self {
            parent: Some(parent),
            sorts: Vec::new(),
            level,
        }
    }

    pub fn asc(
        self,
        field_name: String,
        table_alias: Option<String>,
        define_sort_type: bool,
    ) -> Self {
        self.by(
            field_name,
            table_alias,
            SortType::Ascending,
            define_sort_type,
        )
    }

    pub fn desc(
        self,
        field_name: String,
        table_alias: Option<String>,
        define_sort_type: bool,
    ) -> Self {
        self.by(
            field_name,
            table_alias,
            SortType::Descending,
            define_sort_type,
        )
    }

    pub fn by(
        mut self,
        field_name: String,
        table_alias: Option<String>,
        sort_type: SortType,
        define_sort_type: bool,
    ) -> Self {
        let clause = OrderByClause::new_with_alias(
            field_name,
            table_alias,
            sort_type,
            define_sort_type,
            self.level,
        );
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

    pub fn end_order_by_block(&self) -> &NvSelect<T> {
        match self.parent {
            Some(parent) => unsafe { &*parent },
            None => panic!("null-reference to parent of NvSelect<T>"),
        }
    }
}
