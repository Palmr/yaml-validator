use crate::error::SchemaErrorKind;
use crate::Validate;
use std::convert::TryFrom;
use yaml_rust::{Yaml, YamlLoader};

fn load_simple(source: &'static str) -> Yaml {
    YamlLoader::load_from_str(source).unwrap().remove(0)
}

mod schemaobject {
    use super::*;
    use crate::SchemaObject;
    #[test]
    fn from_yaml() {
        SchemaObject::try_from(&load_simple(
            r#"
            items:
              - name: something
                type: string
        "#,
        ))
        .unwrap();
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaObject::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        assert_eq!(
            SchemaObject::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }
}

mod schemastring {
    use super::*;
    use crate::SchemaString;
    #[test]
    fn from_yaml() {
        SchemaString::try_from(&load_simple("type: string")).unwrap();
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaString::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        assert_eq!(
            SchemaString::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        assert_eq!(
            SchemaString::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_string() {
        let schema = SchemaString {};
        schema.validate(&load_simple("hello world")).unwrap();
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaString {};

        assert_eq!(
            schema.validate(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "string",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn validate_array() {
        let schema = SchemaString {};

        assert_eq!(
            schema.validate(&load_simple(r#"
                - abc
                - 123
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "string",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaString {};

        assert_eq!(
            schema.validate(&load_simple(r#"
                hello: world
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "string",
                actual: "hash"
            }
            .into()
        );
    }
}

mod schemainteger {
    use super::*;
    use crate::SchemaInteger;
    #[test]
    fn from_yaml() {
        SchemaInteger::try_from(&load_simple("type: string")).unwrap();
    }

    #[test]
    fn from_string() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn from_integer() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn from_array() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple(
                r#"
                - hello
                - world
            "#
            ))
            .unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_string() {
        let schema = SchemaInteger {};
        
        assert_eq!(
            schema.validate(&load_simple("hello world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn validate_integer() {
        let schema = SchemaInteger {};
        
        schema.validate(&load_simple("10")).unwrap();
    }

    #[test]
    fn validate_array() {
        let schema = SchemaInteger {};

        assert_eq!(
            schema.validate(&load_simple(r#"
                - abc
                - 123
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "array"
            }
            .into()
        );
    }

    #[test]
    fn validate_hash() {
        let schema = SchemaInteger {};

        assert_eq!(
            schema.validate(&load_simple(r#"
                hello: world
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "integer",
                actual: "hash"
            }
            .into()
        );
    }
}
