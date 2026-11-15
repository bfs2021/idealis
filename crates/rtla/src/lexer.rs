use crate::ast::Location;
use crate::{ErrorKind, ParseError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Name,
    Keyword,
    Number,
    Decimal,
    String,
    Symbol,
    End,
}
#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub text: String,
    pub raw: String,
    pub kind: Kind,
    pub location: Location,
}

const KEYWORDS: &[&str] = &[
    "ASSUME",
    "ASSUMPTION",
    "AXIOM",
    "BY",
    "CASE",
    "CHOOSE",
    "CONSTANT",
    "CONSTANTS",
    "COROLLARY",
    "DEF",
    "DOMAIN",
    "ELSE",
    "ENABLED",
    "EXCEPT",
    "EXTENDS",
    "IF",
    "IN",
    "INSTANCE",
    "LAMBDA",
    "LEMMA",
    "LET",
    "LOCAL",
    "MODULE",
    "NEW",
    "OBVIOUS",
    "OMITTED",
    "PROOF",
    "PROPOSITION",
    "PROVE",
    "RECURSIVE",
    "SUBSET",
    "THEN",
    "THEOREM",
    "UNCHANGED",
    "UNION",
    "USE",
    "HIDE",
    "VARIABLE",
    "VARIABLES",
    "WITH",
];
const SYMBOLS: &[&str] = &[
    "-+->", "<=>", "|->", ">>_", "...", "::=", "^+", "^*", "^#", "=>", "==", "<<", ">>", "<=",
    ">=", "=<", "/\\", "\\/", "/=", "[]", "]_", "<>", "->", "<-", "~>", "..", "::", ":=", ":>",
    "<:", "@@", "++", "--", "**", "//", "^^", "||", "&&", "%%", "##", "$$", "??", "!!", "=", "<",
    ">", "/", "[", "]", "(", ")", "{", "}", "|", "-", "~", ".", "+", "*", "^", "%", ",", ":", "!",
    "@", "'", "_", "\\", "&", "$", "?",
];
const ALIASES: &[(&str, &str)] = &[
    ("\\in", "\\in"),
    ("\\notin", "\\notin"),
    ("\\subseteq", "\\subseteq"),
    ("\\cup", "\\cup"),
    ("\\union", "\\cup"),
    ("\\cap", "\\cap"),
    ("\\intersect", "\\cap"),
    ("\\X", "\\X"),
    ("\\times", "\\X"),
    ("\\o", "\\o"),
    ("\\circ", "\\o"),
    ("\\div", "\\div"),
    ("\\land", "/\\"),
    ("\\lor", "\\/"),
    ("\\lnot", "~"),
    ("\\neg", "~"),
    ("\\equiv", "<=>"),
    ("\\leq", "<="),
    ("\\geq", ">="),
    ("\\A", "\\A"),
    ("\\forall", "\\A"),
    ("\\E", "\\E"),
    ("\\exists", "\\E"),
    ("\\AA", "\\AA"),
    ("\\EE", "\\EE"),
    ("\\cdot", "\\cdot"),
    ("∈", "\\in"),
    ("∉", "\\notin"),
    ("⊆", "\\subseteq"),
    ("∪", "\\cup"),
    ("∩", "\\cap"),
    ("∀", "\\A"),
    ("∃", "\\E"),
    ("∧", "/\\"),
    ("∨", "\\/"),
    ("¬", "~"),
    ("≤", "<="),
    ("≥", ">="),
    ("≠", "#"),
    ("⇒", "=>"),
    ("⇔", "<=>"),
    ("□", "[]"),
    ("◇", "<>"),
    ("×", "\\X"),
    ("⟨", "<<"),
    ("⟩", ">>"),
    ("#", "#"),
];

struct Scanner<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
    column: usize,
}
impl Scanner<'_> {
    fn location(&self) -> Location {
        Location {
            start: self.offset,
            end: self.offset,
            line: self.line,
            column: self.column,
        }
    }
    fn advance(&mut self, bytes: usize) {
        for character in self.source[self.offset..self.offset + bytes].chars() {
            if character == '\n' {
                self.line += 1;
                self.column = 1;
            } else if character == '\t' {
                self.column += 8 - (self.column - 1) % 8;
            } else {
                self.column += character.len_utf16();
            }
        }
        self.offset += bytes;
    }
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            kind: ErrorKind::Syntax,
            message: message.into(),
            location: self.location(),
        }
    }
}

