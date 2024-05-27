#![allow(dead_code)]

use crate::sqlbuilder::{DatabaseDialect, SqlAggregateFunction};
use std::sync::Arc ;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FieldDefMode {
    FieldRaw,
    FieldWType,
    FnStaticParameter,
    FnParameterizedValues,
}

impl std::fmt::Display for FieldDefMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldDefMode::FieldRaw => write!(f, "Field"),
            FieldDefMode::FieldWType => write!(f, "Field [Strong-Typed]"),
            FieldDefMode::FnStaticParameter => write!(f, "Fn Static"),
            FieldDefMode::FnParameterizedValues => write!(f, "Fn Parameterized"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldDef<T> {
    field: String,
    table_alias: Option<String>,
    static_param_values: Arc<Vec<String>>,
    parameter_values: Arc<Vec<T>>,
    fn_values: Arc<Vec<T>>,
    function_name: String,
    parameter_format: String,
    enclose_field_name: bool,
    aggregate_fn: SqlAggregateFunction,
    field_alias: Option<String>,
    start_parameter_index: u32,
    current_parameter_index: u32,
    level: u32,
    mode: FieldDefMode,
    dialect: DatabaseDialect,
}

impl<T> FieldDef<T> {
    pub fn new_field_def(
        dialect: DatabaseDialect,
        field: String,
        table_alias: Option<String>,
        enclose_field_name: bool,
        aggregate_fn: SqlAggregateFunction,
        field_alias: Option<String>,
        level: u32,
        mode: FieldDefMode,
    ) -> Self {
        Self {
            field,
            table_alias,
            static_param_values: Arc::new(Vec::new()),
            parameter_values: Arc::new(Vec::new()),
            fn_values: Arc::new(Vec::new()),
            function_name: String::new(),
            parameter_format: String::new(),
            enclose_field_name,
            aggregate_fn,
            field_alias,
            start_parameter_index: 0,
            current_parameter_index: 0,
            level,
            mode,
            dialect,
        }
    }

    pub fn new_static_function(
        dialect: DatabaseDialect,
        function_name: String,
        static_param_values: Arc<Vec<String>>,
        level: u32,
        alias: Option<String>,
    ) -> Self {
        Self {
            field: String::new(),
            table_alias: None,
            static_param_values,
            parameter_values: Arc::new(Vec::new()),
            fn_values: Arc::new(Vec::new()),
            function_name,
            parameter_format: String::new(),
            enclose_field_name: false,
            aggregate_fn: SqlAggregateFunction::None,
            field_alias: alias,
            start_parameter_index: 0,
            current_parameter_index: 0,
            level,
            mode: FieldDefMode::FnStaticParameter,
            dialect,
        }
    }

    pub fn new_dynamic_function(
        dialect: DatabaseDialect,
        function_name: String,
        parameter_format: String,
        parameter_values: Arc<Vec<T>>,
        fn_param_values: Arc<Vec<T>>,
        static_param_values: Arc<Vec<String>>,
        param_index: u32,
        level: u32,
        alias: Option<String>,
    ) -> Self {
        let current_parameter_index = Self::process_function_parameter_index(
            param_index,
            &parameter_format,
            &fn_param_values,
        );
        Self {
            field: String::new(),
            table_alias: None,
            static_param_values,
            parameter_values,
            fn_values: fn_param_values,
            function_name,
            parameter_format,
            enclose_field_name: false,
            aggregate_fn: SqlAggregateFunction::None,
            field_alias: alias,
            start_parameter_index: param_index,
            current_parameter_index,
            level,
            mode: FieldDefMode::FnParameterizedValues,
            dialect,
        }
    }

    fn process_function_parameter_index(
        current_param_index: u32,
        parameter_format: &str,
        parameter_values: &[T],
    ) -> u32 {
        if parameter_format.is_empty() {
            return current_param_index;
        }

        let mut index_params = 0;
        let mut param_index = current_param_index;
        let size_params = parameter_values.len();

        for ch in parameter_format.chars() {
            if index_params < size_params && ch == 'v' {
                param_index += 1;
                index_params += 1;
            }
        }

        param_index
    }

    fn build_field(&self) -> String {
        let mut oss = String::new();

        if self.aggregate_fn == SqlAggregateFunction::Distinct {
            oss.push_str(&self.aggregate_function_to_string(self.aggregate_fn));
            oss.push(' ');
        } else if self.aggregate_fn != SqlAggregateFunction::None {
            oss.push_str(&self.aggregate_function_to_string(self.aggregate_fn));
            oss.push('(');
        }

        if let Some(ref alias) = self.table_alias {
            oss.push_str(alias);
            oss.push('.');
        }

        oss.push_str(&self.field);

        if self.aggregate_fn == SqlAggregateFunction::Distinct {
            oss.push_str("");
        } else if self.aggregate_fn != SqlAggregateFunction::None {
            oss.push(')');
        }

        if let Some(ref alias) = self.field_alias {
            oss.push_str(" AS ");
            oss.push_str(alias);
        }

        oss
    }

    fn build_function_with_dynamic_parameters(&self) -> String {
        let mut param_index = self.start_parameter_index;
        let mut index_params = 0;
        let mut index_statics = 0;
        let size_params = self.fn_values.len();
        let size_statics = self.static_param_values.len();
        let mut is_first_element = true;
        let mut fn_call = String::new();

        fn_call.push_str(&self.function_name);
        fn_call.push('(');

        for ch in self.parameter_format.chars() {
            if (index_statics < size_statics || index_params < size_params)
                && (ch == 's' || ch == 'v')
            {
                if !is_first_element {
                    fn_call.push_str(", ");
                }
            }

            if ch == 's' && index_statics < size_statics {
                fn_call.push_str(&self.static_param_values[index_statics]);
                index_statics += 1;
                is_first_element = false;
            } else if ch == 'v' && index_params < size_params {
                fn_call.push_str(&self.determine_parameter_format(param_index));
                param_index += 1;
                index_params += 1;
                is_first_element = false;
            }
        }

        fn_call.push(')');
        if let Some(ref alias) = self.field_alias {
            fn_call.push_str(" AS ");
            fn_call.push_str(alias);
        }

        fn_call
    }

    fn build_function_with_static_parameters(&self) -> String {
        let mut fn_call = String::new();
        fn_call.push_str(&self.function_name);
        fn_call.push('(');
        for (i, param) in self.static_param_values.iter().enumerate() {
            if i > 0 {
                fn_call.push_str(", ");
            }
            fn_call.push_str(param);
        }
        fn_call.push(')');
        if let Some(ref alias) = self.field_alias {
            fn_call.push_str(" AS ");
            fn_call.push_str(alias);
        }
        fn_call
    }

    pub fn mode(&self) -> FieldDefMode {
        self.mode
    }

    pub fn get_current_parameter_index(&self) -> u32 {
        self.current_parameter_index
    }

    pub fn field(&self) -> &String {
        &self.field
    }

    pub fn table_alias(&self) -> &Option<String> {
        &self.table_alias
    }

    pub fn field_alias(&self) -> &Option<String> {
        &self.field_alias
    }

    pub fn enclose_field_name(&self) -> bool {
        self.enclose_field_name
    }

    pub fn aggregate_function(&self) -> SqlAggregateFunction {
        self.aggregate_fn
    }

    pub fn function_name(&self) -> &String {
        &self.function_name
    }

    pub fn static_parameter_values(&self) -> Arc<Vec<String>> {
        self.static_param_values.clone()
    }

    pub fn values(&self) -> Arc<Vec<T>> {
        self.parameter_values.clone()
    }

    pub fn generate_query(&self) -> String {
        match self.mode {
            FieldDefMode::FieldRaw => self.build_field(),
            FieldDefMode::FieldWType => self.build_field(),
            FieldDefMode::FnStaticParameter => self.build_function_with_static_parameters(),
            FieldDefMode::FnParameterizedValues => self.build_function_with_dynamic_parameters(),
        }
    }

    fn aggregate_function_to_string(&self, fn_type: SqlAggregateFunction) -> String {
        match fn_type {
            SqlAggregateFunction::Distinct => "DISTINCT".to_string(),
            SqlAggregateFunction::Count => "COUNT".to_string(),
            SqlAggregateFunction::Avg => "AVG".to_string(),
            SqlAggregateFunction::Sum => "SUM".to_string(),
            SqlAggregateFunction::ToUpper => "TO_UPPER".to_string(),
            SqlAggregateFunction::ToLower => "TO_LOWER".to_string(),
            _ => "".to_string(),
        }
    }

    fn determine_parameter_format(&self, param_index: u32) -> String {
        // Placeholder for actual parameter format logic
        format!("param{}", param_index)
    }
}
