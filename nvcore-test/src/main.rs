use nvcore::sqlbuilder::Select;
fn main() {
    let select = Select::new();
    let select = select
        .from()
        .add_table("users".to_string())
        .add_table("company".to_string())
        .end_from()
        .where_clause()
        .add_condition("cond1".to_string())
        .end_where();

    println!("Select Index: {}", select.index());    
    println!("Query: {}", select.generate_query());
}
