//! The command behind the colours of short pieces of code outside the source column: signatures in
//! the member table, the side panels and the view "Change", the code lines of the hit list. The
//! caller sends the text and a file name (for the language); nothing is read from the project
//! folder, so the read right is enough (the text is index data the caller has already).

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, RelativePath};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::consts::{
    CODE_HIGHLIGHT, HIGHLIGHT_DESCRIPTION, HIGHLIGHT_MAX_CHARS, HIGHLIGHT_MAX_PIECES,
    HIGHLIGHT_TITLE,
};
use super::explorer::ExplorerPorts;
use super::source::{TokenOut, token_outs};

/// One piece of code to colour: a file name that tells the language, and the text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Piece {
    /// Path of the file the code belongs to; only its extension counts.
    pub path: String,
    /// The code, as one line or several.
    pub text: String,
}

/// The input of `code.highlight`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HighlightInput {
    /// The pieces to colour.
    pub pieces: Vec<Piece>,
}

/// The runs of one piece; empty for plain code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PieceOut {
    /// The coloured runs; `line` counts the lines of the piece from 0.
    pub tokens: Vec<TokenOut>,
}

/// The answer of `code.highlight`: one entry per piece, in the order of the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HighlightOut {
    /// The runs of each piece.
    pub pieces: Vec<PieceOut>,
}

/// Colours pieces of code with the language packs.
pub struct CodeHighlight {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl ExplorerPorts {
    /// The runs of one piece; a piece that is too long, has no valid path or no colours is plain.
    fn colour_piece(&self, piece: &Piece) -> PieceOut {
        let plain = PieceOut { tokens: Vec::new() };
        if piece.text.chars().count() > HIGHLIGHT_MAX_CHARS {
            return plain;
        }
        let Ok(path) = RelativePath::new(piece.path.as_str()) else {
            return plain;
        };
        let lines: Vec<String> = piece.text.lines().map(str::to_owned).collect();
        PieceOut {
            tokens: token_outs(&self.highlighter.highlight(&path, &lines), &lines),
        }
    }
}

impl Command for CodeHighlight {
    type Input = HighlightInput;
    type Output = HighlightOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(CODE_HIGHLIGHT),
        title_key: HIGHLIGHT_TITLE,
        description_key: HIGHLIGHT_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<HighlightOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let pieces = request
            .input
            .pieces
            .iter()
            .enumerate()
            .map(|(index, piece)| {
                if index < HIGHLIGHT_MAX_PIECES {
                    self.ports.colour_piece(piece)
                } else {
                    PieceOut { tokens: Vec::new() }
                }
            })
            .collect();
        Ok(HighlightOut { pieces })
    }
}
