use cranelift_entity::{PrimaryMap, entity_impl};
use std::{borrow::Cow, num::NonZeroU32, ops::Index};

/// Bit-width of a value.
pub type Width = NonZeroU32;

/// Arena handle for a [`WidthExpr`].
#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct WidthExprId(u32);
entity_impl!(WidthExprId, "WidthExpr");

/// Arena handle for a [`BoolExpr`].
#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct BoolExprId(u32);
entity_impl!(BoolExprId, "BoolExpr");

/// Arena handle for a [`BitVecExpr`].
#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct BitVecExprId(u32);
entity_impl!(BitVecExprId, "BitVecExpr");

/// Expression computing a [`Width`].
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum WidthExpr {
    /// Symbolic width variable.
    Var(String),
    /// Concrete width.
    Const(Width),
    /// Sum of two widths.
    Add(WidthExprId, WidthExprId),
    /// Difference of two widths.
    Sub(WidthExprId, WidthExprId),
    /// Conditional width: `cond ? then : else`.
    Ite(BoolExprId, WidthExprId, WidthExprId),
}

/// Boolean width expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BoolExpr {
    /// Literal boolean.
    Const(bool),
    /// Logical negation.
    Not(BoolExprId),
    /// Logical OR.
    Or(BoolExprId, BoolExprId),
    /// Logical AND.
    And(BoolExprId, BoolExprId),
    /// Greater-than comparison of two widths.
    Gt(WidthExprId, WidthExprId),
    /// Equality comparison of two widths.
    Eq(WidthExprId, WidthExprId),
}

/// Binary arithmetic operators for [`BitVecExpr::BinArith`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BitVecBinArithOp {
    /// Bitwise AND.
    And,
    /// Addition.
    Add,
    /// Bitwise OR.
    Or,
}

/// Unary arithmetic operators for [`BitVecExpr::UnArith`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BitVecUnArithOp {
    /// Bitwise NOT.
    Not,
    /// Signed negation.
    Neg,
}

/// Comparison operators for [`BitVecExpr::Cmp`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BitVecCmpOp {
    /// Greater-than.
    Gt,
    /// Equality.
    Eq,
}

/// Expression computing a bit-vector value.
///
/// Most variants carry a [`WidthExprId`] giving the result width.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BitVecExpr {
    /// Symbolic bit-vector variable.
    Var(WidthExprId, String),
    /// Constant bit-vector.
    Const(WidthExprId, u64),
    /// Arithmetic op applied to one operand.
    UnArith(WidthExprId, BitVecUnArithOp, BitVecExprId),
    /// Arithmetic op applied to two operands.
    BinArith(WidthExprId, BitVecBinArithOp, BitVecExprId, BitVecExprId),
    /// Comparison of two operands.
    ///
    /// Outputs a 1-bit bit-vector: `#b1` is true, `#b0` is false.
    Cmp(BitVecCmpOp, BitVecExprId, BitVecExprId),
    /// Concatenation of two bit-vectors.
    Cat(WidthExprId, BitVecExprId, BitVecExprId),
    /// Conditional bit-vector: `cond ? then : else`.
    ///
    /// `cond` is expected to be a 1-bit bit-vector: `#b1` is true, `#b0` is false.
    Ite(WidthExprId, BitVecExprId, BitVecExprId, BitVecExprId),
    /// Sign extension.
    SExt(WidthExprId, BitVecExprId),
    /// Zero extension.
    ZExt(WidthExprId, BitVecExprId),
}

/// Expression arena.
///
/// TODO: hashcons?
pub struct Ctx {
    width_exprs: PrimaryMap<WidthExprId, WidthExpr>,
    bool_exprs: PrimaryMap<BoolExprId, BoolExpr>,
    bv_exprs: PrimaryMap<BitVecExprId, BitVecExpr>,
    /// Width constant `1`.
    width_one: WidthExprId,
    /// 1-bit constant `#b0`.
    bv_false: BitVecExprId,
    /// 1-bit constant `#b1`.
    bv_true: BitVecExprId,
}

impl Default for Ctx {
    /// Builds an empty `Ctx`, pre-populated with the common constants
    /// `width_one`, `bv_false`, `bv_true`.
    fn default() -> Self {
        let mut width_exprs: PrimaryMap<WidthExprId, WidthExpr> = Default::default();
        let width_one = width_exprs.push(WidthExpr::Const(NonZeroU32::new(1).unwrap()));
        let mut bv_exprs: PrimaryMap<BitVecExprId, BitVecExpr> = Default::default();
        let bv_false = bv_exprs.push(BitVecExpr::Const(width_one, 0));
        let bv_true = bv_exprs.push(BitVecExpr::Const(width_one, 1)); // Arbitrary choice
        Self {
            width_exprs,
            bool_exprs: Default::default(),
            bv_exprs,
            width_one,
            bv_false,
            bv_true,
        }
    }
}

impl Index<WidthExprId> for Ctx {
    type Output = WidthExpr;
    /// Looks up a [`WidthExpr`] by its [`WidthExprId`].
    fn index(&self, index: WidthExprId) -> &Self::Output {
        &self.width_exprs[index]
    }
}

