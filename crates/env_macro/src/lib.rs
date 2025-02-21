// SPDX-License-Identifier: MIT
// Based on the code from Alexandria library (https://github.com/keep-starknet-strange/alexandria)
// Copyright (c) 2025 Alexandria Contributors

use std::str::FromStr;

use cairo_lang_filesystem::ids::{FileKind, FileLongId, VirtualFile};
use cairo_lang_macro::{inline_macro, Diagnostic, ProcMacroResult, TokenStream};
use cairo_lang_parser::db::ParserGroup;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{ArgClause, Expr, ExprInlineMacro, WrappedArgList};
use cairo_lang_utils::{Intern, Upcast};
use num_bigint::BigInt;

/// Returns the value of an environment variable as a numeric value.
///
/// If the environment variable is not set, the macro will return a diagnostic error.
/// You can also specify a default value that will be returned if the environment variable is not set.
///
/// For example:
/// ```
/// let version: ByteArray = env!("VERSION");
/// let version: ByteArray = env!("VERSION", 1);
/// ```
#[inline_macro]
pub fn env(token_stream: TokenStream) -> ProcMacroResult {
    match expand_env_macro(token_stream) {
        Ok(token_stream) => ProcMacroResult::new(token_stream),
        Err(diagnostic) => {
            ProcMacroResult::new(TokenStream::empty()).with_diagnostics(diagnostic.into())
        }
    }
}

/// Expands the environment variable macro given the macro name, the expected type of the variable and the token stream.
/// Returns the value of the environment variable as a token stream or a diagnostic error if the variable is not set or there were parsing errors.
fn expand_env_macro(token_stream: impl ToString) -> Result<TokenStream, Diagnostic> {
    let db = SimpleParserDatabase::default();
    // Get the ExprInlineMacro object so we can use the helper functions.
    let mac = parse_inline_macro("env!", token_stream, &db);
    // Get the arguments of the macro. This macro expects a tuple as argument so we get the WrappedArgList::ParenthesizedArgList
    let macro_args = if let WrappedArgList::ParenthesizedArgList(args) = mac.arguments(db.upcast())
    {
        args.arguments(db.upcast()).elements(db.upcast())
    } else {
        vec![]
    };

    if macro_args.len() == 0 {
        return Err(Diagnostic::error("Please specify the environment variable name").into());
    }

    let env_var_name = get_env_variable_name(db.upcast(), &macro_args[0].arg_clause(db.upcast()))?;

    match std::env::var(&env_var_name) {
        Ok(val) => {
            let numeric_val = BigInt::from_str(&val).map_err(|_| {
                Diagnostic::error(&format!(
                    "Failed to parse numeric environment variable: {}",
                    val
                ))
                .into()
            })?;
            Ok(TokenStream::new(numeric_val.to_string()))
        }
        Err(_) => {
            if macro_args.len() == 2 {
                get_default_value(&db, &macro_args[1].arg_clause(db.upcast()))
            } else {
                Err(
                    Diagnostic::error(&format!("Environment variable {} not set", env_var_name))
                        .into(),
                )
            }
        }
    }
}

/// Returns an [`ExprInlineMacro`] from the text received.
/// The expected text is the macro arguments.
fn parse_inline_macro(
    macro_name: &str,
    token_stream: impl ToString,
    db: &SimpleParserDatabase,
) -> ExprInlineMacro {
    // Create a virtual file that will be parsed.
    let file = FileLongId::Virtual(VirtualFile {
        parent: None,
        name: "parser_input".into(),
        content: format!("{}{}", macro_name, token_stream.to_string()).into(),
        code_mappings: [].into(),
        kind: FileKind::Expr,
    })
    .intern(db);

    // Could fail if there was a parsing error but it shouldn't happen as the file has already
    // been parsed once to reach this macro.
    let node = db
        .file_expr_syntax(file)
        .expect("Failed to parse inline macro");

    let Expr::InlineMacro(inline_macro) = node else {
        unreachable!() // should not happen
    };

    inline_macro
}

/// Parses the second argument of the macro, which is the default value.
/// Returns the default value as a token stream or a diagnostic error if there was a parsing error.
fn get_default_value(
    db: &SimpleParserDatabase,
    arg_clause: &ArgClause,
) -> Result<TokenStream, Diagnostic> {
    let base_expr = match arg_clause {
        ArgClause::Unnamed(arg_clause) => arg_clause.value(db.upcast()),
        _ => return Err(Diagnostic::error("Expected unnamed default argument").into()),
    };

    if let Expr::Literal(base_lit) = base_expr {
        let numeric_val = base_lit
            .numeric_value(db.upcast())
            .ok_or(Diagnostic::error("Failed to parse numeric default").into())?;
        Ok(TokenStream::new(numeric_val.to_string()))
    } else {
        Err(Diagnostic::error("Expected numeric default").into())
    }
}

/// Parses the first argument of the macro, which is the environment variable name.
/// Returns the environment variable name as a string or a diagnostic error if the parsing failed.
fn get_env_variable_name(
    db: &SimpleParserDatabase,
    arg_clause: &ArgClause,
) -> Result<String, Diagnostic> {
    let base_expr = match arg_clause {
        ArgClause::Unnamed(arg_clause) => arg_clause.value(db.upcast()),
        _ => return Err(Diagnostic::error("Expected unnamed argument").into()),
    };

    if let Expr::String(base_lit) = base_expr {
        base_lit
            .string_value(db.upcast())
            .ok_or(Diagnostic::error("Failed to parse environment variable name").into())
    } else {
        Err(Diagnostic::error("Expected environment variable name").into())
    }
}
