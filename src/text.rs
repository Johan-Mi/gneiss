use lsp_types::{InitializeParams, Position, PositionEncodingKind};
use ropey::Rope;
use std::borrow::Cow;
use tree_sitter::{Node, Point};

#[derive(Clone, Copy, Default)]
pub enum PositionEncoding {
    Utf8,
    #[default]
    Utf16,
    Utf32,
}

impl TryFrom<PositionEncodingKind> for PositionEncoding {
    type Error = ();

    fn try_from(value: PositionEncodingKind) -> Result<Self, Self::Error> {
        if value == PositionEncodingKind::UTF8 {
            Ok(Self::Utf8)
        } else if value == PositionEncodingKind::UTF16 {
            Ok(Self::Utf16)
        } else if value == PositionEncodingKind::UTF32 {
            Ok(Self::Utf32)
        } else {
            Err(())
        }
    }
}

impl From<PositionEncoding> for PositionEncodingKind {
    fn from(value: PositionEncoding) -> Self {
        match value {
            PositionEncoding::Utf8 => Self::UTF8,
            PositionEncoding::Utf16 => Self::UTF16,
            PositionEncoding::Utf32 => Self::UTF32,
        }
    }
}

impl From<&InitializeParams> for PositionEncoding {
    fn from(params: &InitializeParams) -> Self {
        params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| {
                let encodings = g.position_encodings.as_ref()?;
                // Prefer UTF-8
                if encodings.contains(&PositionEncodingKind::UTF8) {
                    Some(PositionEncodingKind::UTF8)
                } else {
                    encodings.first().cloned()
                }
            })
            .map(Self::try_from)
            .and_then(Result::ok)
            .unwrap_or_default()
    }
}

impl PositionEncoding {
    pub fn position_to_byte(self, text: &Rope, pos: Position) -> usize {
        text.line_to_byte(pos.line as usize)
            + match self {
                Self::Utf8 => pos.character as usize,
                Self::Utf16 => {
                    let line = text.line(pos.line as usize);
                    line.char_to_byte(
                        line.utf16_cu_to_char(pos.character as usize),
                    )
                }
                Self::Utf32 => text
                    .line(pos.line as usize)
                    .char_to_byte(pos.character as usize),
            }
    }

    pub fn position_to_point(self, text: &Rope, pos: Position) -> Point {
        // FIXME: This gives incorrect results for characters with a width other
        // than 1 cell. I think. The Tree-sitter documentation is not very clear
        // about what a column is.
        Point {
            row: pos.line as usize,
            column: match self {
                Self::Utf8 => text
                    .line(pos.line as usize)
                    .byte_to_char(pos.character as usize),
                Self::Utf16 => text
                    .line(pos.line as usize)
                    .utf16_cu_to_char(pos.character as usize),
                Self::Utf32 => pos.character as usize,
            },
        }
    }

    pub fn byte_to_position(self, text: &Rope, byte: usize) -> Position {
        let line = text.byte_to_line(byte);
        let byte_offset_on_line = byte - text.line_to_byte(line);
        let character = match self {
            Self::Utf8 => byte_offset_on_line,
            Self::Utf16 => text
                .line(line)
                .byte_slice(..byte_offset_on_line)
                .len_utf16_cu(),
            Self::Utf32 => text.line(line).byte_to_char(byte_offset_on_line),
        } as u32;
        Position {
            line: line as u32,
            character,
        }
    }
}

pub fn byte_to_point(text: &Rope, byte: usize) -> Point {
    // FIXME: This gives incorrect results for characters with a width other
    // than 1 cell. I think. The Tree-sitter documentation is not very clear
    // about what a column is.
    let row = text.byte_to_line(byte);
    let line_start = text.line_to_byte(row);
    let column = text.line(row).byte_to_char(byte - line_start);
    Point { row, column }
}

pub fn node_text<'a>(node: Node, text: &'a Rope) -> Cow<'a, str> {
    text.byte_slice(node.byte_range()).into()
}
