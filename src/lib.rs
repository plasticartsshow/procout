//! # procout 
//! __What:__ It prints the output of a _procedural_ macro to a file. 
//! __Wherefore:__ To ease debugging by clarifying the source  of errors with explicit line numbers instead of the unavoidably opaque errors often produced when debugging 
//! procedural macros in Rust. 
//! __Whereby:__ Add a function call to your proc macro and use the command-line feature. 
//!
//! This depends on the procedural macro _compiling_ to code. If it's not at the stage where it compiles, 
//! it has to get there before this will produce useful output.
//!
//! ## Whereby 
//! Given a procedural macro's constructed as so,
//! 
//! ```ignore
//! use proc_macro::{TokenStream};
//! use proc_macro2::{Span};
//! use quote::{quote};
//! use syn::{Ident};
//! 
//! #[proc_macro]
//! pub fn ast(input: TokenStream) -> TokenStream {
//!   let module_ident = Ident::new("this_module", Span::mixed_site());
//!   let code_block: proc_macro2::TokenStream = quote!{  
//!      pub mod #module_ident {
//!        /* ... some truly fantastic code, well done ... */
//!      }
//!   };
//!   // Convert and return the code 
//!   TokenStream::from(code_block)
//! }
//! ```
//! Just insert a call to procout before the conversion and return step. 
//! 
//! ```ignore
//! use proc_macro::{TokenStream};
//! use proc_macro2::{Span}; 
//! use procout::{procout}; // Look!
//! use quote::{quote};
//! use syn::{Ident};
//! 
//! #[proc_macro]
//! pub fn ast(input: TokenStream) -> TokenStream {
//!   let module_ident = Ident::new("this_module", Span::mixed_site());
//!   let code_block: proc_macro2::TokenStream = quote!{  
//!      pub mod #module_ident {
//!        /* ... some truly fantastic code, well done ... */
//!      }
//!   };
//!
//!   // Look!
//!   procout(&code_block, Some(module_ident), Some("a/valid/path/string"));
//!
//!   // Convert and return the code 
//!   TokenStream::from(code_block)
//! }
//! ```
//!
//! Now, when calling something like `cargo test --features procout`
//!
//! The code will print to the `a/valid/path/string` specified as a file corresponding to `module_ident`.
//! By default, the path string is the local `tests` directory, so __after__ the first run using the `procout`
//! feature, it's possible to run something like `cargo test --test module_ident` and get better errors 
//! from the compiler. 
//!
//! Now with these splendid features:  
//! - A unit test module will be generated with a no-op test that just imports the module named in `module_ident`.
//! - `module_ident` should be the name of a generated module.
//! - If no path is specified, the default path will be the current working directory's `tests` subfolder,
//! - If no `module_ident` is specified, the default will be a generic timestamp.
//!  
//! ### Warning:
//! This will overwrite whatever's at the specified path, so be careful when prototyping. 
//!
//! ## Features 
//! - `procout` Outputs the macro to a file. Calling `procout` with this feature disabled is an intentional no-op.
//! - `formatted` Calls `rustfmt` on the created file. This is enabled by default and is recommended. 
//! - `notification` Prints a notification to stdout on success. This is enabled by default. 
use chrono::{
  DateTime, Utc
};
use inflector::{
  cases::{
    snakecase::{to_snake_case}
  },
};
use proc_macro2::{
  TokenStream,
  Span,
};
use quote::{
  quote
};
use std::{
  env, 
  fs::{
    DirBuilder, File,
  },
  io::{
    prelude::*,
  },
  path::{
    PathBuf
  },
  process::{
    Command,
  },
  string::{
    ToString,
  },
};
use syn::{
  Ident,
};

/// The format used for default timestamped file names
pub static TIMESTAMP_FORMAT: &str = "out_%Y_%m%d_%H%S";

/// Handle printing code to a file 
/// - `code_block` This is the code that should be printed (the [TokenStream] output of the macro being debugged)
/// - `module_ident` This is the optional name of the module generated by the macro.  
/// - `output_path` This is the directory to write the file to.
pub fn procout(
  code_block: &TokenStream,
  module_ident: Option<Ident>,
  output_path: Option<&str>,
) {
  if cfg!(any(feature = "procout", feature="procout_messy")) {
    // Select a target path 
    let mut target_path: PathBuf = output_path.map_or_else(
      || {
        let mut local_path = env::current_dir().expect("Must identify current dir");
        local_path.push("tests");
        local_path
      },
      |path_str| {
        PathBuf::from(path_str)
      }
    );
    
    // Create the path ignoring existing 
    DirBuilder::new()
      .recursive(true)
      .create(target_path.clone())
      .expect("Creates macro output dir");
    
    // Parse the module Ident
    let module_ident: Ident = module_ident.unwrap_or_else(
      || {
        let now: DateTime<Utc> = Utc::now();
        let timestamp: String = format!("{}", now.format(&TIMESTAMP_FORMAT));
        Ident::new(&timestamp, Span::mixed_site()) 
      }
    );
    // Pick a file name 
    let file_name = format!("{}.rs", to_snake_case(&module_ident.to_string()));
    target_path.push(file_name);
    let target_path_str = target_path.to_str().expect("Must create string from target path");
    let mut target_file = File::create(target_path.clone())
      .expect("Creates macro output file");
    
    // Write to file
    target_file.write_all(&format!(
      "{}",
      quote!{
        #![allow(unused_imports)]
        #![allow(dead_code)]
        #code_block
        #[test]
        fn macro_test() {
          use #module_ident::*;
        }
      }
    ).as_bytes())
      .expect("Writes macro to file as test");
    
    if cfg!(feature = "notification") {
      std::println!("Wrote macro to `{}` ", target_path_str);
    }
    
    if cfg!(feature = "formatted") {
      // Try to rustfmt the output, ignoring failure 
      match Command::new("rustfmt").arg(target_path_str).output() {
        Ok(output) => std::println!("rustfmt status: {}", output.status),
        Err(err) => std::println!("Could not rustfmt \"{}\":\n {:#?}", target_path_str, err),
      }
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_procout() {
    let target_module = "test_procout_module";
    let module_ident = Ident::new(&target_module, Span::mixed_site());
    let code_block: proc_macro2::TokenStream = quote!{  
       pub mod #module_ident {
         const CUSS: &str = "SPIT";
       }
    };
    
    procout(&code_block, Some(module_ident), None);
    procout(&code_block, None, Some("tests/blah"));
    let target_output = format!( 
      "#![allow(unused_imports)]\
      \n#![allow(dead_code)]\
      \npub mod {} {{\
      \n    const CUSS: &str = \"SPIT\";\n\
      }}\n\
      #[test]\n\
      fn macro_test() {{\
      \n    use {}::*;\n\
      }}\n",
      target_module,
      target_module,
    );
    let mut target_path: PathBuf = env::current_dir().expect("Must identify current dir");
    target_path.push("tests");
    target_path.push(format!("{}.rs", target_module));
    let mut target_file = File::open(target_path).expect("Must open target file");
    
    let mut contents = String::new();
    target_file.read_to_string(&mut contents).expect("Test must read file to string");
    
    assert_eq!(
      contents,
      target_output,
      "Must write target output to file in tests directory corresponding to module Ident"
    );
  }
}