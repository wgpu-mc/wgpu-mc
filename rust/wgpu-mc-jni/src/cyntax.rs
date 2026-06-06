use std::fmt::Write;
use codespan_reporting::files::SimpleFiles;
use cyntax_common::ast::{PreprocessingToken, Whitespace};
use cyntax_common::ctx::ParseContext;
use cyntax_common::spanned::Spanned;

fn print_tokens<'src, I: Iterator<Item = &'src Spanned<PreprocessingToken>>>(out: &mut dyn Write, ctx: &'src ParseContext, source: &'src str, tokens: I) {
    for spanned_token in tokens {
        match &spanned_token.value {
            PreprocessingToken::Identifier(identifier) => {
                write!(out, "{}", ctx.strings.resolve(*identifier).unwrap());
            }
            PreprocessingToken::BlueIdentifier(identifier) => {
                write!(out, "{}", ctx.strings.resolve(*identifier).unwrap());
            }
            PreprocessingToken::StringLiteral(string) => {
                write!(out, "{}", ctx.strings.resolve(*string).unwrap());
            }
            PreprocessingToken::CharLiteral(chars) => {
                write!(out, "'{}'", ctx.strings.resolve(*chars).unwrap());
            }
            PreprocessingToken::PPNumber(number) => {
                write!(out, "{}", ctx.strings.resolve(*number).unwrap());
            }
            PreprocessingToken::Delimited(d) => {
                print_tokens(out, ctx, source, std::iter::once(&d.opener));
                print_tokens(out, ctx, source, d.inner_tokens.iter());
                print_tokens(out, ctx, source, std::iter::once(&d.closer));
            }
            PreprocessingToken::ControlLine(inner) => {
                write!(out, "#");
                print_tokens(out, ctx, source, inner.iter());
            }
            PreprocessingToken::Whitespace(whitespace) => match whitespace {
                Whitespace::Space => write!(out, " ").unwrap(),
                Whitespace::Newline => write!(out, "\n").unwrap(),
                Whitespace::Tab => write!(out, "\t").unwrap(),
            },
            PreprocessingToken::Punctuator(punctuator) => write!(out, "{}", punctuator.to_string()).unwrap(),
        }
    }
}

pub fn preprocess(src: &str, defines: &[(&str, &str)]) -> String {
    let src: String = defines.iter().map(|(l, r)| format!("#define {l} {r}\n")).chain([src.to_string()]).collect();

    let files = SimpleFiles::new();

    let mut context = ParseContext {
        files,
        strings: Default::default(),
        current_file: 0,
    };

    let lexer = cyntax_lexer::lexer::Lexer::new(&mut context, &src);
    let tokens: Vec<_> = lexer.collect();

    let processor = cyntax_preprocessor::Preprocessor::new(&mut context, &tokens);

    let result = processor.expand().unwrap();

    let mut out = String::new();

    print_tokens(&mut out, &context, &src, result.iter());

    out
}