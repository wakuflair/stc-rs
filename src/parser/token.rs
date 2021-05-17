use crate::parser::*;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub enum Tok {
    /// '.'
    Access,
    /// '+'
    Plus,
    /// '-'
    Minus,
    /// '*'
    Multiply,
    /// '/'
    Division,
    /// '('
    LeftParentheses,
    /// ')'
    RightParentheses,
    /// ','
    Comma,
    /// ';'
    Semicolon,
    /// ':'
    Colon,
    /// ':='
    Assign,
    /// '='
    Equal,
    /// '<>'
    NotEqual,
    /// '>'
    Greater,
    /// '>='
    GreaterEqual,
    /// '<'
    Less,
    /// '<='
    LessEqual,
    /// '|'
    BitOr,
    /// '&'
    BitAnd,
    /// '^'
    Deref,
    /// 'POINTER'
    Pointer,
    /// 'ARRAY'
    Array,
    /// 'OF'
    Of,
    /// 'IF'
    If,
    /// 'THEN'
    Then,
    /// 'ELSE'
    Else,
    /// 'ELSEIF'
    ElseIf,
    /// 'END_IF'
    EndIf,
    /// 'TO'
    To,
    /// 'FUNCTION'
    Function,
    /// 'END_FUNCTION'
    EndFunction,
    /// 'PROGRAM'
    Program,
    /// 'END_PROGRAM'
    EndProgram,
    /// 'FUNCTION_BLOCK'
    FunctionBlock,
    /// 'END_FUNCTION_BLOCK'
    EndFunctionBlock,
    /// 'VAR'
    Var,
    /// 'VAR_GLOBAL'
    VarGlobal,
    /// 'VAR_INPUT'
    VarInput,
    /// 'VAR_INOUT'
    VarInOut,
    /// 'VAR_OUTPUT'
    VarOutput,
    /// 'VAR_TEMP'
    VarTemp,
    /// 'VAR_STAT'
    VarStat,
    /// 'END_VAR'
    EndVar,
    /// 'RETAIN'
    Retain,
    /// 'PERSISTENT'
    Persistent,
    /// 'BIT', one bit type
    Bit,
    /// 'BOOL', boolean type
    Bool,
    /// 'SINT', 8 bits signed
    SInt,
    /// 'BYTE', 8 bits unsigned
    Byte,
    /// 'INT', 16 bits signed
    Int,
    /// 'UINT', 16 bits unsigned
    UInt,
    /// 'DINT', 32 bits signed
    DInt,
    /// 'LINT', 64 bits signed
    LInt,
    /// 'ULINT', 64 bits unsigned
    ULInt,
    /// 'REAL', 32 bits signed
    Real,
    /// 'LREAL', 64 bits unsigned
    LReal,
    /// 'TIME' 32 bits time
    Time,
    /// 'LTIME' 64 bits time
    LTime,
    /// 'STRING', string type
    String,
    /// Literal
    Literal(LiteralType),
    /// Identifier
    Identifier(StString),
}

impl Into<String> for &Tok {
    fn into(self) -> String {
        let s = match &self {
            Tok::Access => ".",
            Tok::Plus => "+",
            Tok::Minus => "-",
            Tok::Multiply => "*",
            Tok::Division => "/",
            Tok::LeftParentheses => "(",
            Tok::RightParentheses => ")",
            Tok::Comma => ",",
            Tok::Semicolon => ";",
            Tok::Colon => ":",
            Tok::Assign => ":=",
            Tok::Equal => "=",
            Tok::NotEqual => "<>",
            Tok::Greater => ">",
            Tok::GreaterEqual => ">=",
            Tok::Less => "<",
            Tok::LessEqual => "<=",
            Tok::BitOr => "|",
            Tok::BitAnd => "&",
            Tok::Deref => "^",
            Tok::Pointer => "POINTER",
            Tok::Array => "ARRAY",
            Tok::Of => "OF",
            Tok::To => "TO",
            Tok::If => "IF",
            Tok::Then => "THEN",
            Tok::Else => "ELSE",
            Tok::ElseIf => "ELSEIF",
            Tok::EndIf => "END_IF",
            Tok::Function => "FUNCTION",
            Tok::EndFunction => "END_FUNCTION",
            Tok::Program => "PROGRAM",
            Tok::EndProgram => "END_PROGRAM",
            Tok::FunctionBlock => "FUNCTION_BLOCK",
            Tok::EndFunctionBlock => "END_FUNCTION_BLOCK",
            Tok::VarGlobal => "VAR_GLOBAL",
            Tok::Var => "VAR",
            Tok::VarInput => "VAR_INPUT",
            Tok::VarInOut => "VAR_INOUT",
            Tok::VarOutput => "VAR_OUTPUT",
            Tok::VarTemp => "VAR_TEMP",
            Tok::VarStat => "VAR_STAT",
            Tok::EndVar => "END_VAR",
            Tok::Retain => "RETAIN",
            Tok::Persistent => "PERSISTENT",
            Tok::Int => "INT",
            Tok::Real => "REAL",
            Tok::LReal => "LREAL",
            Tok::Bit => "BIT",
            Tok::Bool => "BOOL",
            Tok::SInt => "SINT",
            Tok::Byte => "BYTE",
            Tok::UInt => "UINT",
            Tok::DInt => "DINT",
            Tok::LInt => "LINT",
            Tok::ULInt => "ULINT",
            Tok::Time => "TIME",
            Tok::LTime => "LTIME",
            Tok::String => "STRING",
            Tok::Literal(_) => unimplemented!(),
            Tok::Identifier(s) => &s.origin_string(),
        };

        s.to_owned()
    }
}

impl Into<StString> for Tok {
    fn into(self) -> StString {
        let s: String = (&self).into();

        s.into()
    }
}

impl Display for Tok {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<String>::into(self))
    }
}