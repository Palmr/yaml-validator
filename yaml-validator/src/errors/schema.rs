#![macro_use]

use thiserror::Error;

use super::GenericError;
use crate::breadcrumb::{Breadcrumb, BreadcrumbSegment, BreadcrumbSegmentVec};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SchemaErrorKind<'a> {
    #[error("wrong type, expected {expected} got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'a str,
    },
    #[error("malformed field: {error}")]
    MalformedField { error: String },
    #[error("field '{field}' missing")]
    FieldMissing { field: &'a str },
    #[error("field '{field}' is not specified in the schema")]
    ExtraField { field: &'a str },
    #[error("unknown type specified: {unknown_type}")]
    UnknownType { unknown_type: &'a str },
    #[error("multiple errors were encountered: {errors:?}")]
    Multiple { errors: Vec<SchemaError<'a>> },
}

/// A wrapper type around `SchemaErrorKind` containing path information about where the error occurred.
#[derive(Debug, PartialEq, Eq)]
pub struct SchemaError<'schema> {
    pub kind: SchemaErrorKind<'schema>,
    pub state: Breadcrumb<'schema>,
}

impl<'a> SchemaError<'a> {
    fn flatten<A: AsRef<str>>(
        &self,
        fmt: &mut std::fmt::Formatter<'_>,
        root: A,
    ) -> std::fmt::Result {
        match &self.kind {
            SchemaErrorKind::Multiple { errors } => {
                for err in errors {
                    err.flatten(fmt, format!("{}{}", root.as_ref(), self.state))?;
                }
            }
            err => writeln!(fmt, "{}{}: {}", root.as_ref(), self.state, err)?,
        }

        Ok(())
    }

    pub fn add_path_name(path: &'a str) -> impl Fn(SchemaError<'a>) -> SchemaError<'a> {
        move |mut err: SchemaError<'a>| -> SchemaError<'a> {
            err.state.push(BreadcrumbSegment::Name(path));
            err
        }
    }

    pub fn add_path_index(index: usize) -> impl Fn(SchemaError<'a>) -> SchemaError<'a> {
        move |mut err: SchemaError<'a>| -> SchemaError<'a> {
            err.state.push(BreadcrumbSegment::Index(index));
            err
        }
    }
}

impl<'a> std::fmt::Display for SchemaError<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.flatten(fmt, "#")
    }
}

impl<'a> SchemaErrorKind<'a> {
    #[must_use]
    pub fn with_path(self, path: BreadcrumbSegmentVec<'a>) -> SchemaError<'a> {
        SchemaError {
            kind: self,
            state: Breadcrumb::new(path),
        }
    }

    #[must_use]
    pub fn with_path_name(self, path: &'a str) -> SchemaError<'a> {
        let mut err: SchemaError = self.into();
        err.state.push(BreadcrumbSegment::Name(path));
        err
    }

    #[must_use]
    pub fn with_path_index(self, index: usize) -> SchemaError<'a> {
        let mut err: SchemaError = self.into();
        err.state.push(BreadcrumbSegment::Index(index));
        err
    }
}

impl<'a> From<SchemaErrorKind<'a>> for SchemaError<'a> {
    fn from(kind: SchemaErrorKind<'a>) -> SchemaError<'a> {
        SchemaError {
            kind,
            state: Breadcrumb::default(),
        }
    }
}

impl<'a> From<Vec<SchemaError<'a>>> for SchemaError<'a> {
    fn from(errors: Vec<SchemaError<'a>>) -> Self {
        SchemaErrorKind::Multiple { errors }.into()
    }
}

impl<'a> From<GenericError<'a>> for SchemaErrorKind<'a> {
    fn from(e: GenericError<'a>) -> Self {
        match e {
            GenericError::WrongType { expected, actual } => {
                SchemaErrorKind::WrongType { expected, actual }
            }
            GenericError::FieldMissing { field } => SchemaErrorKind::FieldMissing { field },
            GenericError::ExtraField { field } => SchemaErrorKind::ExtraField { field },
            GenericError::Multiple { errors } => SchemaErrorKind::Multiple {
                errors: errors
                    .into_iter()
                    .map(SchemaErrorKind::from)
                    .map(SchemaError::from)
                    .collect(),
            },
        }
    }
}

impl<'a> From<GenericError<'a>> for SchemaError<'a> {
    fn from(e: GenericError<'a>) -> Self {
        SchemaErrorKind::from(e).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::SchemaObject;
    use crate::utils::load_simple;
    use crate::{Context, Validate};
    use std::convert::TryFrom;
    #[test]
    fn test_error_path() {
        let yaml = load_simple(
            r#"
            items:
              test:
                type: integer
              something:
                type: object
                items:
                  level2:
                    type: object
                    items:
                      leaf: 
                        notype: hello
            "#,
        );

        let err = SchemaObject::try_from(&yaml).unwrap_err();

        assert_eq!(
            format!("{err}"),
            "#.items.something.items.level2.items.leaf: field \'type\' missing\n",
        );
    }

    #[test]
    fn test_error_path_validation() {
        let yaml = load_simple(
            r#"
            items:
              test:
                type: integer
              something:
                type: object
                items:
                  level2:
                    type: array
                    items:
                      type: object
                      items:
                        num:
                          type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();
        let document = load_simple(
            r#"
            test: 20
            something:
              level2:
                - num: abc
                - num:
                    hash: value
                - num:
                    - array: hello
                - num: 10
                - num: jkl
            "#,
        );
        let ctx = Context::default();
        let err = schema.validate(&ctx, &document).unwrap_err();

        assert_eq!(
            format!("{err}"),
            r#"#.something.level2[0].num: wrong type, expected integer got string
#.something.level2[1].num: wrong type, expected integer got hash
#.something.level2[2].num: wrong type, expected integer got array
#.something.level2[4].num: wrong type, expected integer got string
"#
        );
    }
}
