//! APML AST.
//!
//! This AST structure is designed to correspond byte by byte
//! to the source file in order to obtain a complete reverse
//! conversion capability to the source file.

use std::{borrow::Cow, rc::Rc};

/// A APML abstract-syntax-tree, consisting of a list of tokens.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApmlAst<'a>(pub Vec<Token<'a>>);

impl ToString for ApmlAst<'_> {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// A token in the AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token<'a> {
    /// A space character (`' '`, ASCII code 0x20).
    Space,
    /// A newline character (`'\n'`, ASCII code 0x0A).
    Newline,
    /// A comment (`"#<text>"`).
    Comment(Cow<'a, str>),
    /// A variable definition (`"<name>=<value>"`).
    Variable(VariableDefinition<'a>),
}

impl ToString for Token<'_> {
    fn to_string(&self) -> String {
        match self {
            Token::Space => " ".to_string(),
            Token::Newline => "\n".to_string(),
            Token::Comment(text) => format!("#{}", text),
            Token::Variable(def) => def.to_string(),
        }
    }
}

/// A variable definition (`"<name>=<value>"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableDefinition<'a> {
    /// Name of the variable.
    pub name: Cow<'a, str>,
    /// Value of the variable.
    pub value: VariableValue<'a>,
}

impl ToString for VariableDefinition<'_> {
    fn to_string(&self) -> String {
        format!("{}={}", self.name, self.value.to_string())
    }
}

/// Value of a variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableValue<'a> {
    /// A string value (`"<text>"`).
    String(Rc<Text<'a>>),
}

impl ToString for VariableValue<'_> {
    fn to_string(&self) -> String {
        match self {
            VariableValue::String(text) => text.to_string(),
        }
    }
}

/// A section of text.
///
/// Text is made up of several text units.
/// For example:
/// - `abc'123'` is made up of an unquoted unit `abc` and a single-quoted unit `123`.
/// - `"abc$0"` is made up of one double-quoted unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text<'a>(pub Vec<TextUnit<'a>>);

impl ToString for Text<'_> {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|unit| unit.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// A unit of text.
///
/// See [Text] and [Word] for more documentation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextUnit<'a> {
    /// An unquoted text unit (`"<words>"`).
    Unquoted(Vec<Word<'a>>),
    /// A single-quoted text unit (`"'<text>'"`).
    SingleQuote(Cow<'a, str>),
    /// A double-quoted text unit (`"\"<words>\""`).
    DuobleQuote(Vec<Word<'a>>),
}

impl ToString for TextUnit<'_> {
    fn to_string(&self) -> String {
        match self {
            TextUnit::Unquoted(words) => words
                .iter()
                .map(|word| word.to_string())
                .collect::<Vec<_>>()
                .join(""),
            TextUnit::SingleQuote(text) => text.to_string(),
            TextUnit::DuobleQuote(words) => format!(
                "\"{}\"",
                words
                    .iter()
                    .map(|word| word.to_string())
                    .collect::<Vec<_>>()
                    .join("")
            ),
        }
    }
}

/// A word is a part of a text unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Word<'a> {
    /// A literal string (`"<parts>"`)
    Literal(Vec<LiteralPart<'a>>),
    /// An unbraced variable expansion (`"$<var>"`).
    UnbracedVariable(Cow<'a, str>),
    /// A braced variable expansion (`"${<expansion>}"`).
    BracedVariable(BracedExpansion<'a>),
}

impl ToString for Word<'_> {
    fn to_string(&self) -> String {
        match self {
            Word::Literal(parts) => parts
                .iter()
                .map(|part| part.to_string())
                .collect::<Vec<_>>()
                .join(""),
            Word::UnbracedVariable(name) => format!("${}", name),
            Word::BracedVariable(exp) => format!("${{{}}}", exp.to_string()),
        }
    }
}

/// A element of literal words.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralPart<'a> {
    /// A string (`"<text>"`).
    String(Cow<'a, str>),
    /// An escaped character (`"\\<char>"`).
    Escaped(char),
    /// A tag for discard newlines (`"\\\n"`).
    LineContinuation,
}

impl ToString for LiteralPart<'_> {
    fn to_string(&self) -> String {
        match self {
            LiteralPart::String(text) => text.to_string(),
            LiteralPart::Escaped(ch) => format!("\\{}", ch),
            LiteralPart::LineContinuation => "\\\n".to_string(),
        }
    }
}

/// A braced variable expansion (`"<name>[modifier]"`).
///
/// Note that for [ExpansionModifier::Length], the format is `"#<name>"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BracedExpansion<'a> {
    /// Name of the variable.
    pub name: Cow<'a, str>,
    /// Modifier to apply to the expanded value.
    pub modifier: Option<ExpansionModifier<'a>>,
}

impl ToString for BracedExpansion<'_> {
    fn to_string(&self) -> String {
        match &self.modifier {
            Some(ExpansionModifier::Length) => format!("#{}", self.name),
            None => self.name.to_string(),
            Some(modifier) => format!("{}{}", self.name, modifier.to_string()),
        }
    }
}

