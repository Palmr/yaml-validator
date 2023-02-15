use yaml_validator_cli::{actual_main, Error, Opt};

#[test]
fn test_all_types_example() {
    actual_main(&Opt {
        schemas: vec!["../examples/all-types/schema.yaml".into()],
        files: vec!["../examples/all-types/customers.yaml".into()],
        uri: "customer-list".into(),
    })
    .unwrap();
}

#[test]
fn test_multiple_schemas_example() {
    actual_main(&Opt {
        schemas: vec![
            "../examples/multiple-schemas/person-schema.yaml".into(),
            "../examples/multiple-schemas/phonebook-schema.yaml".into(),
        ],
        files: vec!["../examples/multiple-schemas/mybook.yaml".into()],
        uri: "phonebook".into(),
    })
    .unwrap();
}

#[test]
fn test_nesting_example() {
    actual_main(&Opt {
        schemas: vec!["../examples/nesting/schema.yaml".into()],
        files: vec!["../examples/nesting/mybook.yaml".into()],
        uri: "phonebook".into(),
    })
    .unwrap();
}

#[test]
fn test_locating_errors_example() {
    assert_eq!(
        actual_main(&Opt {
            schemas: vec!["../examples/locating-errors/schema.yaml".into()],
            files: vec!["../examples/locating-errors/phonebook.yaml".into()],
            uri: "phonebook".into(),
        })
        .unwrap_err(),
        Error::Validation(
            "../examples/locating-errors/phonebook.yaml:
#[1].age: wrong type, expected integer got real
#[2].age: wrong type, expected integer got string
#[2].name: wrong type, expected string got integer
"
            .into()
        )
    );
}

#[test]
fn test_branching_examples() {
    assert_eq!(
        actual_main(&Opt {
            schemas: vec!["../examples/branching/schema.yaml".into()],
            files: vec!["../examples/branching/usernames.yaml".into()],
            uri: "user-list".into(),
        })
            .unwrap_err(),
        Error::Validation(
            "../examples/branching/usernames.yaml:
#[2].password: special requirements for field not met: supplied value does not match regex pattern for field
"
                .into()
        )
    );
}

#[test]
fn test_non_existent_schema_file() {
    assert_eq!(
        actual_main(&Opt {
            schemas: vec!["not_found.yaml".into()],
            files: vec!["".into()],
            uri: String::new(),
        })
        .unwrap_err(),
        Error::Multiple(vec![Error::File(
            "could not read file not_found.yaml: No such file or directory (os error 2)\n".into()
        )])
    );
}

#[test]
fn test_non_existent_file() {
    assert_eq!(
        actual_main(&Opt {
            schemas: vec!["../examples/nesting/schema.yaml".into()],
            files: vec!["not_found.yaml".into()],
            uri: "person".into(),
        })
        .unwrap_err(),
        Error::Multiple(vec![Error::File(
            "could not read file not_found.yaml: No such file or directory (os error 2)\n".into()
        )])
    );
}

#[test]
fn test_unknown_schema_uri() {
    assert_eq!(
        actual_main(&Opt {
            schemas: vec!["../examples/nesting/schema.yaml".into()],
            files: vec!["../examples/nesting/mybook.yaml".into()],
            uri: "not-found".into(),
        })
        .unwrap_err(),
        Error::Validation("schema referenced by uri `not-found` not found in context\n".into())
    );
}
