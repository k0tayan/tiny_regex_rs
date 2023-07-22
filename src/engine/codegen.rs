//! ASTからコード生成を行う
use super::{parser::AST, Instruction};
use crate::helper::safe_add;
use std::{
    error::Error,
    fmt::{self, Display},
};

/// コード生成エラーを表す型
#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
    FailPlus
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}

/// コード生成器
#[derive(Default, Debug)]
struct Generator {
    pc: usize,
    insts: Vec<Instruction>,
}

/// コード生成を行う関数
pub fn get_code(ast: &AST) -> Result<Vec<Instruction>, Box<CodeGenError>> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}

/// コード生成器のメソッド定義
impl Generator {
    /// コード生成を行う関数の入り口
    fn gen_code(&mut self, ast: &AST) -> Result<(), Box<CodeGenError>> {
        self.gen_expr(ast)?;
        self.inc_pc()?;
        self.insts.push(Instruction::Match);
        Ok(())
    }

    /// ASTをパターン分けしコード生成を行う関数
    fn gen_expr(&mut self, ast: &AST) -> Result<(), Box<CodeGenError>> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::Dot => self.gen_dot()?,
            AST::Or(e1, e2) => self.gen_or(e1, e2)?,
            AST::Plus(e) => self.gen_plus(e)?,
            AST::Star(e) => self.gen_star(e)?,
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,
        }

        Ok(())
    }

    /// char命令生成関数
    fn gen_char(&mut self, c: char) -> Result<(), Box<CodeGenError>> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    /// dot命令生成器。
    ///
    fn gen_dot(&mut self) -> Result<(), Box<CodeGenError>> {
        let inst = Instruction::Dot;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    /// OR演算子のコード生成器。
    ///
    /// 以下のようなコードを生成。
    ///
    /// ```text
    ///     split L1, L2
    /// L1: e1のコード
    ///     jmp L3
    /// L2: e2のコード
    /// L3:
    /// ```
    fn gen_or(&mut self, e1: &AST, e2: &AST) -> Result<(), Box<CodeGenError>> {
        // split L1, L2
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0); // self.pcがL1。L2を仮に0と設定
        self.insts.push(split);

        // L1: e1のコード
        self.gen_expr(e1)?;

        // jmp L3
        let jmp_addr = self.pc;
        self.insts.push(Instruction::Jump(0)); // L3を仮に0と設定

        // L2の値を設定
        self.inc_pc()?;
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailOr));
        }

        // L2: e2のコード
        self.gen_expr(e2)?;

        // L3の値を設定
        if let Some(Instruction::Jump(l3)) = self.insts.get_mut(jmp_addr) {
            *l3 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailOr));
        }

        Ok(())
    }

    /// ?限量子のコード生成器。
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    ///     split L1, L2
    /// L1: eのコード
    /// L2:
    /// ```
    fn gen_question(&mut self, e: &AST) -> Result<(), Box<CodeGenError>> {
        // TODO:
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0); // self.pcがL1。L2を仮に0と設定
        self.insts.push(split);

        // L1: eのコード
        self.gen_expr(e)?;

        // L2の値を設定
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailQuestion));
        }
        Ok(())
    }

    /// 以下のようなコードを生成
    ///
    /// ```text
    /// L1: eのコード
    ///     split L1, L2
    /// L2:
    /// ```
    fn gen_plus(&mut self, e: &AST) -> Result<(), Box<CodeGenError>> {
        // TODO:
        let l1_addr = self.pc;

        self.gen_expr(e)?;

        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(l1_addr, 0); // self.pcがL1。L2を仮に0と設定
        self.insts.push(split);

        // L2の値を設定
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailPlus));
        }
        Ok(())
    }

    /// *限量子のコード生成器。
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    /// L1: split L2, L3
    /// L2: eのコード
    ///     jump L1
    /// L3:
    /// ```
    fn gen_star(&mut self, e: &AST) -> Result<(), Box<CodeGenError>> {
        // TODO:
        let l1_addr = self.pc;

        // L1: split L2, L3
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0); // self.pcがL2。L3を仮に0と設定
        self.insts.push(split);

        // L2: eのコード
        self.gen_expr(e)?;

        // jump L1
        self.insts.push(Instruction::Jump(l1_addr));

        // L3の値を設定
        self.inc_pc()?;
        if let Some(Instruction::Split(_, l3)) = self.insts.get_mut(l1_addr) {
            *l3 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailStar));
        }
        Ok(())
    }

    /// 連続する正規表現のコード生成
    fn gen_seq(&mut self, exprs: &[AST]) -> Result<(), Box<CodeGenError>> {
        for e in exprs {
            self.gen_expr(e)?;
        }

        Ok(())
    }

    /// プログラムカウンタをインクリメント
    fn inc_pc(&mut self) -> Result<(), Box<CodeGenError>> {
        safe_add(&mut self.pc, &1, || Box::new(CodeGenError::PCOverFlow))
    }
}