/// A modifier in the braced variable expansion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpansionModifier<'a> {
    /// Reference to a substring (`":offset"` or `":offset:length"`).
    ///
    /// The range is [offset, (offset+length)) (indexing from zero).
    Substring {
        /// Offset.
        offset: usize,
        /// Length.
        length: Option<usize>,
    },
    /// Stripping the shortest matching prefix (`"#<pattern>"`).
    StripShortestPrefix(Rc<GlobPattern<'a>>),
    /// Stripping the longest matching prefix (`"##<pattern>"`).
    StripLongestPrefix(Rc<GlobPattern<'a>>),
    /// Stripping the shortest matching suffix (`"%<pattern>"`).
    StripShortestSuffix(Rc<GlobPattern<'a>>),
    /// Stripping the longest matching suffix (`"%%<pattern>"`).
    StripLongestSuffix(Rc<GlobPattern<'a>>),
    /// Replacing the first match of a pattern with a text (`"/<pattern>/<string>"`).
    ReplaceOnce {
        pattern: Rc<GlobPattern<'a>>,
        string: Rc<Text<'a>>,
    },
    /// Replacing the all matches of a pattern with a text (`"//<pattern>/<string>"`).
    ReplaceAll {
        pattern: Rc<GlobPattern<'a>>,
        string: Rc<Text<'a>>,
    },
    /// Replacing the prefix of a pattern with a text (`"/#<pattern>/<string>"`).
    ReplacePrefix {
        pattern: Rc<GlobPattern<'a>>,
        string: Rc<Text<'a>>,
    },
    /// Replacing the suffix of a pattern with a text (`"/%<pattern>/<string>"`).
    ReplaceSuffix {
        pattern: Rc<GlobPattern<'a>>,
        string: Rc<Text<'a>>,
    },
    /// Upper-casify the first match of a pattern (`"^<pattern>"`).
    UpperOnce(Rc<GlobPattern<'a>>),
    /// Upper-casify the all matches of a pattern (`"^^<pattern>"`).
    UpperAll(Rc<GlobPattern<'a>>),
    /// Lower-casify the first match of a pattern (`",<pattern>"`).
    LowerOnce(Rc<GlobPattern<'a>>),
    /// Lower-casify the all matches of a pattern (`",,<pattern>"`).
    LowerAll(Rc<GlobPattern<'a>>),
    /// Producing errors when the variable is unset or null (`":?<text>"`).
    ErrorOnUnset(Rc<Text<'a>>),
    /// Returning the length of the variable.
    ///
    /// Note that this modifier uses a special format, see [BracedExpansion].
    Length,
    /// Returning a text when the variable is unset or null (`":-<text>"`).
    WhenUnset(Rc<Text<'a>>),
    /// Returning a text when the variable is set (`":+<text>"`).
    WhenSet(Rc<Text<'a>>),
}

impl ToString for ExpansionModifier<'_> {
    fn to_string(&self) -> String {
        match self {
            ExpansionModifier::Substring { offset, length } => match length {
                None => format!(":{}", offset),
                Some(length) => format!(":{}:{}", offset, length),
            },
            ExpansionModifier::StripShortestPrefix(pattern) => format!("#{}", pattern.to_string()),
            ExpansionModifier::StripLongestPrefix(pattern) => format!("##{}", pattern.to_string()),
            ExpansionModifier::StripShortestSuffix(pattern) => format!("%{}", pattern.to_string()),
            ExpansionModifier::StripLongestSuffix(pattern) => format!("%%{}", pattern.to_string()),
            ExpansionModifier::ReplaceOnce { pattern, string } => {
                format!("/{}/{}", pattern.to_string(), string.to_string())
            }
            ExpansionModifier::ReplaceAll { pattern, string } => {
                format!("//{}/{}", pattern.to_string(), string.to_string())
            }
            ExpansionModifier::ReplacePrefix { pattern, string } => {
                format!("/#{}/{}", pattern.to_string(), string.to_string())
            }
            ExpansionModifier::ReplaceSuffix { pattern, string } => {
                format!("/%{}/{}", pattern.to_string(), string.to_string())
            }
            ExpansionModifier::UpperOnce(pattern) => format!("^{}", pattern.to_string()),
            ExpansionModifier::UpperAll(pattern) => format!("^^{}", pattern.to_string()),
            ExpansionModifier::LowerOnce(pattern) => format!(",{}", pattern.to_string()),
            ExpansionModifier::LowerAll(pattern) => format!(",,{}", pattern.to_string()),
            ExpansionModifier::ErrorOnUnset(text) => format!(":?{}", text.to_string()),
            ExpansionModifier::Length => unreachable!(),
            ExpansionModifier::WhenUnset(text) => format!(":-{}", text.to_string()),
            ExpansionModifier::WhenSet(text) => format!(":+{}", text.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobPattern<'a>(pub Vec<GlobPart<'a>>);

impl ToString for GlobPattern<'_> {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// A element of glob patterns.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GlobPart<'a> {
    /// Matches a fixed string (`"<text>"`).
    String(Cow<'a, str>),
    /// Matches an escaped character (`"\\<char>"`).
    Escaped(char),
    /// Matches any string (`'*'`).
    AnyString,
    /// Matches any single character (`'?'`).
    AnyChar,
    /// Matches a characters range (`"[<range>]"`).
    Range(Cow<'a, str>),
}

impl ToString for GlobPart<'_> {
    fn to_string(&self) -> String {
        match self {
            GlobPart::String(text) => text.to_string(),
            GlobPart::Escaped(ch) => format!("\\{}", ch),
            GlobPart::AnyString => "*".to_string(),
            GlobPart::AnyChar => "?".to_string(),
            GlobPart::Range(range) => format!("[{}]", range),
        }
    }
}