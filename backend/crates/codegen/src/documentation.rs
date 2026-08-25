use crate::utilities::FHIR_PRIMITIVES;
use haste_fhir_model::r4::generated::resources::ResourceType;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn format_documentation(documentation: &str) -> String {
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

pub(crate) fn generate_doc_attributes(documentation: &str) -> TokenStream {
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
