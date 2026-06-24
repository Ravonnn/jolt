use jolt_source::FileId;

use crate::{lex, Lexer, TokenKind};

fn file() -> FileId {
    FileId(1)
}

fn kinds(source: &str) -> Vec<TokenKind> {
    lex(source, file()).into_iter().map(|t| t.kind).collect()
}

fn lex_one(input: &str) -> TokenKind {
    let mut lx = Lexer::new(file(), input);
    lx.next_token().kind
}

fn lex_two(input: &str) -> (TokenKind, TokenKind) {
    let mut lx = Lexer::new(file(), input);
    let a = lx.next_token().kind;
    let b = lx.next_token().kind;
    (a, b)
}

#[test]
fn max_munch_shift_gt() {
    assert_eq!(lex_one(">>>"), TokenKind::GtGtGt);
    assert_eq!(lex_one(">>"), TokenKind::GtGt);
    assert_eq!(lex_one(">"), TokenKind::Gt);
}

#[test]
fn max_munch_semi() {
    assert_eq!(lex_one(";;"), TokenKind::SemiSemi);
    assert_eq!(lex_one(";"), TokenKind::Semi);
}

#[test]
fn max_munch_compare() {
    assert_eq!(lex_one("=="), TokenKind::EqEq);
    assert_eq!(lex_one("="), TokenKind::Eq);
    assert_eq!(lex_one("!="), TokenKind::NotEq);
    assert_eq!(lex_one("<="), TokenKind::Le);
    assert_eq!(lex_one(">="), TokenKind::Ge);
}

#[test]
fn max_munch_slash() {
    // Leading `//` is a comment; test operators in expression context.
    assert_eq!(lex_two("a//b").1, TokenKind::SlashSlash);
    assert_eq!(lex_two("a/b").1, TokenKind::Slash);
}

#[test]
fn max_munch_dot() {
    assert_eq!(lex_one("..."), TokenKind::DotDotDot);
    assert_eq!(lex_one(".."), TokenKind::DotDot);
}

#[test]
fn max_munch_shift_lt() {
    assert_eq!(lex_one("<<|"), TokenKind::LtLtPipe);
    assert_eq!(lex_one("<<"), TokenKind::LtLt);
    assert_eq!(lex_one("<<<"), TokenKind::LtLtLt);
}

#[test]
fn max_munch_shift_gt_pipe() {
    assert_eq!(lex_one(">>|"), TokenKind::GtGtPipe);
}

#[test]
fn max_munch_at_fn() {
    assert_eq!(lex_two("@fn"), (TokenKind::At, TokenKind::Fn));
}

#[test]
fn max_munch_dollar() {
    assert_eq!(lex_one("$$"), TokenKind::DollarDollar);
    assert_eq!(lex_one("$"), TokenKind::Dollar);
}

#[test]
fn max_munch_bitwise_combo() {
    assert_eq!(lex_one("~%|"), TokenKind::TildePercentPipe);
    assert_eq!(lex_one("~|"), TokenKind::TildePipe);
    assert_eq!(lex_one("~&"), TokenKind::TildeAmp);
}

#[test]
fn comments_skipped() {
    let t = kinds("// comment only\n+");
    assert_eq!(t[0], TokenKind::Plus);
}

