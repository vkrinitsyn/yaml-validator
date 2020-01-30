use super::*;
use std::convert::TryFrom;
use std::str::FromStr;
use yaml_rust::YamlLoader;

const DIFFERENT_TYPES: &'static str = r#"---
schema:
  - name: somestring
    type: string

  - name: counter
    type: number

  - name: somedict
    type: dictionary
    value: 
      type: dictionary
      value:
        type: string
  - name: someobject
    type: object
    fields:
      - name: inside1
        type: string
      - name: inside2
        type: number
"#;

#[test]
fn deserialize_many_types() {
    let _rd = YamlSchema::from_str(DIFFERENT_TYPES);
}

#[test]
fn load_from_yaml() {
    let yaml = YamlLoader::load_from_str(DIFFERENT_TYPES).unwrap();
    for doc in yaml.into_iter() {
        let schema = YamlSchema::try_from(doc).unwrap();
        println!("{:?}", schema);
    }
}

#[test]
fn load_datastring_from_yaml_integer() {
    let integer = YamlLoader::load_from_str("20").unwrap().remove(0);

    assert_eq!(
        DataString::try_from(integer).unwrap_err().error,
        YamlSchemaError::SchemaParsingError("datastring is not an object")
    );
}

#[test]
fn load_datastring_with_string_max_length() {
    let wrong_optionals = YamlLoader::load_from_str(
        r#"---
type: string
max_length: hello
"#,
    )
    .unwrap()
    .remove(0);

    assert_eq!(
        DataString::try_from(wrong_optionals).unwrap_err().error,
        YamlSchemaError::WrongType("i64", "string")
    );
}

#[test]
fn load_datastring_with_extra_fields() {
    let wrong_optionals = YamlLoader::load_from_str(
        r#"---
type: string
extra_field: hello
"#,
    )
    .unwrap()
    .remove(0);

    assert_eq!(
        DataString::try_from(wrong_optionals).unwrap_err().error,
        YamlSchemaError::SchemaParsingError("string element contains superfluous elements")
    );
}

const YAML_SCHEMA: &'static str = r#"---
schema:
  - name: schema
    type: list
    inner:
      type: object
      fields:
        - name: name
          type: string
        - name: type
          type: string
        - name: inner
          type: object
          fields:
            - name: type
              type: string
            - name: fields
              type: list
              inner:
                type: dictionary
"#;

#[test]
fn validate_yaml_schema() {
    let schema = YamlSchema::from_str(YAML_SCHEMA).unwrap();

    schema.validate_str(&YAML_SCHEMA, None).unwrap();
}

const MISSING_NAME_FIELD_IN_SCHEMA: &'static str = r#"---
schema:
  - hello: world
"#;

#[test]
fn test_missing_fields_in_schema() {
    let schema = YamlSchema::from_str(YAML_SCHEMA).unwrap();

    let err = schema
        .validate_str(&MISSING_NAME_FIELD_IN_SCHEMA, None)
        .expect_err("this should fail");
    assert_eq!(
        format!("{}", err),
        "$.schema[0]: missing field, 'name' not found"
    );
}

const WRONG_TYPE_FOR_NAME_FIELD_IN_SCHEMA: &'static str = r#"---
schema:
  - name: 200
"#;

#[test]
fn test_wrong_type_for_field_in_schema() {
    let schema = YamlSchema::from_str(YAML_SCHEMA).unwrap();

    let err = schema
        .validate_str(&WRONG_TYPE_FOR_NAME_FIELD_IN_SCHEMA, None)
        .expect_err("this should fail");
    assert_eq!(
        format!("{}", err),
        "$.schema[0].name: wrong type, expected 'string' got 'Number(PosInt(200))'"
    );
}

const STRING_LIMIT_SCHEMA: &'static str = r#"---
schema:
  - name: somestring
    type: string
    max_length: 20
    min_length: 10
"#;

const STRING_LIMIT_TOO_SHORT: &'static str = "somestring: hello";
const STRING_LIMIT_TOO_LONG: &'static str = "somestring: hello world how are ya really";
const STRING_LIMIT_JUST_RIGHT: &'static str = "somestring: hello world";

#[test]
fn test_string_limits() {
    let schema = YamlSchema::from_str(STRING_LIMIT_SCHEMA).unwrap();

    assert_eq!(
        format!(
            "{}",
            schema
                .validate_str(&STRING_LIMIT_TOO_LONG, None)
                .expect_err("this should fail")
        ),
        "$.somestring: string validation error: string too long, max is 20, but string is 29"
    );

    assert_eq!(
        format!(
            "{}",
            schema
                .validate_str(&STRING_LIMIT_TOO_SHORT, None)
                .expect_err("this should fail")
        ),
        "$.somestring: string validation error: string too short, min is 10, but string is 5"
    );

    assert!(schema.validate_str(STRING_LIMIT_JUST_RIGHT, None).is_ok());
}

const DICTIONARY_WITH_SET_TYPES_SCHEMA: &'static str = r#"---
schema:
  - name: dict
    type: dictionary
    value:
      type: number
"#;

const DICTIONARY_WITH_CORRECT_TYPES: &'static str = r#"---
dict:
  hello: 10
  world: 20
"#;

const DICTIONARY_WITH_WRONG_TYPES: &'static str = r#"---
dict:
  hello: world
  world: hello
"#;

#[test]
fn test_dictionary_validation() {
    let schema = YamlSchema::from_str(DICTIONARY_WITH_SET_TYPES_SCHEMA).unwrap();

    assert!(schema
        .validate_str(&DICTIONARY_WITH_CORRECT_TYPES, None)
        .is_ok());
    assert_eq!(
        format!(
            "{}",
            schema
                .validate_str(&DICTIONARY_WITH_WRONG_TYPES, None)
                .expect_err("this should fail")
        ),
        "$.dict.hello: wrong type, expected 'number' got 'String(\"world\")'"
    );
}

const SCHEMA_WITH_URI: &'static str = r#"---
uri: myuri/v1
schema:
  - name: testproperty
    type: number
"#;

const SCHEMA_WITH_REFERENCE: &'static str = r#"---
schema:
  - name: propref
    type: reference
    uri: myuri/v1
"#;

const YAML_FILE_WITH_REFERENCE: &'static str = r#"---
propref:
  testproperty: 10
"#;

#[test]
fn test_schema_reference() {
    let context = YamlContext::from_schemas(vec![YamlSchema::from_str(SCHEMA_WITH_URI).unwrap()]);

    let schema = YamlSchema::from_str(SCHEMA_WITH_REFERENCE).unwrap();
    schema
        .validate_str(&YAML_FILE_WITH_REFERENCE, Some(&context))
        .unwrap();
}
