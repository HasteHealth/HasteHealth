use std::path::Path;

use crate::utilities::{FHIR_PRIMITIVES, RUST_KEYWORDS, generate::capitalize, load};
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, OperationDefinitionParameter, Resource, ResourceType},
    terminology::{AllTypes, BoundCode, OperationParameterUse},
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use walkdir::WalkDir;

fn get_operation_definitions(resource: &Resource) -> Result<Vec<&OperationDefinition>, String> {
    match resource {
        Resource::Bundle(bundle) => Ok(bundle
            .entry
            .as_ref()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| entry.resource.as_ref())
                    .filter_map(|resource| match resource.as_ref() {
                        Resource::OperationDefinition(op_def) => Some(op_def),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()),
        Resource::OperationDefinition(op_def) => Ok(vec![op_def]),
        _ => Err("Resource is not a Bundle or OperationDefinition".to_string()),
    }
}

fn get_name(op_def: &OperationDefinition) -> String {
    op_def
        .id
        .clone()
        .expect("Operation definition must have an id.")
        .split('-')
        .map(capitalize)
        .collect()
}

fn parameter_type_name(type_: &str) -> &str {
    if let Some(primitive) = FHIR_PRIMITIVES.get(type_) {
        primitive.as_str()
    } else if type_ == "Element" {
        "ParametersParameterValueTypeChoice"
    } else {
        type_
    }
}

fn parameter_type_ident(type_: &str) -> Ident {
    format_ident!("{}", parameter_type_name(type_))
}

fn create_field_value(type_: &str, is_array: bool, required: bool) -> TokenStream {
    let type_ident = parameter_type_ident(type_);

    let field_type = if is_array {
        quote! { Vec<#type_ident> }
    } else {
        quote! { #type_ident }
    };

    if required {
        quote! { #field_type }
    } else {
        quote! { Option<#field_type> }
    }
}

fn is_resource_return(parameters: &[&OperationDefinitionParameter]) -> bool {
    parameters.len() == 1
        && parameters[0].name.value.as_deref() == Some("return")
        && parameters[0].type_.as_ref().is_some_and(|parameter_type| {
            parameter_type == &AllTypes::any()
                || ResourceType::try_from(parameter_type.as_str().unwrap_or_default()).is_ok()
        })
}

fn generate_parameter_type(
    name: &str,
    parameters: &[&OperationDefinitionParameter],
    is_base: bool,
) -> Vec<TokenStream> {
    let mut generated_types = Vec::new();
    let mut fields = Vec::with_capacity(parameters.len());

    for parameter in parameters {
        let (field_ident, attribute_rename) = process_field_names(parameter);

        let description = parameter
            .documentation
            .as_ref()
            .and_then(|documentation| documentation.value.as_deref())
            .map_or_else(|| field_ident.to_string(), format_documentation);

        let doc_attributes = generate_doc_attributes(&description);

        let is_array = parameter.max.value.as_deref() != Some("1");
        let required = parameter.min.value.unwrap_or_default() > 0;

        let (field_type, nested_attribute) = if let Some(type_) = parameter.type_.as_ref() {
            let type_name = if type_ == &AllTypes::any() {
                "Resource"
            } else {
                type_.as_str().unwrap_or_default()
            };

            (create_field_value(type_name, is_array, required), quote! {})
        } else {
            let nested_struct_name = format_nested_name(name, parameter);

            let nested_parameters = parameter
                .part
                .as_deref()
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>();

            generated_types.extend(generate_parameter_type(
                &nested_struct_name,
                &nested_parameters,
                false,
            ));

            (
                create_field_value(&nested_struct_name, is_array, required),
                quote! {
                    #[parameter_nested]
                },
            )
        };

        fields.push(quote! {
            #doc_attributes
            #attribute_rename
            #nested_attribute
            pub #field_ident: #field_type
        });
    }

    generated_types.push(build_struct_tokens(name, parameters, &fields, is_base));

    generated_types
}

fn process_field_names(p: &OperationDefinitionParameter) -> (Ident, TokenStream) {
    let initial_name = p.name.value.as_deref().expect("Parameter must have a name");

    let replaced_name = initial_name.replace('-', "_");

    // A leading `_` is a FHIR naming convention and is not meaningful to the
    // generated Rust field name. Preserve the original FHIR name through
    // #[parameter_rename = "..."] instead.
    let formatted_name = replaced_name
        .strip_prefix('_')
        .unwrap_or(&replaced_name)
        .to_string();

    let is_rust_keyword = RUST_KEYWORDS.contains(&formatted_name.as_str());

    let field_ident = if is_rust_keyword {
        format_ident!("{}_", formatted_name)
    } else {
        format_ident!("{}", formatted_name)
    };

    let attribute_rename =
        if formatted_name != *initial_name || replaced_name != *initial_name || is_rust_keyword {
            quote! {
                #[parameter_rename = #initial_name]
            }
        } else {
            quote! {}
        };

    (field_ident, attribute_rename)
}

fn format_nested_name(parent_name: &str, parameter: &OperationDefinitionParameter) -> String {
    let initial_name = parameter
        .name
        .value
        .as_deref()
        .expect("Parameter must have a name");

    let formatted_name = initial_name.replace('-', "_");

    let capitalized_parts = formatted_name
        .split('_')
        .map(capitalize)
        .collect::<String>();

    format!("{parent_name}{capitalized_parts}")
}

fn resource_return_tokens(parameters: &[&OperationDefinitionParameter]) -> Option<TokenStream> {
    if !is_resource_return(parameters) {
        return None;
    }

    let parameter = parameters.first()?;

    let required = parameter.min.value.unwrap_or_default() > 0;

    let type_str = parameter
        .type_
        .as_ref()
        .and_then(BoundCode::as_str)
        .unwrap_or_default();

    let return_type = if type_str == "Any" {
        "Resource"
    } else {
        type_str
    };

    let return_type_ident = format_ident!("{}", return_type);

    let return_value = if required {
        quote! { value.return_ }
    } else {
        quote! { value.return_.unwrap_or_default() }
    };

    Some(if return_type == "Resource" {
        quote! { #return_value }
    } else {
        quote! { Resource::#return_type_ident(#return_value) }
    })
}

/// Constructs the final `TokenStream` (struct definition and `From`
/// implementation) by differentiating between base resource returns and
/// standard parameter wraps.
fn build_struct_tokens(
    name: &str,
    parameters: &[&OperationDefinitionParameter],
    fields: &[TokenStream],
    is_base: bool,
) -> TokenStream {
    let struct_name = format_ident!("{}", name);

    if is_base && let Some(returned_value) = resource_return_tokens(parameters) {
        return quote! {
            #[derive(Debug, FromParameters)]
            pub struct #struct_name {
                #(#fields),*
            }

            impl From<#struct_name> for Resource {
                fn from(value: #struct_name) -> Self {
                    #returned_value
                }
            }
        };
    }

    quote! {
        #[derive(Debug, FromParameters, ToParameters)]
        pub struct #struct_name {
            #(#fields),*
        }

        impl From<#struct_name> for Resource {
            fn from(value: #struct_name) -> Self {
                let parameters: Vec<ParametersParameter> = value.into();

                Resource::Parameters(Parameters {
                    parameter: Some(parameters),
                    ..Default::default()
                })
            }
        }
    }
}

fn generate_parameters(
    parameters: &[OperationDefinitionParameter],
    parameter_use: &BoundCode<OperationParameterUse>,
    name: &str,
) -> Vec<TokenStream> {
    let parameters = parameters
        .iter()
        .filter(|parameter| parameter.use_ == *parameter_use)
        .collect::<Vec<_>>();

    generate_parameter_type(name, &parameters, true)
}

fn generate_output(parameters: &[OperationDefinitionParameter]) -> Vec<TokenStream> {
    generate_parameters(parameters, &OperationParameterUse::out(), "Output")
}

fn generate_input(parameters: &[OperationDefinitionParameter]) -> Vec<TokenStream> {
    generate_parameters(parameters, &OperationParameterUse::in_(), "Input")
}

struct OperationImports {
    resources: Vec<Ident>,
    types: Vec<Ident>,
}

impl OperationImports {
    fn new() -> Self {
        Self {
            resources: Vec::new(),
            types: vec![format_ident!("FHIRString")],
        }
    }

    fn add_resource(&mut self, name: &str) {
        self.resources.push(format_ident!("{}", name));
    }

    fn add_type(&mut self, name: &str) {
        self.types.push(format_ident!("{}", name));
    }

    fn sort_and_dedup(&mut self) {
        Self::sort_and_dedup_idents(&mut self.resources);
        Self::sort_and_dedup_idents(&mut self.types);
    }

    fn sort_and_dedup_idents(idents: &mut Vec<Ident>) {
        idents.sort_by_key(std::string::ToString::to_string);
        idents.dedup_by_key(|ident| ident.to_string());
    }
}

fn add_parameter_type(imports: &mut OperationImports, parameter: &OperationDefinitionParameter) {
    if let Some(type_) = parameter.type_.as_ref() {
        let type_name = if type_ == &AllTypes::any() {
            "Resource"
        } else {
            type_.as_str().unwrap_or_default()
        };

        if type_name == "Resource" {
            imports.add_resource("Resource");
        } else if type_name == "Element" {
            imports.add_resource("ParametersParameterValueTypeChoice");
        } else if let Some(primitive) = FHIR_PRIMITIVES.get(type_name) {
            imports.add_type(primitive);
        } else if ResourceType::try_from(type_name).is_ok() {
            imports.add_resource(type_name);
        } else {
            imports.add_type(type_name);
        }
    }

    for part in parameter.part.as_deref().unwrap_or_default() {
        add_parameter_type(imports, part);
    }
}

fn collect_imports(parameters: &[OperationDefinitionParameter]) -> OperationImports {
    let mut imports = OperationImports::new();

    imports.add_resource("Parameters");
    imports.add_resource("ParametersParameter");
    imports.add_resource("Resource");

    for parameter in parameters {
        add_parameter_type(&mut imports, parameter);
    }

    imports.sort_and_dedup();
    imports
}

/* ------------------------------------------------------------------------- */
/* Documentation generation                                                  */
/* ------------------------------------------------------------------------- */

fn format_documentation(documentation: &str) -> String {
    let mut output = String::with_capacity(documentation.len());
    let mut position = 0;

    while position < documentation.len() {
        let remaining = &documentation[position..];

        if let Some(consumed) = normalize_table(remaining, &mut output) {
            position += consumed;
            continue;
        }

        if let Some(consumed) = normalize_http_operation(remaining, &mut output) {
            position += consumed;
            continue;
        }

        if let Some(consumed) = normalize_code_span(remaining, &mut output) {
            position += consumed;
            continue;
        }

        if let Some(consumed) = normalize_markdown(remaining, &mut output) {
            position += consumed;
            continue;
        }

        if let Some(consumed) = normalize_fhir_reference(remaining, &mut output) {
            position += consumed;
            continue;
        }

        if let Some(consumed) = normalize_fhir_type_path(remaining, &mut output) {
            position += consumed;
            continue;
        }

        if let Some(consumed) = normalize_identifier(remaining, &mut output) {
            position += consumed;
            continue;
        }

        let character = remaining.chars().next().unwrap();
        output.push(character);
        position += character.len_utf8();
    }

    let documentation = normalize_single_quoted_literals(&output);
    normalize_canonical_examples(&documentation)
}

fn normalize_http_operation(documentation: &str, output: &mut String) -> Option<usize> {
    const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

    let (method, rest) = documentation.split_once(' ')?;

    if !METHODS.contains(&method) || !rest.starts_with("[base]/") {
        return None;
    }

    let path_end = rest
        .char_indices()
        .find(|(_, character)| character.is_whitespace())
        .map_or(rest.len(), |(offset, _)| offset);

    let operation_end = method.len() + 1 + path_end;

    output.push('`');
    output.push_str(&documentation[..operation_end]);
    output.push('`');

    Some(operation_end)
}

fn normalize_table(documentation: &str, output: &mut String) -> Option<usize> {
    let line_end = documentation.find('\n').unwrap_or(documentation.len());
    let line = documentation[..line_end].trim_end();

    if !(line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 2) {
        return None;
    }

    output.push_str(&normalize_table_row_contents(line));

    if line_end < documentation.len() {
        output.push('\n');
        Some(line_end + 1)
    } else {
        Some(line_end)
    }
}

fn normalize_table_row_contents(line: &str) -> String {
    let line = line.trim_end();

    if !line.starts_with('|') || !line.ends_with('|') {
        return line.to_string();
    }

    let mut cells = line.split('|').collect::<Vec<_>>();

    cells.remove(0);
    cells.pop();

    if cells.iter().all(|cell| {
        let cell = cell.trim();

        !cell.is_empty()
            && cell
                .chars()
                .all(|character| character == '-' || character == ':')
    }) {
        return line.to_string();
    }

    let normalized_cells = cells
        .into_iter()
        .map(normalize_table_cell)
        .collect::<Vec<_>>();

    format!("|{}|", normalized_cells.join("|"))
}

fn normalize_table_cell(cell: &str) -> String {
    let mut value = cell.trim();

    if value.len() >= 2 && value.starts_with('`') && value.ends_with('`') {
        value = &value[1..value.len() - 1];
    }

    let mut normalized = String::with_capacity(value.len());
    let mut position = 0;

    while position < value.len() {
        let remaining = &value[position..];

        if let Some(rest) = remaining.strip_prefix("[`")
            && let Some(end) = rest.find("`]")
        {
            normalized.push('[');
            normalized.push_str(&rest[..end]);
            normalized.push(']');

            position += 2 + end + 2;
            continue;
        }

        let character = remaining.chars().next().unwrap();

        match character {
            '`' => normalized.push(' '),
            '\'' => {}
            _ => normalized.push(character),
        }

        position += character.len_utf8();
    }

    let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

    format!("`{normalized}`")
}

fn normalize_code_span(documentation: &str, output: &mut String) -> Option<usize> {
    if documentation.starts_with("[`") {
        let code_end = documentation[1..].find('`')? + 2;

        if documentation.as_bytes().get(code_end) != Some(&b']') {
            return None;
        }

        let after_bracket = &documentation[code_end + 1..];
        let path_end = find_fhir_path_end(after_bracket)?;

        let code = &documentation[2..code_end - 1];
        let path = &after_bracket[..path_end];

        output.push('`');
        output.push_str(code);
        output.push_str(path);
        output.push('`');

        return Some(code_end + 1 + path_end);
    }

    if !documentation.starts_with('`') {
        return None;
    }

    let code_end = documentation[1..].find('`')? + 2;
    let code_span = &documentation[..code_end];
    let after_code_span = &documentation[code_end..];

    if let Some(path_end) = find_fhir_path_end(after_code_span) {
        let code = &code_span[1..code_span.len() - 1];
        let path = &after_code_span[..path_end];

        output.push('`');
        output.push_str(code);
        output.push_str(path);
        output.push('`');

        return Some(code_end + path_end);
    }

    output.push_str(code_span);
    Some(code_end)
}

fn normalize_markdown(documentation: &str, output: &mut String) -> Option<usize> {
    if !documentation.starts_with('[') {
        return None;
    }

    if let Some(consumed) = normalize_markdown_link(documentation, output) {
        return Some(consumed);
    }

    normalize_quoted_bracket_expression(documentation, output)
}

fn normalize_markdown_link(documentation: &str, output: &mut String) -> Option<usize> {
    let end = parse_markdown_link(documentation, 0)?;

    let link = &documentation[..end];
    let after_link = &documentation[end..];

    if link.starts_with("[`") {
        output.push_str(link);
        return Some(end);
    }

    if let Some(path_end) = find_fhir_path_end(after_link) {
        let target = link
            .split_once("](")
            .and_then(|(_, target)| target.strip_suffix(')'))
            .unwrap_or_default();

        if !target.starts_with("http://") && !target.starts_with("https://") {
            let close_bracket = link.find("](")?;
            let text = &link[1..close_bracket];
            let clean_text = strip_markdown_code_ticks(text);
            let path = after_link[..path_end].trim_start_matches('.');

            output.push('`');
            output.push_str(clean_text);
            output.push('.');
            output.push_str(path);
            output.push('`');

            return Some(end + path_end);
        }
    }

    let close_bracket = link.find("](")?;
    let text = &link[1..close_bracket];

    let target_start = close_bracket + 2;
    let target_end = link.rfind(')')?;
    let target = &link[target_start..target_end];

    if target.starts_with("http://") || target.starts_with("https://") {
        output.push_str(link);
        return Some(end);
    }

    let clean_text = strip_markdown_code_ticks(text);

    output.push_str("[`");
    output.push_str(clean_text);
    output.push_str("`](");
    output.push_str(target);
    output.push(')');

    Some(end)
}

fn normalize_quoted_bracket_expression(documentation: &str, output: &mut String) -> Option<usize> {
    let end = parse_quoted_bracket_expression(documentation, 0)?;

    let text = &documentation[2..end - 2];

    output.push_str("[`");
    output.push_str(text);
    output.push_str("`]");

    Some(end)
}

fn normalize_fhir_reference(documentation: &str, output: &mut String) -> Option<usize> {
    if !documentation.starts_with("http://hl7.org/fhir/") {
        return None;
    }

    let end = documentation
        .char_indices()
        .find(|(_, character)| character.is_whitespace())
        .map_or(documentation.len(), |(offset, _)| offset);

    if end == 0 {
        return None;
    }

    let reference = &documentation[..end];

    output.push('`');

    if let Some(reference) = reference.strip_suffix('.') {
        output.push_str(reference);
        output.push('`');
        output.push('.');
    } else {
        output.push_str(reference);
        output.push('`');
    }

    Some(end)
}

fn normalize_fhir_type_path(documentation: &str, output: &mut String) -> Option<usize> {
    let first = documentation.chars().next()?;

    if !first.is_alphabetic() {
        return None;
    }

    let identifier_end = find_identifier_end(documentation, 0)?;
    let identifier = &documentation[..identifier_end];

    if !(FHIR_PRIMITIVES.contains_key(identifier) || ResourceType::try_from(identifier).is_ok()) {
        return None;
    }

    let path = &documentation[identifier_end..];
    let path_end = find_fhir_path_end(path)?;

    let end = identifier_end + path_end;

    output.push('`');
    output.push_str(&documentation[..end]);
    output.push('`');

    Some(end)
}

fn normalize_identifier(documentation: &str, output: &mut String) -> Option<usize> {
    let character = documentation.chars().next()?;

    if !character.is_alphabetic() {
        return None;
    }

    let end = find_identifier_end(documentation, 0)?;
    let word = &documentation[..end];

    let mut chars = word.chars();

    let starts_uppercase = chars.next().is_some_and(|c| c.is_ascii_uppercase());

    let has_lowercase = word.chars().any(|c| c.is_ascii_lowercase());

    let has_internal_uppercase = word.chars().skip(1).any(|c| c.is_ascii_uppercase());

    if !(starts_uppercase && has_lowercase && has_internal_uppercase) {
        return None;
    }

    output.push('`');
    output.push_str(word);
    output.push('`');

    Some(end)
}

fn normalize_single_quoted_literals(documentation: &str) -> String {
    let mut output = String::with_capacity(documentation.len());

    for line in documentation.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);

        if line_without_newline.starts_with('|') && line_without_newline.ends_with('|') {
            output.push_str(line);
            continue;
        }

        let mut chars = line.char_indices().peekable();
        let mut last = 0;

        while let Some((start, character)) = chars.next() {
            if character != '\'' {
                continue;
            }

            let Some((end, _)) = chars.find(|(_, character)| *character == '\'') else {
                break;
            };

            let value = &line[start + 1..end];

            let is_simple_literal = !value.is_empty()
                && value.chars().all(|character| {
                    character.is_ascii_alphanumeric() || character == '_' || character == '-'
                });

            let preceded_by_pipe = start > 0 && line.as_bytes()[start - 1] == b'|';

            let followed_by_pipe = end + 1 < line.len() && line.as_bytes()[end + 1] == b'|';

            if is_simple_literal || (preceded_by_pipe && followed_by_pipe) {
                output.push_str(&line[last..start]);
                output.push('`');
                output.push_str(value);
                output.push('`');

                last = end + 1;
            }
        }

        output.push_str(&line[last..]);
    }

    output
}

fn normalize_canonical_examples(documentation: &str) -> String {
    const PREFIX: &str = "[system]|[version] - e.g. ";
    const URL_PREFIX: &str = "http://";

    let Some(start) = documentation.find(PREFIX) else {
        return documentation.to_string();
    };

    let url_start = start + PREFIX.len();

    let Some(relative_url_start) = documentation[url_start..].find(URL_PREFIX) else {
        return documentation.to_string();
    };

    let url_start = url_start + relative_url_start;

    let url_end = documentation[url_start..]
        .find(char::is_whitespace)
        .map_or(documentation.len(), |offset| url_start + offset);

    let mut value_end = url_end;

    if documentation.as_bytes().get(value_end.wrapping_sub(1)) == Some(&b'.') {
        value_end -= 1;
    }

    let mut output = String::with_capacity(documentation.len() + 2);

    output.push_str(&documentation[..start]);
    output.push('`');
    output.push_str(&documentation[start..value_end]);
    output.push('`');

    if value_end < url_end {
        output.push('.');
    }

    output.push_str(&documentation[url_end..]);

    output
}

fn find_fhir_path_end(documentation: &str) -> Option<usize> {
    if !documentation.starts_with('.') {
        return None;
    }

    let end = documentation
        .char_indices()
        .skip(1)
        .find(|(_, character)| {
            !(character.is_ascii_alphanumeric() || *character == '.' || *character == '_')
        })
        .map_or(documentation.len(), |(offset, _)| offset);

    (end > 1).then_some(end)
}

fn parse_markdown_link(documentation: &str, start: usize) -> Option<usize> {
    if !documentation[start..].starts_with('[') {
        return None;
    }

    let close_bracket = find_unescaped_character(documentation, start + 1, ']')?;

    if documentation.as_bytes().get(close_bracket + 1) != Some(&b'(') {
        return None;
    }

    let close_paren = find_unescaped_character(documentation, close_bracket + 2, ')')?;

    Some(close_paren + 1)
}

fn strip_markdown_code_ticks(text: &str) -> &str {
    let text = text.trim();

    if text.len() >= 4 && text.starts_with("``") && text.ends_with("``") {
        &text[2..text.len() - 2]
    } else if text.len() >= 2 && text.starts_with('`') && text.ends_with('`') {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

fn parse_quoted_bracket_expression(documentation: &str, start: usize) -> Option<usize> {
    let bytes = documentation.as_bytes();

    if bytes.get(start) != Some(&b'[') || bytes.get(start + 1) != Some(&b'\'') {
        return None;
    }

    let content_start = start + 2;
    let quote_end = documentation[content_start..].find('\'')?;
    let quote_end = content_start + quote_end;

    if bytes.get(quote_end + 1) != Some(&b']') {
        return None;
    }

    Some(quote_end + 2)
}

fn find_unescaped_character(documentation: &str, start: usize, target: char) -> Option<usize> {
    documentation[start..]
        .char_indices()
        .find_map(|(offset, character)| (character == target).then_some(start + offset))
}

fn find_identifier_end(documentation: &str, start: usize) -> Option<usize> {
    let mut end = start;

    for (offset, character) in documentation[start..].char_indices() {
        if character.is_whitespace()
            || matches!(
                character,
                '`' | '[' | ']' | '(' | ')' | ',' | '.' | ':' | ';'
            )
        {
            break;
        }

        end = start + offset + character.len_utf8();
    }

    (end > start).then_some(end)
}

fn generate_doc_attributes(documentation: &str) -> TokenStream {
    const WIDTH: usize = 100;

    let lines = documentation
        .lines()
        .flat_map(|line| wrap_documentation_line(line, WIDTH))
        .collect::<Vec<_>>();

    quote! {
        #(
            #[doc = #lines]
        )*
    }
}

fn wrap_documentation_line(line: &str, width: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    if line.starts_with('|') {
        return vec![line.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for word in line.split_whitespace() {
        let word_width = word.chars().count();

        let required_width = if current.is_empty() {
            word_width
        } else {
            current_width + 1 + word_width
        };

        if !current.is_empty() && required_width > width {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }

        if !current.is_empty() {
            current.push(' ');
            current_width += 1;
        }

        current.push_str(word);
        current_width += word_width;
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

/* ------------------------------------------------------------------------- */
/* Operation generation                                                       */
/* ------------------------------------------------------------------------- */

fn generate_operation_definition(file_path: &Path) -> Result<TokenStream, String> {
    let resource = load::load_from_file(file_path)?;
    let op_defs = get_operation_definitions(&resource)?;

    let mut generated = quote! {};

    for op_def in op_defs {
        let name = format_ident!("{}", get_name(op_def));

        let op_code = op_def
            .code
            .value
            .as_ref()
            .expect("Operation must have a code.");

        let parameters = op_def.parameter.as_deref().unwrap_or_default();

        let operation_description = op_def
            .description
            .as_ref()
            .and_then(|description| description.value.as_deref())
            .map(format_documentation)
            .unwrap_or_default();

        let operation_doc_attributes = generate_doc_attributes(&operation_description);

        let mut imports = collect_imports(parameters);

        if name == "ActivityDefinitionDataRequirements" || name == "PlanDefinitionDataRequirements"
        {
            imports.types.retain(|ident| ident != "FHIRString");
        }

        let generated_input = generate_input(parameters);
        let generated_output = generate_output(parameters);

        let resource_imports = &imports.resources;
        let type_imports = &imports.types;

        let resource_use = if resource_imports.is_empty() {
            quote! {}
        } else {
            quote! {
                use haste_fhir_model::r4::generated::resources::{
                    #(#resource_imports),*
                };
            }
        };

        let type_use = if type_imports.is_empty() {
            quote! {}
        } else {
            quote! {
                use haste_fhir_model::r4::generated::types::{
                    #(#type_imports),*
                };
            }
        };

        generated.extend(quote! {
            #operation_doc_attributes
            pub mod #name {
                #resource_use
                #type_use

                use haste_fhir_operation_error::OperationOutcomeError;
                use haste_fhir_ops::derive::{FromParameters, ToParameters};

                pub const CODE: &str = #op_code;

                #(#generated_input)*
                #(#generated_output)*
            }
        });
    }

    Ok(generated)
}

/// Generates operation definitions from JSON files in the provided directories.
///
/// # Errors
///
/// Returns an error if an operation definition cannot be generated from one of
/// the input files.
pub fn generate_operation_definitions_from_files(file_paths: &[String]) -> Result<String, String> {
    let mut generated_code = quote! {
        #![allow(non_snake_case)]
    };

    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).sort_by_file_name().into_iter();

        for entry in walker
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.metadata().is_ok_and(|metadata| metadata.is_file()))
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "json")
            })
        {
            let generated_types = generate_operation_definition(entry.path())?;

            generated_code = quote! {
                #generated_code
                #generated_types
            };
        }
    }

    Ok(generated_code.to_string())
}
