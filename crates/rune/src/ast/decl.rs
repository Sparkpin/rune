use crate::ast;
use crate::parser::Parser;
use crate::traits::{Parse, Peek};
use crate::ParseError;
use runestick::Span;

/// A declaration.
#[derive(Debug, Clone)]
pub enum Decl {
    /// A use declaration.
    DeclUse(ast::DeclUse),
    /// A function declaration.
    DeclFn(ast::DeclFn),
    /// An enum declaration.
    DeclEnum(ast::DeclEnum),
    /// A struct declaration.
    DeclStruct(ast::DeclStruct),
    /// An impl declaration.
    DeclImpl(ast::DeclImpl),
}

impl Decl {
    /// The span of the declaration.
    pub fn span(&self) -> Span {
        match self {
            Self::DeclUse(decl) => decl.span(),
            Self::DeclFn(decl) => decl.span(),
            Self::DeclEnum(decl) => decl.span(),
            Self::DeclStruct(decl) => decl.span(),
            Self::DeclImpl(decl) => decl.span(),
        }
    }

    /// Indicates if the declaration needs a semi-colon or not.
    pub fn needs_semi_colon(&self) -> bool {
        match self {
            Self::DeclUse(..) => true,
            Self::DeclFn(..) => false,
            Self::DeclEnum(..) => false,
            Self::DeclStruct(decl_struct) => decl_struct.needs_semi_colon(),
            Self::DeclImpl(..) => false,
        }
    }
}

impl Peek for Decl {
    fn peek(t1: Option<ast::Token>, _: Option<ast::Token>) -> bool {
        let t1 = match t1 {
            Some(t1) => t1,
            None => return false,
        };

        match t1.kind {
            ast::Kind::Use => true,
            ast::Kind::Enum => true,
            ast::Kind::Struct => true,
            ast::Kind::Fn => true,
            _ => false,
        }
    }
}

impl Parse for Decl {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        Ok(match parser.token_peek_eof()?.kind {
            ast::Kind::Use => Self::DeclUse(parser.parse()?),
            ast::Kind::Enum => Self::DeclEnum(parser.parse()?),
            ast::Kind::Struct => Self::DeclStruct(parser.parse()?),
            ast::Kind::Impl => Self::DeclImpl(parser.parse()?),
            _ => Self::DeclFn(parser.parse()?),
        })
    }
}