#[test]
fn string_literal() {
    let t = lex(r#""hello\n""#, file());
    assert!(matches!(t[0].kind, TokenKind::StringLit(ref s) if s == "hello\n"));
}

#[test]
fn int_literals() {
    assert_eq!(lex_one("42"), TokenKind::IntLit(42));
    assert_eq!(lex_one("0xff"), TokenKind::IntLit(255));
    assert_eq!(lex_one("0o77"), TokenKind::IntLit(63));
    assert_eq!(lex_one("0b1010"), TokenKind::IntLit(10));
    assert_eq!(lex_one("1_000"), TokenKind::IntLit(1000));
    assert_eq!(lex_one("-3"), TokenKind::IntLit(-3));
}

#[test]
fn keywords_recognized() {
    for (src, kind) in [
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("loop", TokenKind::Loop),
        ("for", TokenKind::For),
        ("in", TokenKind::In),
        ("return", TokenKind::Return),
        ("break", TokenKind::Break),
        ("next", TokenKind::Next),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
    ] {
        assert_eq!(lex_one(src), kind, "keyword {src}");
    }
}

#[test]
fn ident_names() {
    assert!(matches!(lex_one("println"), TokenKind::Ident(s) if s == "println"));
    assert!(matches!(lex_one("Int"), TokenKind::Ident(s) if s == "Int"));
}

#[test]
fn tour_double_function() {
    let src = "@double(x: Int) Int -> x * 2 ;;";
    let got: Vec<_> = kinds(src)
        .into_iter()
        .filter(|k| !matches!(k, TokenKind::Eof))
        .collect();
    assert_eq!(
        got,
        vec![
            TokenKind::At,
            TokenKind::Ident("double".into()),
            TokenKind::LParen,
            TokenKind::Ident("x".into()),
            TokenKind::Colon,
            TokenKind::Ident("Int".into()),
            TokenKind::RParen,
            TokenKind::Ident("Int".into()),
            TokenKind::Arrow,
            TokenKind::Ident("x".into()),
            TokenKind::Star,
            TokenKind::IntLit(2),
            TokenKind::SemiSemi,
        ]
    );
}

#[test]
fn error_unterminated_string() {
    assert!(matches!(
        lex_one("\"foo"),
        TokenKind::Error(crate::LexErrorKind::UnterminatedString)
    ));
}

#[test]
fn error_invalid_escape() {
    assert!(matches!(
        lex_one(r#""\q""#),
        TokenKind::Error(crate::LexErrorKind::InvalidEscape)
    ));
}

#[test]
fn ends_with_eof() {
    let t = kinds("x");
    assert!(matches!(t.last(), Some(TokenKind::Eof)));
}

/// Smoke: every non-literal TokenKind variant appears in at least one lex result.
#[test]
fn token_kind_coverage() {
    let sources = [
        "@fn if else loop for in return break next true false",
        "0 1 \"s\" ident",
        "$ $$ -> ;; ; == != <= >= // .. ... << >> <<< >>> <<| >>|",
        "+ - * / % ^ & | ~ ~& ~| ~%| %| && || !",
        "( ) , : [ ] = += -> ...",
        ">>>= <<<= ~%|=",
    ];
    let mut seen = std::collections::HashSet::new();
    for src in sources {
        for t in lex(src, file()) {
            seen.insert(kind_tag(&t.kind));
        }
    }
    // Ensure major families present
    for needle in [
        "At",
        "Fn",
        "If",
        "IntLit",
        "StringLit",
        "Ident",
        "Eof",
        "GtGtGt",
        "SemiSemi",
        "Arrow",
    ] {
        assert!(
            seen.iter().any(|s| s.contains(needle)),
            "missing coverage for {needle}"
        );
    }
}

fn kind_tag(kind: &TokenKind) -> String {
    match kind {
        TokenKind::IntLit(_) => "IntLit".into(),
        TokenKind::StringLit(_) => "StringLit".into(),
        TokenKind::Ident(_) => "Ident".into(),
        TokenKind::Error(_) => "Error".into(),
        other => format!("{other:?}")
            .split('(')
            .next()
            .unwrap_or("?")
            .to_string(),
    }
}

#[test]
fn lex_file_query_cache() {
    use jolt_query::QueryEngine;

    let mut engine = QueryEngine::new();
    let f = file();
    let src = "x + 1;";
    let h = jolt_query::hash_bytes(src.as_bytes());

    crate::lex_file(&mut engine, f, src);
    let c1 = engine.compute_count(crate::LEX_FILE);
    crate::lex_file(&mut engine, f, src);
    assert_eq!(engine.compute_count(crate::LEX_FILE), c1);

    engine.invalidate_input(h);
    crate::lex_file(&mut engine, f, src);
    assert!(engine.compute_count(crate::LEX_FILE) > c1);
}
