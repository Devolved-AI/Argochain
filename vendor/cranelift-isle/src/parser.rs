//! Parser for ISLE language.

use crate::ast::*;
use crate::error::{Error, Errors, Span};
use crate::lexer::{Lexer, Pos, Token};

type Result<T> = std::result::Result<T, Errors>;

/// Parse the top-level ISLE definitions and return their AST.
pub fn parse(lexer: Lexer) -> Result<Defs> {
    let parser = Parser::new(lexer);
    parser.parse_defs()
}

/// The ISLE parser.
///
/// Takes in a lexer and creates an AST.
#[derive(Clone, Debug)]
struct Parser<'a> {
    lexer: Lexer<'a>,
}

/// Used during parsing a `(rule ...)` to encapsulate some form that
/// comes after the top-level pattern: an if-let clause, or the final
/// top-level expr.
enum IfLetOrExpr {
    IfLet(IfLet),
    Expr(Expr),
}

impl<'a> Parser<'a> {
    /// Construct a new parser from the given lexer.
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser { lexer }
    }

    fn error(&self, pos: Pos, msg: String) -> Errors {
        Errors {
            errors: vec![Error::ParseError {
                msg,
                span: Span::new_single(pos),
            }],
            filenames: self.lexer.filenames.clone(),
            file_texts: self.lexer.file_texts.clone(),
        }
    }

    fn expect<F: Fn(&Token) -> bool>(&mut self, f: F) -> Result<Token> {
        if let Some(&(pos, ref peek)) = self.lexer.peek() {
            if !f(peek) {
                return Err(self.error(pos, format!("Unexpected token {:?}", peek)));
            }
            Ok(self.lexer.next()?.unwrap().1)
        } else {
            Err(self.error(self.lexer.pos(), "Unexpected EOF".to_string()))
        }
    }

    fn eat<F: Fn(&Token) -> bool>(&mut self, f: F) -> Result<Option<Token>> {
        if let Some(&(_pos, ref peek)) = self.lexer.peek() {
            if !f(peek) {
                return Ok(None);
            }
            Ok(Some(self.lexer.next()?.unwrap().1))
        } else {
            Ok(None) // EOF
        }
    }

    fn is<F: Fn(&Token) -> bool>(&self, f: F) -> bool {
        if let Some(&(_, ref peek)) = self.lexer.peek() {
            f(peek)
        } else {
            false
        }
    }

    fn pos(&self) -> Pos {
        self.lexer
            .peek()
            .map_or_else(|| self.lexer.pos(), |(pos, _)| *pos)
    }

    fn is_lparen(&self) -> bool {
        self.is(|tok| *tok == Token::LParen)
    }
    fn is_rparen(&self) -> bool {
        self.is(|tok| *tok == Token::RParen)
    }
    fn is_at(&self) -> bool {
        self.is(|tok| *tok == Token::At)
    }
    fn is_sym(&self) -> bool {
        self.is(Token::is_sym)
    }
    fn is_int(&self) -> bool {
        self.is(Token::is_int)
    }

    fn is_const(&self) -> bool {
        self.is(|tok| match tok {
            &Token::Symbol(ref tok_s) if tok_s.starts_with("$") => true,
            _ => false,
        })
    }

    fn expect_lparen(&mut self) -> Result<()> {
        self.expect(|tok| *tok == Token::LParen).map(|_| ())
    }
    fn expect_rparen(&mut self) -> Result<()> {
        self.expect(|tok| *tok == Token::RParen).map(|_| ())
    }
    fn expect_at(&mut self) -> Result<()> {
        self.expect(|tok| *tok == Token::At).map(|_| ())
    }

    fn expect_symbol(&mut self) -> Result<String> {
        match self.expect(Token::is_sym)? {
            Token::Symbol(s) => Ok(s),
            _ => unreachable!(),
        }
    }

    fn eat_sym_str(&mut self, s: &str) -> Result<bool> {
        self.eat(|tok| match tok {
            &Token::Symbol(ref tok_s) if tok_s == s => true,
            _ => false,
        })
        .map(|token| token.is_some())
    }

    fn expect_int(&mut self) -> Result<i128> {
        match self.expect(Token::is_int)? {
            Token::Int(i) => Ok(i),
            _ => unreachable!(),
        }
    }

    fn parse_defs(mut self) -> Result<Defs> {
        let mut defs = vec![];
        while !self.lexer.eof() {
            defs.push(self.parse_def()?);
        }
        Ok(Defs {
            defs,
            filenames: self.lexer.filenames,
            file_texts: self.lexer.file_texts,
        })
    }

    fn parse_def(&mut self) -> Result<Def> {
        self.expect_lparen()?;
        let pos = self.pos();
        let def = match &self.expect_symbol()?[..] {
            "pragma" => Def::Pragma(self.parse_pragma()?),
            "type" => Def::Type(self.parse_type()?),
            "decl" => Def::Decl(self.parse_decl()?),
            "rule" => Def::Rule(self.parse_rule()?),
            "extractor" => Def::Extractor(self.parse_etor()?),
            "extern" => Def::Extern(self.parse_extern()?),
            "convert" => Def::Converter(self.parse_converter()?),
            s => {
                return Err(self.error(pos, format!("Unexpected identifier: {}", s)));
            }
        };
        self.expect_rparen()?;
        Ok(def)
    }

    fn str_to_ident(&self, pos: Pos, s: &str) -> Result<Ident> {
        let first = s
            .chars()
            .next()
            .ok_or_else(|| self.error(pos, "empty symbol".into()))?;
        if !first.is_alphabetic() && first != '_' && first != '$' {
            return Err(self.error(
                pos,
                format!("Identifier '{}' does not start with letter or _ or $", s),
            ));
        }
        if s.chars()
            .skip(1)
            .any(|c| !c.is_alphanumeric() && c != '_' && c != '.' && c != '$')
        {
            return Err(self.error(
                pos,
                format!(
                    "Identifier '{}' contains invalid character (not a-z, A-Z, 0-9, _, ., $)",
                    s
                ),
            ));
        }
        Ok(Ident(s.to_string(), pos))
    }

    fn parse_ident(&mut self) -> Result<Ident> {
        let pos = self.pos();
        let s = self.expect_symbol()?;
        self.str_to_ident(pos, &s)
    }

    fn parse_const(&mut self) -> Result<Ident> {
        let pos = self.pos();
        let ident = self.parse_ident()?;
        if ident.0.starts_with("$") {
            let s = &ident.0[1..];
            Ok(Ident(s.to_string(), ident.1))
        } else {
            Err(self.error(
                pos,
                "Not a constant identifier; must start with a '$'".to_string(),
            ))
        }
    }

    fn parse_pragma(&mut self) -> Result<Pragma> {
        let ident = self.parse_ident()?;
        // currently, no pragmas are defined, but the infrastructure is useful to keep around
        match ident.0.as_str() {
            pragma => Err(self.error(ident.1, format!("Unknown pragma '{}'", pragma))),
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        let pos = self.pos();
        let name = self.parse_ident()?;

        let mut is_extern = false;
        let mut is_nodebug = false;

        while self.lexer.peek().map_or(false, |(_pos, tok)| tok.is_sym()) {
            let sym = self.expect_symbol()?;
            if sym == "extern" {
                is_extern = true;
            } else if sym == "nodebug" {
                is_nodebug = true;
            } else {
                return Err(self.error(
                    self.pos(),
                    format!("unknown type declaration modifier: {}", sym),
                ));
            }
        }

        let ty = self.parse_typevalue()?;
        Ok(Type {
            name,
            is_extern,
            is_nodebug,
            ty,
            pos,
        })
    }

    fn parse_typevalue(&mut self) -> Result<TypeValue> {
        let pos = self.pos();
        self.expect_lparen()?;
        if self.eat_sym_str("primitive")? {
            let primitive_ident = self.parse_ident()?;
            self.expect_rparen()?;
            Ok(TypeValue::Primitive(primitive_ident, pos))
        } else if self.eat_sym_str("enum")? {
            let mut variants = vec![];
            while !self.is_rparen() {
                let variant = self.parse_type_variant()?;
                variants.push(variant);
            }
            self.expect_rparen()?;
            Ok(TypeValue::Enum(variants, pos))
        } else {
            Err(self.error(pos, "Unknown type definition".to_string()))
        }
    }

    fn parse_type_variant(&mut self) -> Result<Variant> {
        if self.is_sym() {
            let pos = self.pos();
            let name = self.parse_ident()?;
            Ok(Variant {
                name,
                fields: vec![],
                pos,
            })
        } else {
            let pos = self.pos();
            self.expect_lparen()?;
            let name = self.parse_ident()?;
            let mut fields = vec![];
            while !self.is_rparen() {
                fields.push(self.parse_type_field()?);
            }
            self.expect_rparen()?;
            Ok(Variant { name, fields, pos })
        }
    }

    fn parse_type_field(&mut self) -> Result<Field> {
        let pos = self.pos();
        self.expect_lparen()?;
        let name = self.parse_ident()?;
        let ty = self.parse_ident()?;
        self.expect_rparen()?;
        Ok(Field { name, ty, pos })
    }

    fn parse_decl(&mut self) -> Result<Decl> {
        let pos = self.pos();

        let pure = self.eat_sym_str("pure")?;
        let multi = self.eat_sym_str("multi")?;
        let partial = self.eat_sym_str("partial")?;

        let term = self.parse_ident()?;

        self.expect_lparen()?;
        let mut arg_tys = vec![];
        while !self.is_rparen() {
            arg_tys.push(self.parse_ident()?);
        }
        self.expect_rparen()?;

        let ret_ty = self.parse_ident()?;

        Ok(Decl {
            term,
            arg_tys,
            ret_ty,
            pure,
            multi,
            partial,
            pos,
        })
    }

    fn parse_extern(&mut self) -> Result<Extern> {
        let pos = self.pos();
        if self.eat_sym_str("constructor")? {
            let term = self.parse_ident()?;
            let func = self.parse_ident()?;
            Ok(Extern::Constructor { term, func, pos })
        } else if self.eat_sym_str("extractor")? {
            let infallible = self.eat_sym_str("infallible")?;

            let term = self.parse_ident()?;
            let func = self.parse_ident()?;

            Ok(Extern::Extractor {
                term,
                func,
                pos,
                infallible,
            })
        } else if self.eat_sym_str("const")? {
            let pos = self.pos();
            let name = self.parse_const()?;
            let ty = self.parse_ident()?;
            Ok(Extern::Const { name, ty, pos })
        } else {
            Err(self.error(
                pos,
                "Invalid extern: must be (extern constructor ...) or (extern extractor ...)"
                    .to_string(),
            ))
        }
    }

    fn parse_etor(&mut self) -> Result<Extractor> {
        let pos = self.pos();
        self.expect_lparen()?;
        let term = self.parse_ident()?;
        let mut args = vec![];
        while !self.is_rparen() {
            args.push(self.parse_ident()?);
        }
        self.expect_rparen()?;
        let template = self.parse_pattern()?;
        Ok(Extractor {
            term,
            args,
            template,
            pos,
        })
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        let pos = self.pos();
        let prio = if self.is_int() {
            Some(
                i64::try_from(self.expect_int()?)
                    .map_err(|err| self.error(pos, format!("Invalid rule priority: {}", err)))?,
            )
        } else {
            None
        };
        let pattern = self.parse_pattern()?;
        let mut iflets = vec![];
        loop {
            match self.parse_iflet_or_expr()? {
                IfLetOrExpr::IfLet(iflet) => {
                    iflets.push(iflet);
                }
                IfLetOrExpr::Expr(expr) => {
                    return Ok(Rule {
                        pattern,
                        iflets,
                        expr,
                        pos,
                        prio,
                    });
                }
            }
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        let pos = self.pos();
        if self.is_int() {
            Ok(Pattern::ConstInt {
                val: self.expect_int()?,
                pos,
            })
        } else if self.is_const() {
            let val = self.parse_const()?;
            Ok(Pattern::ConstPrim { val, pos })
        } else if self.eat_sym_str("_")? {
            Ok(Pattern::Wildcard { pos })
        } else if self.is_sym() {
            let var = self.parse_ident()?;
            if self.is_at() {
                self.expect_at()?;
                let subpat = Box::new(self.parse_pattern()?);
                Ok(Pattern::BindPattern { var, subpat, pos })
            } else {
                Ok(Pattern::Var { var, pos })
            }
        } else if self.is_lparen() {
            self.expect_lparen()?;
            if self.eat_sym_str("and")? {
                let mut subpats = vec![];
                while !self.is_rparen() {
                    subpats.push(self.parse_pattern()?);
                }
                self.expect_rparen()?;
                Ok(Pattern::And { subpats, pos })
            } else {
                let sym = self.parse_ident()?;
                let mut args = vec![];
                while !self.is_rparen() {
                    args.push(self.parse_pattern()?);
                }
                self.expect_rparen()?;
                Ok(Pattern::Term { sym, args, pos })
            }
        } else {
            Err(self.error(pos, "Unexpected pattern".into()))
        }
    }

    fn parse_iflet_or_expr(&mut self) -> Result<IfLetOrExpr> {
        let pos = self.pos();
        if self.is_lparen() {
            self.expect_lparen()?;
            let ret = if self.eat_sym_str("if-let")? {
                IfLetOrExpr::IfLet(self.parse_iflet()?)
            } else if self.eat_sym_str("if")? {
                // Shorthand form: `(if (x))` desugars to `(if-let _
                // (x))`.
                IfLetOrExpr::IfLet(self.parse_iflet_if()?)
            } else {
                IfLetOrExpr::Expr(self.parse_expr_inner_parens(pos)?)
            };
            self.expect_rparen()?;
            Ok(ret)
        } else {
            self.parse_expr().map(|expr| IfLetOrExpr::Expr(expr))
        }
    }

    fn parse_iflet(&mut self) -> Result<IfLet> {
        let pos = self.pos();
        let pattern = self.parse_pattern()?;
        let expr = self.parse_expr()?;
        Ok(IfLet { pattern, expr, pos })
    }

    fn parse_iflet_if(&mut self) -> Result<IfLet> {
        let pos = self.pos();
        let expr = self.parse_expr()?;
        Ok(IfLet {
            pattern: Pattern::Wildcard { pos },
            expr,
            pos,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        let pos = self.pos();
        if self.is_lparen() {
            self.expect_lparen()?;
            let ret = self.parse_expr_inner_parens(pos)?;
            self.expect_rparen()?;
            Ok(ret)
        } else if self.eat_sym_str("#t")? {
            Ok(Expr::ConstInt { val: 1, pos })
        } else if self.eat_sym_str("#f")? {
            Ok(Expr::ConstInt { val: 0, pos })
        } else if self.is_const() {
            let val = self.parse_const()?;
            Ok(Expr::ConstPrim { val, pos })
        } else if self.is_sym() {
            let name = self.parse_ident()?;
            Ok(Expr::Var { name, pos })
        } else if self.is_int() {
            let val = self.expect_int()?;
            Ok(Expr::ConstInt { val, pos })
        } else {
            Err(self.error(pos, "Invalid expression".into()))
        }
    }

    fn parse_expr_inner_parens(&mut self, pos: Pos) -> Result<Expr> {
        if self.eat_sym_str("let")? {
            self.expect_lparen()?;
            let mut defs = vec![];
            while !self.is_rparen() {
                let def = self.parse_letdef()?;
                defs.push(def);
            }
            self.expect_rparen()?;
            let body = Box::new(self.parse_expr()?);
            Ok(Expr::Let { defs, body, pos })
        } else {
            let sym = self.parse_ident()?;
            let mut args = vec![];
            while !self.is_rparen() {
                args.push(self.parse_expr()?);
            }
            Ok(Expr::Term { sym, args, pos })
        }
    }

    fn parse_letdef(&mut self) -> Result<LetDef> {
        let pos = self.pos();
        self.expect_lparen()?;
        let var = self.parse_ident()?;
        let ty = self.parse_ident()?;
        let val = Box::new(self.parse_expr()?);
        self.expect_rparen()?;
        Ok(LetDef { var, ty, val, pos })
    }

    fn parse_converter(&mut self) -> Result<Converter> {
        let pos = self.pos();
        let inner_ty = self.parse_ident()?;
        let outer_ty = self.parse_ident()?;
        let term = self.parse_ident()?;
        Ok(Converter {
            term,
            inner_ty,
            outer_ty,
            pos,
        })
    }
}
