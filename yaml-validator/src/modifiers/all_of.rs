use crate::errors::ValidationError;
use crate::errors::{SchemaError, SchemaErrorKind};
use crate::utils::{CondenseErrors, YamlUtils};
use crate::{Context, PropertyType, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug)]
pub(crate) struct SchemaAllOf<'schema> {
    items: Vec<PropertyType<'schema>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaAllOf<'schema> {
    type Error = SchemaError<'schema>;

    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["allOf"], &[])?;
        let items = SchemaError::condense_errors(
            &mut yaml
                .lookup("allOf", "array", Yaml::as_vec)?
                .iter()
                .map(|property| {
                    PropertyType::try_from(property).map_err(SchemaError::add_path_name("items"))
                }),
        )?;

        if items.is_empty() {
            return Err(SchemaErrorKind::MalformedField {
                error: "allOf modifier requires an array of schemas to validate against".to_owned(),
            }
            .with_path_name("allOf"));
        }

        Ok(SchemaAllOf { items })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaAllOf<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        ValidationError::condense_errors(
            &mut self
                .items
                .iter()
                .enumerate()
                .map(|(_, schema)| schema.validate(ctx, yaml))
                .filter(Result::is_err),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{errors::ValidationErrorKind, utils::load_simple};

    #[test]
    fn one_of_from_yaml() {
        SchemaAllOf::try_from(&load_simple(
            r#"
            allOf:
              - type: integer
              - type: string
        "#,
        ))
        .unwrap();

        assert_eq!(
            SchemaAllOf::try_from(&load_simple(
                r#"
                allOff:
                  - type: integer
            "#,
            ))
            .unwrap_err(),
            SchemaErrorKind::Multiple {
                errors: vec![
                    SchemaErrorKind::FieldMissing { field: "allOf" }.into(),
                    SchemaErrorKind::ExtraField { field: "allOff" }.into(),
                ]
            }
            .into()
        );
    }

    #[test]
    fn validate_unit_case() {
        let yaml = load_simple(
            r#"
            allOf:
              - type: integer
            "#,
        );
        let schema = SchemaAllOf::try_from(&yaml).unwrap();

        schema
            .validate(&Context::default(), &load_simple("10"))
            .unwrap();
    }

    #[test]
    fn validate_multiple_subvalidators() {
        let yaml = load_simple(
            r#"
                allOf:
                  - type: string
                    minLength: 10
                  - type: string
                    maxLength: 10
                "#,
        );

        let schema = SchemaAllOf::try_from(&yaml).unwrap();

        // Validate against a 10-character long string, causing overlap!
        schema
            .validate(&Context::default(), &load_simple("hello you!"))
            .unwrap();

        assert_eq!(
            schema
                .validate(&Context::default(), &load_simple("hi"))
                .unwrap_err(),
            ValidationErrorKind::ValidationError {
                error: "string length is less than minLength"
            }
            .into()
        );
    }
}
