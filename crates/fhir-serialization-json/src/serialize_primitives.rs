use crate::traits::FHIRJSONSerializer;

impl FHIRJSONSerializer for i64 {
    fn serialize_value(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize_extension(&self) -> Option<String> {
        None
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        let mut name = "\"".to_string();
        name.push_str(field);
        name.push_str("\":");
        Some(name + &self.serialize_value()?)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONSerializer for u64 {
    fn serialize_value(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize_extension(&self) -> Option<String> {
        None
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        let mut name = "\"".to_string();
        name.push_str(field);
        name.push_str("\":");
        Some(name + &self.serialize_value()?)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONSerializer for f64 {
    fn serialize_value(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize_extension(&self) -> Option<String> {
        None
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        let mut name = "\"".to_string();
        name.push_str(field);
        name.push_str("\":");
        Some(name + &self.serialize_value()?)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONSerializer for bool {
    fn serialize_value(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize_extension(&self) -> Option<String> {
        None
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        let mut name = "\"".to_string();
        name.push_str(field);
        name.push_str("\":");
        Some(name + &self.serialize_value()?)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

// https://datatracker.ietf.org/doc/html/rfc7159#section-7
// char = unescaped /
//           escape (
//               %x22 /          ; "    quotation mark  U+0022
//               %x5C /          ; \    reverse solidus U+005C
//               %x2F /          ; /    solidus         U+002F

//               %x62 /          ; b    backspace       U+0008
//               %x66 /          ; f    form feed       U+000C
//               %x6E /          ; n    line feed       U+000A
//               %x72 /          ; r    carriage return U+000D
//               %x74 /          ; t    tab             U+0009
//               %x75 4HEXDIG )  ; uXXXX                U+XXXX
impl FHIRJSONSerializer for String {
    fn serialize_value(&self) -> Option<String> {
        let mut char_arr: Vec<char> = Vec::with_capacity(self.len() + 2);
        for c in self.chars() {
            match c {
                // Simple escapes just reuse character
                // | '\u{005C}' | '\u{002F}'
                '\u{0022}' => {
                    char_arr.push('\x5c');
                    char_arr.push(c);
                }
                // Backspace
                '\u{0008}' => {
                    char_arr.push('\x5c');
                    char_arr.push('\x62');
                }
                // Form feed
                '\u{000C}' => {
                    char_arr.push('\x5c');
                    char_arr.push('\x66');
                }
                // Line Feed
                '\u{000A}' => {
                    char_arr.push('\x5c');
                    char_arr.push('\x6e');
                }
                // Carriage return
                '\u{000D}' => {
                    char_arr.push('\x5c');
                    char_arr.push('\x72');
                }
                // Tab
                '\u{0009}' => {
                    char_arr.push('\x5c');
                    char_arr.push('\x74');
                }

                ch => char_arr.push(ch),
            }
        }

        let k: String = char_arr.into_iter().collect();

        Some("\"".to_string() + &k + "\"")
    }

    fn serialize_extension(&self) -> Option<String> {
        None
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        let mut name = "\"".to_string();
        name.push_str(field);
        name.push_str("\":");
        Some(name + &self.serialize_value()?)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl<T> FHIRJSONSerializer for Vec<T>
where
    T: FHIRJSONSerializer,
{
    fn serialize_value(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let mut total = 0;
        let mut string_value = "[".to_string();

        string_value.push_str(
            &self
                .iter()
                .map(|v| {
                    if let Some(string_value) = v.serialize_value() {
                        total += 1;
                        string_value
                    } else {
                        "null".to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(","),
        );

        string_value.push_str("]");

        if total > 0 { Some(string_value) } else { None }
    }

    fn serialize_extension(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        if self[0].is_fp_primitive() {
            let mut total = 0;
            let mut string_value = "[".to_string();
            string_value.push_str(
                &self
                    .iter()
                    .map(|v| {
                        if let Some(extension_string) = v.serialize_extension() {
                            total += 1;
                            extension_string
                        } else {
                            "null".to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(","),
            );

            string_value.push_str("]");

            if total > 0 { Some(string_value) } else { None }
        } else {
            None
        }
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        let extension_value = self.serialize_extension();
        let value = self.serialize_value();

        let mut result = Vec::with_capacity(2);

        if let Some(extension_value) = extension_value {
            let mut ext_string = "\"_".to_string();
            ext_string.push_str(field);
            ext_string.push_str("\":");
            ext_string.push_str(&extension_value);
            result.push(ext_string);
        }

        if let Some(value) = value {
            let mut value_string = "\"".to_string();
            value_string.push_str(field);
            value_string.push_str("\":");
            value_string.push_str(&value);
            result.push(value_string);
        }

        if result.is_empty() {
            None
        } else {
            Some(result.join(","))
        }
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl<T> FHIRJSONSerializer for Option<T>
where
    T: FHIRJSONSerializer,
{
    fn serialize_value(&self) -> Option<String> {
        self.as_ref().and_then(|v| v.serialize_value())
    }

    fn serialize_extension(&self) -> Option<String> {
        self.as_ref().and_then(|v| v.serialize_extension())
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        self.as_ref().and_then(|v| v.serialize_field(field))
    }

    fn is_fp_primitive(&self) -> bool {
        match self {
            Some(v) => v.is_fp_primitive(),
            None => false,
        }
    }
}

impl<T> FHIRJSONSerializer for Box<T>
where
    T: FHIRJSONSerializer,
{
    fn serialize_value(&self) -> Option<String> {
        self.as_ref().serialize_value()
    }

    fn serialize_extension(&self) -> Option<String> {
        self.as_ref().serialize_extension()
    }

    fn serialize_field(&self, field: &str) -> Option<String> {
        self.as_ref().serialize_field(field)
    }

    fn is_fp_primitive(&self) -> bool {
        self.as_ref().is_fp_primitive()
    }
}
