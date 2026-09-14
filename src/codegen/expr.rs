use crate::Result;
use crate::codegen::codegen::Codegen;
use crate::frontend::ast::ExprAST;
use inkwell::{
    FloatPredicate,
    values::{BasicValueEnum, ValueKind},
};

impl<'ctx> Codegen<'ctx> {
    fn compile_number(&mut self, n: f64) -> Result<BasicValueEnum<'ctx>> {
        Ok(self.context.f64_type().const_float(n).into())
    }

    fn compile_variable(&mut self, name: &str) -> Result<BasicValueEnum<'ctx>> {
        self.named_values
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown variable: {}", name).into())
    }

    fn compile_binary(
        &mut self,
        op: char,
        lhs: &ExprAST,
        rhs: &ExprAST,
    ) -> Result<BasicValueEnum<'ctx>> {
        let x = self.compile_expr(lhs)?.into_float_value();
        let y = self.compile_expr(rhs)?.into_float_value();
        let value = match op {
            '+' => self.builder.build_float_add(x, y, "addtmp")?.into(),
            '-' => self.builder.build_float_sub(x, y, "subtmp")?.into(),
            '*' => self.builder.build_float_mul(x, y, "multmp")?.into(),
            '/' => self.builder.build_float_div(x, y, "divtmp")?.into(),
            '<' => {
                let cmp = self
                    .builder
                    .build_float_compare(FloatPredicate::ULT, x, y, "cmptmp")?;
                let bool_as_float = self.builder.build_unsigned_int_to_float(
                    cmp,
                    self.context.f64_type(),
                    "booltmp",
                )?;
                bool_as_float.into()
            }
            _ => return Err(format!("unknown operator: `{}`", op).into()),
        };
        Ok(BasicValueEnum::FloatValue(value))
    }

    fn compile_call(&mut self, callee: &str, args: &[ExprAST]) -> Result<BasicValueEnum<'ctx>> {
        let function = self
            .module
            .get_function(callee)
            .ok_or_else(|| format!("unknown function: {}", callee))?;

        if function.count_params() as usize != args.len() {
            return Err(format!(
                "{} expects {} arguments, got {}",
                callee,
                function.count_params(),
                args.len()
            )
            .into());
        }

        let compiled_args: Result<Vec<_>> = args
            .iter()
            .map(|arg| self.compile_expr(arg).map(Into::into))
            .collect();
        let compiled_args = compiled_args?;

        let call_site = self
            .builder
            .build_call(function, &compiled_args, "calltmp")?;

        match call_site.try_as_basic_value() {
            ValueKind::Basic(basic_value) => Ok(basic_value),
            ValueKind::Instruction(_) => Err(format!("{} returns void", callee).into()),
        }
    }

    pub(crate) fn compile_expr(&mut self, expr: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            ExprAST::Number(n) => self.compile_number(*n),
            ExprAST::Variable(name) => self.compile_variable(name),
            ExprAST::Binary { op, lhs, rhs } => self.compile_binary(*op, lhs, rhs),
            ExprAST::Call { callee, args } => self.compile_call(callee, args),
        }
    }
}
