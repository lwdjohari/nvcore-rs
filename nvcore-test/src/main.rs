use nvcore::sqlbuilder::{
    DatabaseDialect, DefaultPostgresParamType, NvSelect,
    SqlAggregateFunction, SqlOperator,
};
fn main() {
    let select = NvSelect::<DefaultPostgresParamType>::new(DatabaseDialect::PostgreSQL)
        .f(
            "field1".to_string(),
            Some("t".to_string()),
            Some("a".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
            "field2".to_string(),
            Some("t".to_string()),
            Some("a".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
            "field3".to_string(),
            Some("t".to_string()),
            Some("a".to_string()),
            SqlAggregateFunction::None,
            false,
        )
        .f(
            "field4".to_string(),
            Some("t".to_string()),
            Some("a".to_string()),
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
              DefaultPostgresParamType::Int(1)
          )
        .end_where_block();

    println!("SQL QUERY:\n {}", select.generate_query(true));
    // println!("Query: {}", select.generate_query());
}
