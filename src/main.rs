use std::{
    env,
    fs::{File, create_dir},
    io::{BufReader, BufRead},
    path::Path
};
use seahorse::{App, Command, Context, Flag, FlagType};
use yaml_rust::YamlLoader;
use crate::{
    git::git,
    build::build,
    execute::execute,
    clean::clean,
    model::{Project, FromYamlDocument, ParsingError},
    AppError::IOError
};
use std::rc::Rc;

mod model;
mod git;
mod build;
mod execute;
mod tsv;
mod clean;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let app = App::new()
        .name(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .action(safe_wrapper)
        .flag(Flag::new("requirements", "replikate [config] --requirements", FlagType::Bool))
        .flag(Flag::new("git", "replikate [config] --git(-g)", FlagType::Bool).alias("g"))
        .flag(Flag::new("build", "replikate [config] --build(-b)", FlagType::Bool).alias("b"))
        .flag(Flag::new("run", "replikate [config] --run(-r)", FlagType::Bool).alias("r"))
        .flag(Flag::new("clean", "replikate [config] --clean", FlagType::Bool))
        .command(Command::new().name("help").usage("help"));
    app.run(args);
}

#[derive(Clone, Debug)]
pub enum AppError {
    MissingArgument(&'static str),
    IOError(String, Rc<std::io::Error>),
    ExternalError(String),
    Parsing(ParsingError),
}

fn safe_wrapper(c: &Context) {
    let execution = run_app(c);
    if let Some(err) = execution.err() {
        match err {
            AppError::MissingArgument(name) => println!("Missing argument '{}', use --help to show usage.", name),
            AppError::IOError(path, sub_error) => println!("{} for '{}'.", sub_error, path),
            AppError::ExternalError(message) => println!("{}", message),
            AppError::Parsing(err) => { println!("Cannot parse the configuration file: {:?}", err) }
        }
    }
}

fn run_app(c: &Context) -> Result<(), AppError> {
    let config = c.args.first()
        .ok_or(AppError::MissingArgument("config"))?;

    let config_file = File::open(config)
        .map_err(|err| AppError::IOError(config.to_owned(), Rc::new(err)))?;

    let buf = BufReader::new(config_file);
    let mut file_content = String::new();

    for line in buf.lines() {
        let line = line.map_err(|e| IOError(config.to_owned(), Rc::new(e)))?;
        file_content.push_str(&line);
        file_content.push('\n');
    }

    let yaml_doc = YamlLoader::load_from_str(&file_content)
        .map_err(|_| AppError::ExternalError(format!("Cannot parse {} as yaml file.", config).to_owned()))?;

    let path = if config.contains('.') {
        let index_of_last_dot = config.rfind('.').unwrap();
        config[0..index_of_last_dot].to_owned()
    } else {
        config.to_owned()
    };
    let project = Project::from_yaml(&yaml_doc[0])
        .map_err(|e| AppError::Parsing(e))?
        .set_path(&path);

    create_tree(&project)?;

    if c.bool_flag("requirements") {
        println!("Requirements: ");
        for requirement in &project.requirements {
            println!("  {}, version: {}", requirement.name, requirement.version)
        }
    }

    if c.bool_flag("git") {
        git(&project)?;
    }
    if c.bool_flag("build") {
        build(&project)?;
    }
    if c.bool_flag("clean") {
        clean(&project)?;
        create_tree(&project)?;
    }

    if c.bool_flag("run") {
        execute(&project)?;
    }

    Ok(())
}

fn create_tree(p: &Project) -> Result<(), AppError> {
    let into_err = |p: &Path| {
        let p = p.to_str().unwrap().to_owned();
        |e: std::io::Error| AppError::IOError(p, Rc::new(e))
    };

    let path = Path::new(&p.path);
    if !path.exists() {
        create_dir(path)
            .map_err(into_err(path))?;
    }

    let src = path.join("src");
    let src = src.as_path();
    if !src.exists() {
        create_dir(src)
            .map_err(into_err(src))?;
    }

    let results = path.join("logs");
    let results = results.as_path();
    if !results.exists() {
        create_dir(results)
            .map_err(into_err(results))?;
    }

    for exp in &p.experiments {
        let exp_folder = results.join(&exp.name);
        let exp_folder = exp_folder.as_path();

        if !exp_folder.exists() {
            create_dir(exp_folder)
                .map_err(into_err(exp_folder))?;
        }
    }

    Ok(())
}

