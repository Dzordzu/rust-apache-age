use apache_age::serializers::return_operation::to_string;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
enum Status {
    Active,
    Closed,
}

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    pub task_id: u64,
    pub name: String,
    pub status: Option<Status>,

    // You need to skip serialization when you need to skip some fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub comment: String,
}

#[test]
fn test_simple_return() {
    let common = Task {
        task_id: 0,
        name: String::new(),
        status: Some(Status::Closed),
        description: Some(String::new()),
        comment: String::new(),
    };

    let mut none_description = common.clone();
    none_description.description = None;

    let fields_all = ["name", "status", "description", "task_id"]
        .map(|x| x.to_string())
        .to_vec();

    let fields_task_id = vec!["task_id".to_string()];
    let fields_name_status = ["status", "name"].map(|x| x.to_string()).to_vec();
    let fields_name_strange = ["asd", "name"].map(|x| x.to_string()).to_vec();

    let results = [
        (
            &common,
            "var".to_string(),
            Some(fields_all.clone()),
            "task_id: var.task_id, name: var.name, status: var.status, description: var.description"

        ),
        (
            &common,
            "var2".to_string(),
            None,
            "task_id: var2.task_id, name: var2.name, status: var2.status, description: var2.description"

        ),
        (
            &none_description,
            "x".to_string(),
            Some(fields_all),
            "task_id: x.task_id, name: x.name, status: x.status"
        ),
        (
            &common,
            "dupa".to_string(),
            Some(fields_task_id),
            "task_id: dupa.task_id"
        ),
        (
            &common,
            "name".to_string(),
            Some(fields_name_status),
            "name: name.name, status: name.status"
        ),
        (
            &common,
            "name".to_string(),
            Some(fields_name_strange),
            "name: name.name"
        )
    ];

    results.iter().for_each(|x| {
        let r = to_string(x.0, x.1.clone(), x.2.clone()).unwrap();
        assert_eq!(r, x.3)
    });
}
