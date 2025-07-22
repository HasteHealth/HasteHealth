use std::io::{BufWriter, Write};

use crate::{SerializeError, traits::FHIRJSONSerializer};

impl FHIRJSONSerializer for i64 {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(self.to_string().as_bytes())?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write("\"".as_bytes())?;
        writer.write(field.as_bytes())?;
        writer.write("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONSerializer for u64 {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(self.to_string().as_bytes())?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write("\"".as_bytes())?;
        writer.write(field.as_bytes())?;
        writer.write("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONSerializer for f64 {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(self.to_string().as_bytes())?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write("\"".as_bytes())?;
        writer.write(field.as_bytes())?;
        writer.write("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONSerializer for bool {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(self.to_string().as_bytes())?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write("\"".as_bytes())?;
        writer.write(field.as_bytes())?;
        writer.write("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
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
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write(&[b'"'])?;
        for c in self.chars() {
            match c {
                // Simple escapes just reuse character
                // | '\u{005C}' | '\u{002F}'
                '\u{0022}' => {
                    writer.write(&[b'\x5c', c as u8])?;
                }
                // Backspace
                '\u{0008}' => {
                    writer.write(&[b'\x5c', b'\x62'])?;
                }
                // Form feed
                '\u{000C}' => {
                    writer.write(&[b'\x5c', b'\x66'])?;
                }
                // Line Feed
                '\u{000A}' => {
                    writer.write(&[b'\x5c', b'\x6e'])?;
                }
                // Carriage return
                '\u{000D}' => {
                    writer.write(&[b'\x5c', b'\x72'])?;
                }
                // Tab
                '\u{0009}' => {
                    writer.write(&[b'\x5c', b'\x74'])?;
                }

                ch => {
                    writer.write(&[ch as u8])?;
                }
            }
        }

        writer.write(&[b'"'])?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write(&[b'"'])?;
        writer.write(field.as_bytes())?;
        writer.write(&[b'"', b':'])?;
        self.serialize_value(writer)?;
        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl<T> FHIRJSONSerializer for Vec<T>
where
    T: FHIRJSONSerializer,
{
    fn serialize_value(&self, _writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        if self.is_empty() {
            return Ok(false);
        }

        let mut total = 0;

        let mut tmp_buffer = BufWriter::new(Vec::new());
        tmp_buffer.write(&[b'['])?;

        for i in 0..(self.len() - 1) {
            let v = &self[i];
            if v.serialize_value(&mut tmp_buffer)? {
                total += 1;
                tmp_buffer.write(&[b','])?;
            } else {
                tmp_buffer.write("null".as_bytes())?;
            }
        }

        // Last one don't want trailing comma.
        if self[self.len() - 1].serialize_value(&mut tmp_buffer)? {
            total += 1;
        }

        tmp_buffer.write(&[b']'])?;
        Ok(total > 0)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        if self.is_empty() {
            return Ok(false);
        }

        if self[0].is_fp_primitive() {
            let mut total = 0;

            let mut tmp_buffer = BufWriter::new(Vec::new());
            tmp_buffer.write(&[b'['])?;

            for i in 0..(self.len() - 1) {
                let v = &self[i];
                if v.serialize_extension(&mut tmp_buffer)? {
                    total += 1;
                    tmp_buffer.write(&[b','])?;
                } else {
                    tmp_buffer.write("null".as_bytes())?;
                }
            }

            // Last one don't want trailing comma.
            if self[self.len() - 1].serialize_extension(&mut tmp_buffer)? {
                total += 1;
            }

            tmp_buffer.write(&[b']'])?;
            Ok(total > 0)
        } else {
            Ok(false)
        }
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        let mut extension_buffer = BufWriter::new(Vec::new());
        let mut value_buffer = BufWriter::new(Vec::new());

        let should_serialize_extension = self.serialize_extension(&mut extension_buffer)?;
        let shoud_serialize_value = self.serialize_value(&mut value_buffer)?;

        let value_u8 = value_buffer.into_inner()?;
        let extension_u8 = extension_buffer.into_inner()?;

        if should_serialize_extension {
            writer.write(&[b'"', b'_'])?;
            writer.write(field.as_bytes())?;
            writer.write(&[b'"', b':'])?;
            writer.write(&extension_u8)?;
            // If value not empty put trailing comma after extension value.
            if shoud_serialize_value {
                writer.write(&[b','])?;
            }
        }

        if shoud_serialize_value {
            writer.write(&[b'"'])?;
            writer.write(field.as_bytes())?;
            writer.write(&[b'"', b':'])?;
            writer.write(&value_u8)?;
        }

        Ok(shoud_serialize_value || should_serialize_extension)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl<T> FHIRJSONSerializer for Option<T>
where
    T: FHIRJSONSerializer,
{
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        match self {
            Some(v) => v.serialize_value(writer),
            None => Ok(false),
        }
    }

    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        match self {
            Some(v) => v.serialize_extension(writer),
            None => Ok(false),
        }
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        match self {
            Some(v) => v.serialize_field(field, writer),
            None => Ok(false),
        }
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
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        self.as_ref().serialize_value(writer)
    }

    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        self.as_ref().serialize_extension(writer)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        self.as_ref().serialize_field(field, writer)
    }

    fn is_fp_primitive(&self) -> bool {
        self.as_ref().is_fp_primitive()
    }
}
