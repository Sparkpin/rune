use crate::ast;
use crate::compiler::{Compiler, Needs};
use crate::error::CompileResult;
use crate::traits::Compile;
use crate::CompileError;
use runestick::Inst;

/// Compile a unary expression.
impl Compile<(&ast::ExprUnary, Needs)> for Compiler<'_, '_> {
    fn compile(&mut self, (expr_unary, needs): (&ast::ExprUnary, Needs)) -> CompileResult<()> {
        let span = expr_unary.span();
        log::trace!("ExprUnary => {:?}", self.source.source(span));

        // NB: special unary expressions.
        if let ast::UnaryOp::BorrowRef { .. } = expr_unary.op {
            return Err(CompileError::UnsupportedRef {
                span: expr_unary.span(),
            });
        }

        self.compile((&*expr_unary.expr, Needs::Value))?;

        match expr_unary.op {
            ast::UnaryOp::Not { .. } => {
                self.asm.push(Inst::Not, span);
            }
            op => {
                return Err(CompileError::UnsupportedUnaryOp { span, op });
            }
        }

        // NB: we put it here to preserve the call in case it has side effects.
        // But if we don't need the value, then pop it from the stack.
        if !needs.value() {
            self.asm.push(Inst::Pop, span);
        }

        Ok(())
    }
}
