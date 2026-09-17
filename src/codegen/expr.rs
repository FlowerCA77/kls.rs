use crate::Result;
use crate::codegen::Codegen;
use crate::frontend::ast::ExprAST;

use inkwell::{
    FloatPredicate,
    values::{BasicValue, BasicValueEnum, FastMathFlags, FloatValue, ValueKind},
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

    fn compile_unary(&mut self, op: &str, operand: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        let v = self.compile_expr(operand)?.into_float_value();

        let value = match op {
            "-" => {
                let zero = self.context.f64_type().const_float(0.0);
                self.builder.build_float_sub(zero, v, "negtmp")?
            }
            "+" => v,
            _ => return Err(format!("undefined unary operator: `{}`", op).into()),
        };
        Ok(value.into())
    }

    fn compile_binary(
        &mut self,
        op: &str,
        lhs: &ExprAST,
        rhs: &ExprAST,
    ) -> Result<BasicValueEnum<'ctx>> {
        let x: FloatValue = self.compile_expr(lhs)?.into_float_value();
        let y: FloatValue = self.compile_expr(rhs)?.into_float_value();

        let value: FloatValue = match op {
            "+" => self.builder.build_float_add(x, y, "addtmp")?.into(),
            "-" => self.builder.build_float_sub(x, y, "subtmp")?.into(),
            "*" => self.builder.build_float_mul(x, y, "multmp")?.into(),
            "/" => self.builder.build_float_div(x, y, "divtmp")?.into(),
            "<" => {
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
            "<=" => {
                let cmp = self
                    .builder
                    .build_float_compare(FloatPredicate::ULE, x, y, "cmptmp")?;
                let bool_as_float = self.builder.build_unsigned_int_to_float(
                    cmp,
                    self.context.f64_type(),
                    "booltmp",
                )?;
                bool_as_float.into()
            }
            ">" => {
                let cmp = self
                    .builder
                    .build_float_compare(FloatPredicate::UGT, x, y, "cmptmp")?;
                let bool_as_float = self.builder.build_unsigned_int_to_float(
                    cmp,
                    self.context.f64_type(),
                    "booltmp",
                )?;
                bool_as_float.into()
            }
            ">=" => {
                let cmp = self
                    .builder
                    .build_float_compare(FloatPredicate::UGE, x, y, "cmptmp")?;
                let bool_as_float = self.builder.build_unsigned_int_to_float(
                    cmp,
                    self.context.f64_type(),
                    "booltmp",
                )?;
                bool_as_float.into()
            }
            _ => return Err(format!("undefined binary operator: `{}`", op).into()),
        };

        if self.options.fast_math {
            if let Some(inst) = value.as_instruction_value() {
                inst.set_fast_math_flags(FastMathFlags::all())?;
            }
        }

        Ok(BasicValueEnum::FloatValue(value))
    }

    fn compile_call(&mut self, callee: &str, args: &[ExprAST]) -> Result<BasicValueEnum<'ctx>> {
        let function = if let Some(f) = self.module.get_function(callee) {
            f
        } else if let Some(proto) = self.function_protos.get(callee).cloned() {
            self.compile_prototype(&proto)?
        } else {
            return Err(format!("unknown function: {}", callee).into());
        };

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

    fn compile_if(
        &mut self,
        cond: &ExprAST,
        e_true: &ExprAST,
        e_false: &ExprAST,
    ) -> Result<BasicValueEnum<'ctx>> {
        let cond_val = self.compile_expr(cond)?.into_float_value();
        let zero = self.context.f64_type().const_float(0.0);
        let cmp =
            self.builder
                .build_float_compare(FloatPredicate::ONE, cond_val, zero, "ifcond")?;

        let function = self
            .builder
            .get_insert_block()
            .ok_or("cannot insert the if-entry basic block")?
            .get_parent()
            .ok_or("cannot get the parent of if-entry basic block")?;

        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let ifcont_bb = self.context.append_basic_block(function, "ifcont");

        self.builder
            .build_conditional_branch(cmp, then_bb, else_bb)?;

        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expr(e_true)?.into_float_value();
        self.builder.build_unconditional_branch(ifcont_bb)?;
        let then_bb = self
            .builder
            .get_insert_block()
            .ok_or("cannot insert basic block of then-branch")?;

        self.builder.position_at_end(else_bb);
        let else_val = self.compile_expr(e_false)?.into_float_value();
        self.builder.build_unconditional_branch(ifcont_bb)?;
        let else_bb = self
            .builder
            .get_insert_block()
            .ok_or("cannot insert basic block of else-branch")?;

        self.builder.position_at_end(ifcont_bb);
        let phi = self.builder.build_phi(self.context.f64_type(), "iftmp")?;
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

        Ok(phi.as_basic_value())
    }

    fn compile_for(
        &mut self,
        var: &str,
        e_init: &ExprAST,
        e_cond: &ExprAST,
        e_step: &ExprAST,
        e_body: &ExprAST,
    ) -> Result<BasicValueEnum<'ctx>> {
        let init_val = self.compile_expr(e_init)?.into_float_value();

        let function = self
            .builder
            .get_insert_block()
            .ok_or("cannot insert the for-entry basic block")?
            .get_parent()
            .ok_or("cannot get the parent of for-entry basic block")?;
        let preheader_bb = self
            .builder
            .get_insert_block()
            .ok_or("cannot insert the preheader basic block")?;

        let loop_bb = self.context.append_basic_block(function, "loop");
        self.builder.build_unconditional_branch(loop_bb)?;

        self.builder.position_at_end(loop_bb);
        let variable = self.builder.build_phi(self.context.f64_type(), var)?;
        variable.add_incoming(&[(&init_val, preheader_bb)]);

        let old_binding = self
            .named_values
            .insert(var.to_string(), variable.as_basic_value());

        let cond_val = self.compile_expr(e_cond)?.into_float_value();
        let zero = self.context.f64_type().const_float(0.0);
        let cmp =
            self.builder
                .build_float_compare(FloatPredicate::ONE, cond_val, zero, "loopcond")?;

        let body_bb = self.context.append_basic_block(function, "loopbody");
        let after_bb = self.context.append_basic_block(function, "afterloop");
        self.builder
            .build_conditional_branch(cmp, body_bb, after_bb)?;

        self.builder.position_at_end(body_bb);
        self.compile_expr(e_body)?;

        let step_val = self.compile_expr(e_step)?.into_float_value();

        let body_end_bb = self
            .builder
            .get_insert_block()
            .ok_or("cannot insert basic block after for-loop body")?;
        self.builder.build_unconditional_branch(loop_bb)?;
        variable.add_incoming(&[(&step_val, body_end_bb)]);

        self.builder.position_at_end(after_bb);
        match old_binding {
            Some(old) => self.named_values.insert(var.to_string(), old),
            None => self.named_values.remove(var),
        };

        Ok(self.context.f64_type().const_float(0.0).into())
    }

    pub(crate) fn compile_expr(&mut self, expr: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            ExprAST::Number(n) => self.compile_number(*n),
            ExprAST::Variable(name) => self.compile_variable(name),
            ExprAST::Unary { op, operand } => self.compile_unary(op.as_str(), operand),
            ExprAST::Binary { op, lhs, rhs } => self.compile_binary(op.as_str(), lhs, rhs),
            ExprAST::Call { callee, args } => self.compile_call(callee, args),
            ExprAST::If {
                cond,
                e_true,
                e_false,
            } => self.compile_if(cond, e_true, e_false),
            ExprAST::For {
                var,
                e_init,
                e_cond,
                e_step,
                e_body,
            } => self.compile_for(var, e_init, e_cond, e_step, e_body),
        }
    }
}
