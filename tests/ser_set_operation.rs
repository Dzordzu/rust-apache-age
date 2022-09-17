use apache_age::serializers::set_operation::to_string;
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
fn test_simple_set() {
    let common = Task {
        task_id: 0,
        name: String::new(),
        status: Some(Status::Closed),
        description: Some(String::new()),
        comment: String::new(),
    };

    let mut none_description = common.clone();
    none_description.description = None;

    let results = [
        (
            &common,
            "var".to_string(),
            None,
            None,
            "var.task_id = $task_id, var.name = $name, var.status = $status, var.description = $description"

        ),
        (
            &none_description,
            "x".to_string(),
            None,
            None,
            "x.task_id = $task_id, x.name = $name, x.status = $status"
        ),
        (
            &common,
            "dupa".to_string(),
            Some("kok".to_string()),
            None,
            "dupa.task_id = $kok.task_id, dupa.name = $kok.name, dupa.status = $kok.status, dupa.description = $kok.description"
        ),
        (
            &common,
            "com".to_string(),
            Some("xd".to_string()),
            Some(["name", "description"].iter().map(|x| x.to_string()).collect()),
            "com.name = $xd.name, com.description = $xd.description"
        ),
    ];

    results.iter().for_each(|x| {
        let r = to_string(x.0, x.1.clone(), x.2.clone(), x.3.clone()).unwrap();
        assert_eq!(r, x.4)
    });
}
