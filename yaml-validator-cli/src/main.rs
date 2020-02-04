use std::convert::TryFrom;
use std::fs::read;
use std::path::PathBuf;
use structopt::StructOpt;
use yaml_validator::{Context, Validate, Yaml, YamlLoader};

mod error;
use error::Error;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "yaml-validator-cli",
    about = "    Command-line interface to the yaml-validator library.
    Use it to validate YAML files against a context of any number of cross-referencing schema files.
    The schema format is proprietary, and does not offer compatibility with any other known YAML tools"
)]
struct Opt {
    #[structopt(
        parse(from_os_str),
        short,
        long = "schema",
        help = "Schemas to include in context to validate against. Schemas are added in order, but do not validate references to other schemas upon loading."
    )]
    schemas: Vec<PathBuf>,

    #[structopt(short, long, help = "URI of the schema to validate the files against.")]
    uri: String,

    #[structopt(
        parse(from_os_str),
        help = "Files to validate against the selected schemas."
    )]
    files: Vec<PathBuf>,
}

fn read_file(filename: &PathBuf) -> Result<String, Error> {
    Ok(String::from_utf8_lossy(&read(filename).unwrap())
        .parse()
        .unwrap())
}

fn load_yaml(filenames: Vec<PathBuf>) -> Result<Vec<Yaml>, Vec<Error>> {
    let (yaml, errs): (Vec<_>, Vec<_>) = filenames
        .iter()
        .map(|file| {
            read_file(&file)
                .and_then(|source| YamlLoader::load_from_str(&source).map_err(Error::from))
        })
        .partition(Result::is_ok);

    if !errs.is_empty() {
        Err(errs.into_iter().map(Result::unwrap_err).collect())
    } else {
        Ok(yaml.into_iter().map(Result::unwrap).flatten().collect())
    }
}

fn secret_main(opt: Opt) -> Result<(), Error> {
    if opt.schemas.is_empty() {
        return Err(Error::ValidationError(
            "No schemas supplied, see the --schema option for information".into(),
        ));
    }

    if opt.files.is_empty() {
        return Err(Error::ValidationError(
            "No files to validate were supplied, use --help for more information".into(),
        ));
    }

    let yaml_schemas = load_yaml(opt.schemas).map_err(Error::Multiple)?;
    let context = Context::try_from(&yaml_schemas)?;

    let schema = {
        if let Some(schema) = context.get_schema(&opt.uri) {
            schema
        } else {
            return Err(Error::ValidationError(format!(
                "Schema referenced by uri `{}` not found in context",
                opt.uri
            )));
        }
    };

    let documents = load_yaml(opt.files).map_err(Error::Multiple)?;
    for doc in documents {
        schema
            .validate(&context, &doc)
            .map_err(|err| Error::ValidationError(format!("{:?}: {}", doc, err)))?;
        println!("valid");
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    match secret_main(opt) {
        Ok(()) => println!("All files validated successfully!"),
        Err(e) => {
            println!("failed: {}", e);
            std::process::exit(1);
        }
    }
}
