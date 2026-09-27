//! Async MLLP framing, for a listener that serves many senders at once.
//!
//! [`crate::mllp::MllpFormatter::read_frame`] reads a byte at a time from a
//! blocking [`std::io::Read`], which is fine for a client sending one message
//! and wrong for a listener: it holds a thread per connection and cannot be
//! cancelled, so one sender that opens a socket and stops writing occupies that
//! thread until the process restarts.
//!
//! This reader is cancel-safe (a timeout around it drops cleanly), buffers
//! rather than syscalling per byte, and refuses a frame past
//! [`MAX_FRAME_BYTES`] instead of growing until the host runs out of memory.

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use tokio::io::{AsyncRead, AsyncReadExt};

const START_BLOCK: u8 = 0x0B;
const END_BLOCK: u8 = 0x1C;
const CARRIAGE_RETURN: u8 = 0x0D;

/// Largest message this listener accepts, in bytes.
///
/// A sender that never writes `<EB>` would otherwise grow the buffer forever.
/// 16 MiB is far above any real HL7 v2 message a large one with embedded
/// documents runs to a few hundred KiB so this rejects only a broken or
/// hostile sender.
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// Why a frame read ended.
#[derive(Debug)]
pub enum FrameEnd {
    /// A complete frame; the payload with framing bytes removed.
    Frame(Vec<u8>),
    /// The sender closed cleanly between frames. Not an error: a sender that
    /// sends one message and hangs up is behaving correctly.
    Closed,
}

/// Reads one MLLP frame from `reader`.
///
/// Returns [`FrameEnd::Closed`] when the stream ends *between* frames, and an
/// error when it ends *within* one, since a truncated message must not be
/// mistaken for a complete one.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] when the stream does not begin with
/// `<SB>`, the frame exceeds [`MAX_FRAME_BYTES`], the stream ends mid-frame, or
/// the underlying read fails.
pub async fn read_frame<R>(reader: &mut R) -> Result<FrameEnd, OperationOutcomeError>
where
    R: AsyncRead + Unpin,
{
    // Leading bytes before `<SB>` are a framing error, not something to skip:
    // silently resynchronising would accept a corrupted stream as valid.
    let first = match reader.read_u8().await {
        Ok(byte) => byte,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(FrameEnd::Closed),
        Err(e) => return Err(read_error(&e)),
    };

    if first != START_BLOCK {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            format!("Expected an MLLP frame to start with <SB> (0x0B), got 0x{first:02X}"),
        ));
    }

    let mut payload = Vec::with_capacity(4096);
    // `<EB>` is only a terminator when `<CR>` follows it, and a payload may
    // legitimately contain 0x1C, so the previous byte has to be remembered.
    let mut pending_end_block = false;

    loop {
        let byte = match reader.read_u8().await {
            Ok(byte) => byte,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(OperationOutcomeError::error(
                    IssueType::incomplete(),
                    format!(
                        "Stream ended after {} byte(s) without <EB><CR>; \
                         a truncated message is not acknowledged",
                        payload.len()
                    ),
                ));
            }
            Err(e) => return Err(read_error(&e)),
        };

        if pending_end_block {
            if byte == CARRIAGE_RETURN {
                return Ok(FrameEnd::Frame(payload));
            }
            // The 0x1C was payload after all.
            payload.push(END_BLOCK);
            pending_end_block = false;
        }

        if byte == END_BLOCK {
            pending_end_block = true;
            continue;
        }

        payload.push(byte);

        if payload.len() > MAX_FRAME_BYTES {
            return Err(OperationOutcomeError::error(
                IssueType::too_costly(),
                format!("MLLP frame exceeded the {MAX_FRAME_BYTES} byte limit"),
            ));
        }
    }
}

fn read_error(error: &std::io::Error) -> OperationOutcomeError {
    OperationOutcomeError::error(
        IssueType::exception(),
        format!("Failed to read from the MLLP stream: {error}"),
    )
}

/// Wraps `payload` in `<SB>`/`<EB><CR>`.
#[must_use]
pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let mut framed = Vec::with_capacity(payload.len() + 3);
    framed.push(START_BLOCK);
    framed.extend_from_slice(payload);
    framed.push(END_BLOCK);
    framed.push(CARRIAGE_RETURN);
    framed
}
