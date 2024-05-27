use std::time::SystemTime;

use nvcore::sqlbuilder::{
    DatabaseDialect, DefaultPostgresParamType, NvSelect, SqlAggregateFunction, SqlOperator,
};
fn main() {
    let mut select 
      = NvSelect::<DefaultPostgresParamType>::new(DatabaseDialect::PostgreSQL);

    select = select
        .f(
            "field1".to_string(),
            Some("t".to_string()),
            Some("f1".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
            "field2".to_string(),
            Some("t".to_string()),
            Some("f2".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
            "field3".to_string(),
            Some("t".to_string()),
            Some("f3".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
            "field4".to_string(),
            Some("t".to_string()),
            Some("f4".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
          "field5".to_string(),
          Some("t".to_string()),
          Some("f5".to_string()),
          SqlAggregateFunction::None,
          false,
      )
        .from()
        .add_table_with_alias(&"table".to_string(), &Some("a".to_string()))
        .end_from_table_block()
        .where_clause()
        .add_condition(
            &"a.field1".to_string(),
            &SqlOperator::Equal,
            DefaultPostgresParamType::Int(1),
        )
        .or()
        .add_condition(
            &"a.field2".to_string(),
            &SqlOperator::Equal,
            DefaultPostgresParamType::String("Hello world".to_string()),
        )
        .or()
        .add_condition(
            &"a.field3".to_string(),
            &SqlOperator::Equal,
            DefaultPostgresParamType::TimePoint(SystemTime::now()),
        )
        .or()
        .add_condition(
            &"a.field4".to_string(),
            &SqlOperator::Equal,
            DefaultPostgresParamType::BigInt(192738124),
        )
        .or()
        .add_condition(
            &"a.field5".to_string(),
            &SqlOperator::Equal,
            DefaultPostgresParamType::Float(0.5),
        )
        .end_where_block();

    println!("SQL QUERY:\n {}\n", select.generate_query(true));

    println!("PARAMETER VALUES: {}", "");

    let pv = select.values();
    {
        let values = pv.read().unwrap();
        for v in values.iter() {
            println!("{}", v);
        }
    }

    // println!("Query: {}", select.generate_query());
}
