#![allow(dead_code)]

use std::fmt;
use std::sync::{Arc, RwLock};

// Define the Select struct
#[derive(Debug)]
pub struct Select {
    from: RwLock<Option<Arc<FromStatement>>>,
    where_: RwLock<Option<Arc<WhereStatement>>>,
    index: RwLock<u32>,
}

impl Select {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            from: RwLock::new(None),
            where_: RwLock::new(None),
            index: RwLock::new(1),
        })
    }

    pub fn from(self: Arc<Self>) -> Arc<FromStatement> {
        let mut from_guard = self.from.write().unwrap();
        if from_guard.is_none() {
            let from_statement = Arc::new(FromStatement::new(self.clone()));
            *from_guard = Some(from_statement.clone());
            from_statement
        } else {
            from_guard.as_ref().unwrap().clone()
        }
    }

    pub fn where_clause(self: Arc<Self>) -> Arc<WhereStatement> {
        let mut from_guard = self.where_.write().unwrap();
        if from_guard.is_none() {
            let where_statement = Arc::new(WhereStatement::new(self.clone()));
            *from_guard = Some(where_statement.clone());
            where_statement
        } else {
            from_guard.as_ref().unwrap().clone()
        }
    }

    pub fn index(&self) -> u32 {
        *self.index.read().unwrap()
    }

    pub fn increment_index(&self) {
        let mut index = self.index.write().unwrap();
        *index += 1;
    }

    pub fn generate_query(&self) -> String {
        format!("last parameter index: {}", self.index())
    }
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Select")
    }
}

// Define the FromStatement struct
#[derive(Debug)]
pub struct FromStatement {
    parent: Arc<Select>,
    table_name: RwLock<String>,
}

impl FromStatement {
    pub fn new(parent: Arc<Select>) -> Self {
        FromStatement {
            parent,
            table_name: RwLock::new(String::new()),
        }
    }

    pub fn add_table(self: Arc<Self>, table_name: String) -> Arc<Self> {
        *self.table_name.write().unwrap() = table_name.clone();
        self.parent.increment_index();
        println!("Add table: {}", table_name);
        self
    }

    pub fn end_from(self: Arc<Self>) -> Arc<Select> {
        self.parent.clone()
    }
}

impl fmt::Display for FromStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FromStatement")
    }
}

// Define the FromStatement struct
#[derive(Debug)]
pub struct WhereStatement {
    parent: Arc<Select>,
    table_name: RwLock<String>,
}

impl WhereStatement {
    pub fn new(parent: Arc<Select>) -> Self {
        WhereStatement {
            parent,
            table_name: RwLock::new(String::new()),
        }
    }

    pub fn add_condition(self: Arc<Self>, table_name: String) -> Arc<Self> {
        *self.table_name.write().unwrap() = table_name.clone();
        self.parent.increment_index();
        println!("Add condition: {}", table_name);
        self
    }

    pub fn end_where(self: Arc<Self>) -> Arc<Select> {
        self.parent.clone()
    }
}

impl fmt::Display for WhereStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WhereStatement")
    }
}
// Example usage
// fn main() {
//     let select = Select::new();
//     let select = select
//         .from()
//         .add_table("users".to_string())
//         .end_from();

//     println!("Select Index: {}", select.index());
//     select.finalize();
//     println!("Hello, world!");
// }
