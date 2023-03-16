use lsp_types::{InitializeParams, PositionEncodingKind};
use ropey::Rope;
use std::borrow::Cow;
use tree_sitter::Node;

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

pub fn node_text<'a>(node: Node, text: &'a Rope) -> Cow<'a, str> {
    text.byte_slice(node.byte_range()).into()
}
