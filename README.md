# procout

## procout
__What:__ It prints the output of a _procedural_ macro to a file.
__Wherefore:__ To ease debugging by clarifying the source  of errors with explicit line numbers instead of the unavoidably opaque errors often produced when debugging
procedural macros in Rust.
__Whereby:__ Add a function call to your proc macro and use the command-line feature.

This depends on the procedural macro _compiling_ to code. If it's not at the stage where it compiles,
it has to get there before this will produce useful output.

### Whereby
Given a procedural macro's constructed as so,

```rust
use proc_macro::{TokenStream};
use proc_macro2::{Span};
use quote::{quote};
use syn::{Ident};

#[proc_macro]
pub fn ast(input: TokenStream) -> TokenStream {
  let module_ident = Ident::new("this_module", Span::mixed_site());
  let code_block: proc_macro2::TokenStream = quote!{
     pub mod #module_ident {
       /* ... some truly fantastic code, well done ... */
     }
  };
  // Convert and return the code
  TokenStream::from(code_block)
}
```
Just insert a call to procout before the conversion and return step.

```rust
use proc_macro::{TokenStream};
use proc_macro2::{Span};
use procout::{procout}; // Look!
use quote::{quote};
use syn::{Ident};

#[proc_macro]
pub fn ast(input: TokenStream) -> TokenStream {
  let module_ident = Ident::new("this_module", Span::mixed_site());
  let code_block: proc_macro2::TokenStream = quote!{
     pub mod #module_ident {
       /* ... some truly fantastic code, well done ... */
     }
  };

  // Look!
  procout(&code_block, Some(module_ident), Some("a/valid/path/string"));

  // Convert and return the code
  TokenStream::from(code_block)
}
```

Now, when calling something like `cargo test --features procout`

The code will print to the `a/valid/path/string` specified as a file corresponding to `module_ident`.
By default, the path string is the local `tests` directory, so __after__ the first run using the `procout`
feature, it's possible to run something like `cargo test --test module_ident` and get better errors
from the compiler.

Now with these splendid features:
- A unit test module will be generated with a no-op test that just imports the module named in `module_ident`.
- `module_ident` should be the name of a generated module.
- If no path is specified, the default path will be the current working directory's `tests` subfolder,
- If no `module_ident` is specified, the default will be a generic timestamp.

#### Warning:
This will overwrite whatever's at the specified path, so be careful when prototyping.

### Features
- `procout` Outputs the macro to a file. Calling `procout` with this feature disabled is an intentional no-op.
- `formatted` Calls `rustfmt` on the created file. This is enabled by default and is recommended.
- `notification` Prints a notification to stdout on success. This is enabled by default.

License: MIT
