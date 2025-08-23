use std::path::Path;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Lit, parse_macro_input};
use walkdir::WalkDir;

#[proc_macro]
pub fn load_artifacts(input: TokenStream) -> TokenStream {
    let mut token_include_paths = Vec::new();
    for token_tree in input.into_iter() {
        let literal_stream: TokenStream = token_tree.into();
        let literal = parse_macro_input!(literal_stream as Lit);

        // call site location.
        let span = proc_macro::Span::call_site();
        // get the path from the call site.
        let source_file = span.local_file().unwrap();
        let source_file_path = source_file.as_path();
        let parent_path = source_file_path.parent().unwrap();

        match literal {
            Lit::Str(lit_str) => {
                let directory = lit_str.value();
                let path = Path::new(&directory);

                let location = parent_path.join(path);

                for entry in WalkDir::new(location) {
                    let path = entry.as_ref().unwrap().path();
                    if !path.is_dir() && path.extension().map_or(false, |ext| ext == "json") {
                        let entry_path = path.to_str().unwrap();
                        token_include_paths.push(entry_path.replace(
                            (parent_path.to_str().unwrap().to_string() + "/").as_str(),
                            "",
                        ));
                    }
                }
            }
            _ => {
                panic!("Invalid literal for macro.")
            }
        }
    }

    let ret = quote! {
        &[ #(include_str!(#token_include_paths)),* ]
    };

    ret.into()
}
