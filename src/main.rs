use std::{io::ErrorKind, process::exit};

use clap::Parser;
use inquire::validator::Validation;

static C_GIT_IGNORE: &str = include_str!("templates/c.gitignore");
static CPP_GIT_IGNORE: &str = include_str!("templates/cpp.gitignore");

#[derive(Parser, Default)]
struct Args {
    /// Project name
    #[arg(short, long)]
    name: Option<String>,

    /// C or CPP
    #[arg(short, long)]
    language: Option<String>,
}

struct ValidArgs<'a> {
    name: String,
    language: String,
    extension: String,
    gitignore: &'a str,
    cmake: String,
}

impl Args {
    fn new(name: Option<String>, language: Option<String>) -> Self {
        Args { name, language }
    }

    fn to_real(self) -> ValidArgs<'static> {
        let name = self.name.unwrap();
        let language = self.language.unwrap().to_owned();
        let (extension, gitignore, cmake) = match language.as_str() {
            "C" => (".c".into(), &C_GIT_IGNORE, "C".into()).into(),
            "CPP" => (".cpp".into(), &CPP_GIT_IGNORE, "CXX".into()),
            _ => (".c".into(), &C_GIT_IGNORE, "C".into()),
        };

        ValidArgs {
            name,
            language,
            extension,
            gitignore,
            cmake,
        }
    }
}

fn main() {
    let args = Args::parse();
    let valid_args: ValidArgs;

    if args.language.is_some() && args.name.is_some() {
        let v_args = args.to_real();
        if v_args.language != "C" && v_args.language != "CPP" {
            eprintln!("Language: Only C and CPP (C++) available.");
            exit(1);
        }
        valid_args = v_args;
    } else {
        valid_args = interactive_prompt().to_real();
    }

    create_project(&valid_args);
}

fn interactive_prompt() -> Args {
    let name = inquire::Text::new("Project name")
        .with_validator(|name: &str| {
            if name.trim().len() == 0 {
                return Ok(Validation::Invalid(
                    "Project name necessary ( ｡ •̀ ᴖ •́ ｡)".into(),
                ));
            } else {
                if !match std::env::consts::OS {
                    "linux" | "freebsd" => !name.contains('/'),
                    "windows" => name
                        .chars()
                        .any(|c| ['<', '>', ':', '"', '/', '\\', '|', '?', '*'].contains(&c)),
                    "macos" => !name.contains("/") && !name.contains(":"),
                    _ => {
                        return Ok(Validation::Invalid(
                            "Could not identify the OS ( – ⌓ – )".into(),
                        ))
                    }
                } {
                    return Ok(Validation::Invalid(
                        "Invalid character in name ( ｡ •̀ ᴖ •́ ｡)".into(),
                    ));
                };
            }
            return Ok(Validation::Valid);
        })
        .prompt();

    let options: Vec<&str> = vec!["C", "CPP"];
    let language = inquire::Select::new("Language: ", options)
        .with_help_message("For the creation of CMake file and the main script")
        .prompt();

    Args::new(
        Some(name.unwrap().to_string()),
        Some(language.unwrap().to_string()),
    )
}

fn create_project(args: &ValidArgs) {
    let cmake = &args.cmake;
    let project_name = &args.name;
    let language = &args.language;
    let gitignore = &args.gitignore;
    let extension = &args.extension;

    if let Err(err) = std::fs::create_dir(&project_name) {
        if err.kind() == ErrorKind::AlreadyExists {
            eprintln!("\"{}\" folder already exists (｡•́︿•̀｡)", &project_name);
            exit(1);
        } else {
            eprintln!(
                "Could not create project folder (｡•́︿•̀｡): {}",
                err.to_string()
            );
            exit(1);
        }
    } else {
        let _ = std::fs::create_dir(format!("./{}/src", &project_name))
            .expect("Could not create \"src\" folder (｡•́︿•̀｡)");
        let _ = std::fs::create_dir(format!("./{}/build", &project_name))
            .expect("Could not create \"build\" folder (｡•́︿•̀｡)");
        let _ = std::fs::create_dir(format!("./{}/include", &project_name))
            .expect("Could not create \"include\" folder (｡•́︿•̀｡)");

        let undo_all = || {
            let _ = std::fs::remove_dir_all(format!("./{}", project_name));
            exit(1);
        };

        let c_make = format!(
            "cmake_minimum_required(VERSION 3.11)

set(PROJECT_NAME {})
                
project(${{PROJECT_NAME}} {})

file(GLOB_RECURSE SOURCES \"src/*.cpp\")

include_directories(${{CMAKE_SOURCE_DIR}}/include)

add_executable(${{PROJECT_NAME}} ${{SOURCES}})",
            project_name, cmake
        );

        let main_c = format!(
            "#include <stdio.h>

int main(void) 
{{
    printf(\"Hello World\");
    return 0;
}}"
        );

        let main_cpp = format!(
            "#include <iostream>

int main() 
{{
    std::cout << \"Hello World\" << std::endl;
    return 0;
}}"
        );

        if let Err(err) = std::fs::write(format!("./{}/CMakeLists.txt", &project_name), c_make) {
            eprintln!(
                "Could not create CMakeLists.txt script (｡•́︿•̀｡): {}",
                err.to_string()
            );
            undo_all();
        };

        if let Err(err) = std::fs::write(
            format!("./{}/src/main{}", project_name, extension),
            match language.as_str() {
                "C" => main_c,
                "CPP" => main_cpp,
                _ => main_c,
            },
        ) {
            eprintln!(
                "Could not create main script file (｡•́︿•̀｡): {}",
                err.to_string()
            );
            undo_all();
        };

        if let Err(err) = std::fs::write(
            format!("./{}/build.sh", &project_name),
            format!(
                "cmake -S . -B build -G \"Ninja\"
cmake --build build
./build/{}
            ",
                project_name
            ),
        ) {
            eprintln!(
                "Could not create build script (｡•́︿•̀｡): {}",
                err.to_string()
            );
            undo_all();
        };

        if let Err(err) = std::fs::write(format!("./{}/.gitignore", project_name), gitignore) {
            eprintln!("Could not create .gitignore (｡•́︿•̀｡): {}", err.to_string());
            undo_all();
        }
    }
}
