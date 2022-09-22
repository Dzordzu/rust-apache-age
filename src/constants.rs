pub const CREATE_GRAPH: &str = "SELECT * FROM create_graph($1)";
pub const DROP_GRAPH: &str = "SELECT * FROM drop_graph($1, true)";
pub const CQ_NO_ARG: &str = "";
pub const CQ_ARG: &str = ", $1";
pub const LOAD_AGE: &str = "LOAD 'age'";
pub const SET_AGE: &str = "SET search_path = ag_catalog, \"$user\", public";
pub const GRAPH_EXISTS: &str = "SELECT COUNT(name) FROM ag_graph WHERE name = $1";

macro_rules! cypher_query {
    () => {
        "SELECT * FROM cypher('{}', $$ {} $${}) as (v agtype)"
    };
}

macro_rules! constraint {
    () => {
        "ALTER TABLE \"{}\".\"{}\" ADD CONSTRAINT \"{}\" CHECK({})"
    };
}

macro_rules! unique_index {
    () => {
        "CREATE UNIQUE INDEX \"{}\" ON \"{}\".\"{}\"(agtype_access_operator(properties, '\"{}\"'))"
    };
}

macro_rules! required_constraint {
    () => {
        "agtype_access_operator(properties, '\"{}\"') IS NOT NULL"
    };
}