pub(crate) fn tokenize(source: &str, start: usize, module: bool) -> Result<Vec<Token>, ParseError> {
    let mut scanner = Scanner {
        source,
        offset: 0,
        line: 1,
        column: 1,
    };
    scanner.advance(start);
    let mut tokens = vec![];
    while scanner.offset < source.len() {
        let rest = &source[scanner.offset..];
        let character = rest.chars().next().expect("nonempty source suffix");
        if character.is_whitespace() {
            scanner.advance(character.len_utf8());
            continue;
        }
        if rest.starts_with("\\*") {
            scanner.advance(rest.find('\n').unwrap_or(rest.len()));
            continue;
        }
        if rest.starts_with("(*") {
            let start = scanner.location();
            let mut depth = 1;
            scanner.advance(2);
            while depth > 0 {
                let rest = &source[scanner.offset..];
                if rest.is_empty() {
                    return Err(ParseError {
                        kind: ErrorKind::Syntax,
                        message: "Unterminated comment".into(),
                        location: start,
                    });
                }
                if rest.starts_with("(*") {
                    depth += 1;
                    scanner.advance(2);
                } else if rest.starts_with("*)") {
                    depth -= 1;
                    scanner.advance(2);
                } else {
                    scanner.advance(rest.chars().next().unwrap().len_utf8());
                }
            }
            continue;
        }
        let mut location = scanner.location();
        let (text, kind, length) = if character == '"' {
            scanner.advance(1);
            let mut decoded = String::new();
            loop {
                let rest = &source[scanner.offset..];
                let Some(character) = rest.chars().next() else {
                    return Err(ParseError {
                        kind: ErrorKind::Syntax,
                        message: "Unterminated string".into(),
                        location,
                    });
                };
                if character == '"' {
                    scanner.advance(1);
                    break;
                }
                if character == '\n' || character == '\r' {
                    return Err(scanner.error("Newline in string"));
                }
                if character == '\\' {
                    scanner.advance(1);
                    let escape = source[scanner.offset..]
                        .chars()
                        .next()
                        .ok_or_else(|| scanner.error("Unterminated escape"))?;
                    decoded.push(match escape {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'f' => '\u{c}',
                        '"' => '"',
                        '\\' => '\\',
                        _ => return Err(scanner.error("Invalid string escape")),
                    });
                    scanner.advance(escape.len_utf8());
                } else {
                    decoded.push(character);
                    scanner.advance(character.len_utf8());
                }
            }
            (decoded, Kind::String, 0)
        } else if rest.starts_with("----") || rest.starts_with("====") {
            let length = rest
                .bytes()
                .take_while(|byte| *byte == character as u8)
                .count();
            (rest[..4].into(), Kind::Symbol, length)
        } else if character.is_ascii_alphanumeric() || character == '_' {
            let length = if rest.starts_with("WF_") || rest.starts_with("SF_") {
                3
            } else {
                rest.bytes()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    .count()
            };
            let word = &rest[..length];
            if word.bytes().all(|byte| byte.is_ascii_digit()) {
                if rest[length..].starts_with('.')
                    && !rest[length..].starts_with("..")
                    && rest
                        .as_bytes()
                        .get(length + 1)
                        .is_some_and(u8::is_ascii_digit)
                {
                    let fraction = rest[length + 1..]
                        .bytes()
                        .take_while(u8::is_ascii_digit)
                        .count();
                    let end = length + 1 + fraction;
                    (rest[..end].into(), Kind::Decimal, end)
                } else {
                    (word.into(), Kind::Number, length)
                }
            } else {
                let kind = if KEYWORDS.contains(&word) {
                    Kind::Keyword
                } else if word == "_" || word == "WF_" || word == "SF_" {
                    Kind::Symbol
                } else {
                    Kind::Name
                };
                (word.into(), kind, length)
            }
        } else if character == '\\'
            && rest
                .as_bytes()
                .get(1)
                .is_some_and(|byte| matches!(byte, b'b' | b'B' | b'o' | b'O' | b'h' | b'H'))
            && rest.as_bytes().get(2).is_some_and(|byte| {
                let radix = match rest.as_bytes()[1].to_ascii_lowercase() {
                    b'b' => 2,
                    b'o' => 8,
                    _ => 16,
                };
                char::from(*byte).is_digit(radix)
            })
        {
            let radix = match rest.as_bytes()[1].to_ascii_lowercase() {
                b'b' => 2,
                b'o' => 8,
                _ => 16,
            };
            let length = 2 + rest[2..]
                .chars()
                .take_while(|character| character.is_digit(radix))
                .map(char::len_utf8)
                .sum::<usize>();
            if length == 2 {
                return Err(scanner.error("Missing based numeral digits"));
            }
            (rest[..length].into(), Kind::Number, length)
        } else if let Some((spelling, text)) = ALIASES
            .iter()
            .filter(|(spelling, _)| rest.starts_with(spelling))
            .max_by_key(|(spelling, _)| spelling.len())
        {
            ((*text).into(), Kind::Symbol, spelling.len())
        } else if let Some(symbol) = SYMBOLS.iter().find(|symbol| rest.starts_with(**symbol)) {
            let text = match *symbol {
                "/=" => "#",
                "=<" => "<=",
                _ => symbol,
            };
            (text.into(), Kind::Symbol, symbol.len())
        } else {
            return Err(scanner.error("Unrecognized character"));
        };
        scanner.advance(length);
        location.end = scanner.offset;
        let raw = source[location.start..location.end].into();
        let is_module_end = text == "====";
        tokens.push(Token {
            text,
            raw,
            kind,
            location,
        });
        if module && is_module_end {
            break;
        }
    }
    tokens.push(Token {
        text: String::new(),
        raw: String::new(),
        kind: Kind::End,
        location: scanner.location(),
    });
    Ok(tokens)
}
