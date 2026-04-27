use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::error::CompileError;
use crate::lexer::token::TokenKind;
use crate::parser::Parser;

pub fn expr_stmt_from(expr: Expr, has_semi: bool) -> Stmt {
    Stmt::Expr { expr, has_semi }
}

impl Parser {
    pub fn parse_expr_stmt(&mut self) -> Result<Stmt, CompileError> {
        let expr = self.parse_expr()?;
        let has_semi = self.eat(&TokenKind::Semi);
        Ok(expr_stmt_from(expr, has_semi))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{Block, Expr};
    use crate::lexer::token::{IntSuffix, Token};
    use crate::span::{FileId, Span};

    fn tok(kind: TokenKind) -> Token {
        Token::new(kind, Span::new(FileId(0), 0, 0))
    }

    fn int_tok(v: u64) -> Token {
        tok(TokenKind::IntLiteral(v, IntSuffix::I32))
    }

    fn int_value(e: &Expr) -> u64 {
        match e {
            Expr::IntLit(lit) => lit.value,
            other => panic!("expected IntLit, got {:?}", other),
        }
    }

    fn as_block(e: Expr) -> Block {
        match e {
            Expr::Block(b) => b,
            other => panic!("expected Block, got {:?}", other),
        }
    }

    #[test]
    fn semicolon_significance() {
        // `1i32 ;` → Stmt::Expr { has_semi: true }, semi consumed
        let mut p = Parser::new(vec![
            int_tok(1),
            tok(TokenKind::Semi),
            tok(TokenKind::Eof),
        ]);
        match p.parse_expr_stmt() {
            Ok(Stmt::Expr { expr, has_semi }) => {
                assert!(has_semi);
                assert_eq!(int_value(&expr), 1);
            }
            other => panic!("expected Ok(Stmt::Expr), got {:?}", other),
        }
        assert_eq!(p.pos, 2);
        assert!(p.errors.is_empty());

        // `1i32` (no semi, terminator follows) → Stmt::Expr { has_semi: false }
        let mut p = Parser::new(vec![int_tok(1), tok(TokenKind::Eof)]);
        match p.parse_expr_stmt() {
            Ok(Stmt::Expr { expr, has_semi }) => {
                assert!(!has_semi);
                assert_eq!(int_value(&expr), 1);
            }
            other => panic!("expected Ok(Stmt::Expr), got {:?}", other),
        }
        assert_eq!(p.pos, 1);
        assert!(p.errors.is_empty());
    }

    #[test]
    fn block_value_semantics() {
        // `{ 1i32 }` → tail-only, block-typed
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let b = as_block(p.parse_block().expect("parse_block"));
        assert!(b.stmts.is_empty());
        let tail = b.tail.expect("tail");
        assert_eq!(int_value(&tail), 1);
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // `{ 1i32 ; }` → trailing semi, unit-typed
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Semi),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let b = as_block(p.parse_block().expect("parse_block"));
        assert_eq!(b.stmts.len(), 1);
        match &b.stmts[0] {
            Stmt::Expr { expr, has_semi } => {
                assert!(*has_semi);
                assert_eq!(int_value(expr), 1);
            }
            other => panic!("expected Stmt::Expr, got {:?}", other),
        }
        assert!(b.tail.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // `{ }` → empty, unit-typed
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let b = as_block(p.parse_block().expect("parse_block"));
        assert!(b.stmts.is_empty());
        assert!(b.tail.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // `{ 1i32 ; 2i32 }` → stmt then tail
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Semi),
            int_tok(2),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let b = as_block(p.parse_block().expect("parse_block"));
        assert_eq!(b.stmts.len(), 1);
        match &b.stmts[0] {
            Stmt::Expr { expr, has_semi } => {
                assert!(*has_semi);
                assert_eq!(int_value(expr), 1);
            }
            other => panic!("expected Stmt::Expr, got {:?}", other),
        }
        let tail = b.tail.expect("tail");
        assert_eq!(int_value(&tail), 2);
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));

        // `{ 1i32 ; 2i32 ; }` → all semi'd, unit-typed
        let mut p = Parser::new(vec![
            tok(TokenKind::LBrace),
            int_tok(1),
            tok(TokenKind::Semi),
            int_tok(2),
            tok(TokenKind::Semi),
            tok(TokenKind::RBrace),
            tok(TokenKind::Eof),
        ]);
        let b = as_block(p.parse_block().expect("parse_block"));
        assert_eq!(b.stmts.len(), 2);
        for (i, expected) in [(0u64, 1u64), (1u64, 2u64)] {
            match &b.stmts[i as usize] {
                Stmt::Expr { expr, has_semi } => {
                    assert!(*has_semi);
                    assert_eq!(int_value(expr), expected);
                }
                other => panic!("expected Stmt::Expr, got {:?}", other),
            }
        }
        assert!(b.tail.is_none());
        assert!(p.errors.is_empty());
        assert!(matches!(p.peek(), TokenKind::Eof));
    }
}
