use clap::{error, Error, Parser};
use convert_case::{Case, Casing};
use std::env;
use std::panic;
use std::{fs, path::PathBuf};
use webidl2wit::{ConversionOptions, HandleUnsupported};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The pattern to look for
  input_idl_path: std::path::PathBuf,
  /// The path to the file to read
  output_wit_path: std::path::PathBuf,
}

fn convert_file(input_file: &PathBuf, output_file: &PathBuf) -> Result<(), Error> {
  println!(
    "Converting file: {} -> {}",
    input_file.display(),
    output_file.display()
  );
  let webidl_input = match fs::read_to_string(&input_file) {
    Ok(s) => s,
    Err(e) => {
      return Err(Error::raw(
        error::ErrorKind::Io,
        format!("Error reading input file: {}", e),
      ));
    }
  };
  // Convert
  let result = panic::catch_unwind(|| {
    // Set Conversion Options
    let convert_options = ConversionOptions {
      interface_name: format!(
        "{}-interface",
        input_file
          .file_stem()
          .unwrap()
          .to_string_lossy()
          .to_string()
          .chars()
          .filter(|c| !c.is_numeric())
          .collect::<String>()
          .to_case(Case::Kebab)
      ),
      singleton_interface: Some("global-".to_string()),
      // resource_inheritance: ResourceInheritance::DuplicateMethods,
      unsupported_features: HandleUnsupported::Skip,
      ..Default::default()
    };
    let webidl_ast = match weedle::parse(&webidl_input) {
      Ok(ast) => ast,
      Err(e) => {
        return Err(Error::raw(
          error::ErrorKind::Io,
          format!("Error parsing input file: {}", e),
        ));
      }
    };
    let wit_ast = match webidl2wit::webidl_to_wit(webidl_ast, convert_options) {
      Ok(ast) => ast,
      Err(e) => {
        println!("Error converting webidl to wit: {}", e);
        // Non fatal
        return Ok(None);
      }
    };
    let wit_output = wit_ast.to_string();
    return Ok(Some(wit_output));
  });
  let wit_output = match result {
    Ok(Ok(Some(s))) => s,
    Ok(Err(e)) => {
      return Err(e);
    }
    Err(_) | Ok(Ok(None)) => {
      // TODO: non fatal error
      return Ok(());
    }
  };
  // Write Output File
  match fs::write(&output_file, wit_output) {
    Ok(_) => (),
    Err(e) => {
      return Err(Error::raw(
        error::ErrorKind::Io,
        format!("Error writing output file: {}", e),
      ));
    }
  };
  Ok(())
}

fn convert_directory(
  base_dir: &PathBuf,
  input_dir: &PathBuf,
  output_dir: &PathBuf,
) -> Result<(), Error> {
  println!(
    "Converting directory: {} -> {}",
    input_dir.display(),
    output_dir.display()
  );
  for entry in fs::read_dir(&input_dir)? {
    let path = match entry {
      Ok(e) => e.path(),
      Err(e) => {
        return Err(Error::raw(
          error::ErrorKind::Io,
          format!("Error reading directory: {}", e),
        ));
      }
    };
    let relative_path = match path.strip_prefix(base_dir) {
      Ok(p) => p,
      Err(e) => {
        return Err(Error::raw(
          error::ErrorKind::Io,
          format!("Error stripping prefix: {}", e),
        ));
      }
    };
    match path.is_dir() {
      true => {
        let output_path = output_dir.join(relative_path);
        match convert_directory(&base_dir, &path, &output_path) {
          Err(e) => return Err(e),
          Ok(_) => (),
        }
      }
      false => {
        if path
          .extension()
          .map_or(true, |x| x != "idl" && x != "webidl")
        {
          continue;
        }
        let file_name = match path.file_name() {
          None => {
            return Err(Error::raw(
              error::ErrorKind::Io,
              format!("Error reading file name: {}", path.display()),
            ));
          }
          Some(f) => f,
        };
        let output_file = output_dir.join(file_name).with_extension("wit");
        match convert_file(&path, &output_file) {
          Err(e) => return Err(e),
          Ok(_) => (),
        }
      }
    };
  }
  Ok(())
}

fn main() -> Result<(), Error> {
  env::set_var("RUST_BACKTRACE", "1");
  let args = Cli::parse();
  // Read Input File
  #[warn(unused_parens)]
  let result = match (args.input_idl_path.is_dir(), args.output_wit_path.is_dir()) {
    (true, false) => Err(Error::raw(
      error::ErrorKind::Io,
      "Cannot output directory to file",
    )),
    (false, false) => convert_file(&args.input_idl_path, &args.output_wit_path),
    (false, true) => {
      let file_name = match args.input_idl_path.file_name() {
        None => {
          return Err(Error::raw(
            error::ErrorKind::Io,
            format!("Error reading file name: {}", args.input_idl_path.display()),
          ));
        }
        Some(f) => f,
      };
      let output_file = &args.output_wit_path.join(file_name).with_extension("wit");
      convert_file(&args.input_idl_path, &output_file)
    }
    (true, true) => convert_directory(
      &args.input_idl_path,
      &args.input_idl_path,
      &args.output_wit_path,
    ),
  };

  // End With Ok
  match result {
    Ok(_) => {
      println!("Conversion successful");
      Ok(())
    }
    Err(e) => e.exit(),
  }
}