impl Index<BoolExprId> for Ctx {
    type Output = BoolExpr;
    /// Looks up a [`BoolExpr`] by its [`BoolExprId`].
    fn index(&self, index: BoolExprId) -> &Self::Output {
        &self.bool_exprs[index]
    }
}

impl Index<BitVecExprId> for Ctx {
    type Output = BitVecExpr;
    /// Looks up a [`BitVecExpr`] by its [`BitVecExprId`].
    fn index(&self, index: BitVecExprId) -> &Self::Output {
        &self.bv_exprs[index]
    }
}

/// Concrete widths assigned to each width variable.
pub type WidthAssignment = std::collections::HashMap<String, u32>;

impl Ctx {
    /// Inserts a [`WidthExpr`], returning its handle.
    pub fn insert_width(&mut self, value: WidthExpr) -> WidthExprId {
        self.width_exprs.push(value)
    }

    /// Inserts a [`BoolExpr`], returning its handle.
    pub fn insert_bool(&mut self, value: BoolExpr) -> BoolExprId {
        self.bool_exprs.push(value)
    }

    /// Inserts a [`BitVecExpr`], returning its handle.
    pub fn insert_bv(&mut self, value: BitVecExpr) -> BitVecExprId {
        self.bv_exprs.push(value)
    }

    /// Returns the handle for the width constant `1`.
    pub fn width_one(&self) -> WidthExprId {
        self.width_one
    }

    /// Inserts a width variable with the given name, returning its handle.
    pub fn width_var(&mut self, name: Cow<String>) -> WidthExprId {
        self.width_exprs.push(WidthExpr::Var(name.into_owned()))
    }

    /// Returns the handle for the 1-bit constant `#b0`.
    pub fn bv_false(&self) -> BitVecExprId {
        self.bv_false
    }

    /// Returns the handle for the 1-bit constant `#b1`.
    pub fn bv_true(&self) -> BitVecExprId {
        self.bv_true
    }

    /// Returns the width of a [`BitVecExpr`].
    ///
    /// Most variants carry their width explicitly; [`BitVecExpr::Cmp`] is
    /// defined to produce a 1-bit result (see [`BitVecExpr::Ite`]), so its
    /// width is always [`Ctx::width_one`].
    pub fn width(&self, expr: BitVecExprId) -> WidthExprId {
        match &self[expr] {
            // extract the width parameter:
            BitVecExpr::Var(w, _)
            | BitVecExpr::Const(w, _)
            | BitVecExpr::BinArith(w, _, _, _)
            | BitVecExpr::UnArith(w, _, _)
            | BitVecExpr::Cat(w, _, _)
            | BitVecExpr::Ite(w, _, _, _)
            | BitVecExpr::SExt(w, _)
            | BitVecExpr::ZExt(w, _) => *w,
            BitVecExpr::Cmp(_, _, _) => self.width_one,
        }
    }

    /// Evaulates the concrete width of a [`WidthExpr`] given variable assignments.
    pub fn eval_width(&self, expr: WidthExprId, assgn: &WidthAssignment) -> u32 {
        match &self[expr] {
            WidthExpr::Var(s) => assgn[s],
            WidthExpr::Const(w) => (*w).into(),
            WidthExpr::Add(we1, we2) => self.eval_width(*we1, assgn) + self.eval_width(*we2, assgn),
            WidthExpr::Sub(we1, we2) => self.eval_width(*we1, assgn) - self.eval_width(*we2, assgn),
            WidthExpr::Ite(be, we1, we2) => {
                if self.eval_bool(*be, assgn) {
                    self.eval_width(*we1, assgn)
                } else {
                    self.eval_width(*we2, assgn)
                }
            }
        }
    }

    /// Evaluates a boolean width expression [`BoolExpr`] given variable assignments.
    pub fn eval_bool(&self, expr: BoolExprId, assgn: &WidthAssignment) -> bool {
        match &self[expr] {
            BoolExpr::Const(b) => *b,
            BoolExpr::Not(be) => !self.eval_bool(*be, assgn),
            BoolExpr::Or(be1, be2) => self.eval_bool(*be1, assgn) || self.eval_bool(*be2, assgn),
            BoolExpr::And(be1, be2) => self.eval_bool(*be1, assgn) && self.eval_bool(*be2, assgn),
            BoolExpr::Gt(be1, be2) => self.eval_width(*be1, assgn) > self.eval_width(*be2, assgn),
            BoolExpr::Eq(be1, be2) => self.eval_width(*be1, assgn) == self.eval_width(*be2, assgn),
        }
    }
}

/// A rewrite rule: `lhs -> rhs`, subject to width constraints.
///
#[derive(Debug, Default)]
pub struct RwRule {
    /// Rule name.
    pub name: String,
    /// List of all parametric width variables that appear on the left hand side.
    pub width_vars: Vec<String>,
    /// Left-hand side (pattern to match).
    pub lhs: BitVecExprId,
    /// Right-hand side (replacement).
    pub rhs: BitVecExprId,
}

pub type RwProg = Vec<RwRule>;
