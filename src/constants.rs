pub const CREATE_GRAPH: &'static str = "SELECT * FROM create_graph($1)";
pub const DROP_GRAPH: &'static str = "SELECT * FROM drop_graph($1, true)";
pub const CQ_NO_ARG: &'static str = "";
pub const CQ_ARG: &'static str = ", $1";
pub const LOAD_AGE: &'static str = "LOAD 'age'";
pub const SET_AGE: &'static str = "SET search_path = ag_catalog, \"$user\", public";
pub const GRAPH_EXISTS: &'static str = "SELECT COUNT(name) FROM ag_graph WHERE name = $1";

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
