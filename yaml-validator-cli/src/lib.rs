use clap::Parser;
use std::convert::TryFrom;
use std::fs::read;
use std::path::Path;
use std::path::PathBuf;
use yaml_validator::{
    yaml_rust::{Yaml, YamlLoader},
    Context, Validate,
};

mod error;
use error::Error;

#[derive(Debug, Parser)]
#[command(
    name = "yaml-validator-cli",
    about = "    Command-line interface to the yaml-validator library.
    Use it to validate YAML files against a context of any number of cross-referencing schema files.
    The schema format is proprietary, and does not offer compatibility with any other known YAML tools"
)]
pub struct Opt {
    #[clap(
        short,
        long = "schema",
        help = "Schemas to include in context to validate against. Schemas are added in order, but do not validate references to other schemas upon loading."
    )]
    schemas: Vec<PathBuf>,

    #[clap(short, long, help = "URI of the schema to validate the files against.")]
    uri: String,

    #[clap(help = "Files to validate against the selected schemas.")]
    files: Vec<PathBuf>,
}

fn read_file(filename: &Path) -> Result<String, Error> {
    let contents = read(filename).map_err(|e| {
        Error::File(format!(
            "could not read file {}: {}\n",
            filename.to_string_lossy(),
            e
        ))
    })?;

    let utf8 = String::from_utf8_lossy(&contents).parse().map_err(|e| {
        Error::File(format!(
            "file {} did not contain valid utf8: {}\n",
            filename.to_string_lossy(),
            e
        ))
    })?;

    Ok(utf8)
}

fn load_yaml(filenames: &[PathBuf]) -> Result<Vec<Yaml>, Vec<Error>> {
    let (yaml, errs): (Vec<_>, Vec<_>) = filenames
        .iter()
        .map(|file| {
            read_file(file)
                .and_then(|source| YamlLoader::load_from_str(&source).map_err(Error::from))
        })
        .partition(Result::is_ok);

    if errs.is_empty() {
        Ok(yaml.into_iter().flat_map(Result::unwrap).collect())
    } else {
        Err(errs.into_iter().map(Result::unwrap_err).collect())
    }
}

// Ideally this would just be the real main function, but since errors are
// automatically printed using the Debug trait rather than Display, the error
// messages are not very easy to read.
pub fn actual_main(opt: &Opt) -> Result<(), Error> {
    if opt.schemas.is_empty() {
        return Err(Error::Validation(
            "no schemas supplied, see the --schema option for information\n".into(),
        ));
    }

    if opt.files.is_empty() {
        return Err(Error::Validation(
            "no files to validate were supplied, use --help for more information\n".into(),
        ));
    }

    let yaml_schemas = load_yaml(&opt.schemas).map_err(Error::Multiple)?;
    let context = Context::try_from(&yaml_schemas[..])?;

    let schema = {
        if let Some(schema) = context.get_schema(&opt.uri) {
            schema
        } else {
            return Err(Error::Validation(format!(
                "schema referenced by uri `{}` not found in context\n",
                opt.uri
            )));
        }
    };

    let documents = opt
        .files
        .iter()
        .zip(load_yaml(&opt.files).map_err(Error::Multiple)?);

    for (name, doc) in documents {
        schema.validate(&context, &doc).map_err(|err| {
            Error::Validation(format!(
                "{name}:\n{err}",
                name = name.to_string_lossy(),
                err = err
            ))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
                "could not read file not_found.yaml: No such file or directory (os error 2)\n"
                    .into()
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
                "could not read file not_found.yaml: No such file or directory (os error 2)\n"
                    .into()
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
}