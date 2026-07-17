use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::io::Read;

const START_BLOCK: u8 = 0x0B;
const END_BLOCK: u8 = 0x1C;
const CARRIAGE_RETURN: u8 = 0x0D;
const COMMIT_ACK: u8 = 0x06;
const COMMIT_NAK: u8 = 0x15;

pub struct MllpFormatter;

impl MllpFormatter {
    pub fn encode(payload: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(payload.len() + 3);
        buf.push(START_BLOCK);
        buf.extend_from_slice(payload);
        buf.push(END_BLOCK);
        buf.push(CARRIAGE_RETURN);
        buf
    }

    pub fn decode(framed: &[u8]) -> Result<&[u8], OperationOutcomeError> {
        if framed.len() < 4
            || framed[0] != START_BLOCK
            || framed[framed.len() - 2] != END_BLOCK
            || framed[framed.len() - 1] != CARRIAGE_RETURN
        {
            let k = String::from_utf8_lossy(framed);
            return Err(OperationOutcomeError::error(
                IssueType::EXCEPTION,
                format!("Expected MLLP frame <SB>...<EB><CR>, got: {:?}", k),
            ));
        }
        Ok(&framed[1..framed.len() - 2])
    }

    pub fn ack() -> [u8; 4] {
        [START_BLOCK, COMMIT_ACK, END_BLOCK, CARRIAGE_RETURN]
    }

    pub fn nak() -> [u8; 4] {
        [START_BLOCK, COMMIT_NAK, END_BLOCK, CARRIAGE_RETURN]
    }

    pub fn is_ack(bytes: &[u8]) -> bool {
        bytes == Self::ack()
    }

    pub fn is_nak(bytes: &[u8]) -> bool {
        bytes == Self::nak()
    }

    /// Reads a single MLLP frame from `reader`, returning the raw framed bytes
    /// (including the START_BLOCK / END_BLOCK / CARRIAGE_RETURN wrappers).
    /// Returns an error on EOF mid-frame or an I/O error.
    pub fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, OperationOutcomeError> {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match reader.read(&mut byte) {
                Ok(0) => {
                    return Err(OperationOutcomeError::error(
                        IssueType::EXCEPTION,
                        "Connection closed before complete MLLP frame".to_string(),
                    ));
                }
                Ok(_) => {
                    buf.push(byte[0]);
                    let len = buf.len();

                    if buf[0] != START_BLOCK {
                        return Err(OperationOutcomeError::error(
                            IssueType::EXCEPTION,
                            "MLLP frame does not start with START_BLOCK".to_string(),
                        ));
                    }

                    if len >= 2 && buf[len - 2] == END_BLOCK && buf[len - 1] == CARRIAGE_RETURN {
                        return Ok(buf);
                    }
                }
                Err(e) => {
                    return Err(OperationOutcomeError::error(
                        IssueType::EXCEPTION,
                        format!("Failed to read from stream: {}", e),
                    ));
                }
            }
        }
    }
}
