//! IL entity references.
//!
//! Instructions in Cretonne IL need to reference other entities in the function. This can be other
//! parts of the function like extended basic blocks or stack slots, or it can be external entities
//! that are declared in the function preamble in the text format.
//!
//! These entity references in instruction operands are not implemented as Rust references both
//! because Rust's ownership and mutability rules make it difficult, and because 64-bit pointers
//! take up a lot of space, and we want a compact in-memory representation. Instead, entity
//! references are structs wrapping a `u32` index into a table in the `Function` main data
//! structure. There is a separate index type for each entity type, so we don't lose type safety.
//!
//! The `entities` module defines public types for the entity references along with constants
//! representing an invalid reference. We prefer to use `Option<EntityRef>` whenever possible, but
//! unfortunately that type is twice as large as the 32-bit index type on its own. Thus, compact
//! data structures use the sentinen constant, while function arguments and return values prefer
//! the more Rust-like `Option<EntityRef>` variant.
//!
//! The entity references all implement the `Display` trait in a way that matches the textual IL
//! format.

use entity_map::EntityRef;
use std::default::Default;
use std::fmt::{self, Display, Formatter};
use std::u32;

/// An opaque reference to an extended basic block in a function.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Ebb(u32);

impl EntityRef for Ebb {
    fn new(index: usize) -> Self {
        assert!(index < (u32::MAX as usize));
        Ebb(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

impl Ebb {
    /// Create a new EBB reference from its number. This corresponds to the ebbNN representation.
    pub fn with_number(n: u32) -> Option<Ebb> {
        if n < u32::MAX { Some(Ebb(n)) } else { None }
    }
}

/// Display an `Ebb` reference as "ebb12".
impl Display for Ebb {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "ebb{}", self.0)
    }
}

/// A guaranteed invalid EBB reference.
pub const NO_EBB: Ebb = Ebb(u32::MAX);

impl Default for Ebb {
    fn default() -> Ebb {
        NO_EBB
    }
}

/// An opaque reference to an instruction in a function.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Inst(u32);

impl EntityRef for Inst {
    fn new(index: usize) -> Self {
        assert!(index < (u32::MAX as usize));
        Inst(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Display an `Inst` reference as "inst7".
impl Display for Inst {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "inst{}", self.0)
    }
}

/// A guaranteed invalid instruction reference.
pub const NO_INST: Inst = Inst(u32::MAX);

impl Default for Inst {
    fn default() -> Inst {
        NO_INST
    }
}


/// An opaque reference to an SSA value.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Value(u32);

impl EntityRef for Value {
    fn new(index: usize) -> Value {
        assert!(index < (u32::MAX as usize));
        Value(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Value references can either reference an instruction directly, or they can refer to the
/// extended value table.
pub enum ExpandedValue {
    /// This is the first value produced by the referenced instruction.
    Direct(Inst),

    /// This value is described in the extended value table.
    Table(usize),

    /// This is NO_VALUE.
    None,
}

impl Value {
    /// Create a `Direct` value from its number representation.
    /// This is the number in the vNN notation.
    pub fn direct_with_number(n: u32) -> Option<Value> {
        if n < u32::MAX / 2 {
            let encoding = n * 2;
            assert!(encoding < u32::MAX);
            Some(Value(encoding))
        } else {
            None
        }
    }

    /// Create a `Table` value from its number representation.
    /// This is the number in the vxNN notation.
    pub fn table_with_number(n: u32) -> Option<Value> {
        if n < u32::MAX / 2 {
            let encoding = n * 2 + 1;
            assert!(encoding < u32::MAX);
            Some(Value(encoding))
        } else {
            None
        }
    }
    /// Create a `Direct` value corresponding to the first value produced by `i`.
    pub fn new_direct(i: Inst) -> Value {
        let encoding = i.index() * 2;
        assert!(encoding < u32::MAX as usize);
        Value(encoding as u32)
    }

    /// Create a `Table` value referring to entry `i` in the `DataFlowGraph.extended_values` table.
    /// This constructor should not be used directly. Use the public `DataFlowGraph` methods to
    /// manipulate values.
    pub fn new_table(index: usize) -> Value {
        let encoding = index * 2 + 1;
        assert!(encoding < u32::MAX as usize);
        Value(encoding as u32)
    }

    /// Expand the internal representation into something useful.
    pub fn expand(&self) -> ExpandedValue {
        use self::ExpandedValue::*;
        if *self == NO_VALUE {
            return None;
        }
        let index = (self.0 / 2) as usize;
        if self.0 % 2 == 0 {
            Direct(Inst::new(index))
        } else {
            Table(index)
        }
    }

    /// Assuming that this is a direct value, get the referenced instruction.
    ///
    /// # Panics
    ///
    /// If this is not a value created with `new_direct()`.
    pub fn unwrap_direct(&self) -> Inst {
        if let ExpandedValue::Direct(inst) = self.expand() {
            inst
        } else {
            panic!("{} is not a direct value", self)
        }
    }
}

/// Display a `Value` reference as "v7" or "v2x".
impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        use self::ExpandedValue::*;
        match self.expand() {
            Direct(i) => write!(fmt, "v{}", i.0),
            Table(i) => write!(fmt, "vx{}", i),
            None => write!(fmt, "NO_VALUE"),
        }
    }
}

/// A guaranteed invalid value reference.
pub const NO_VALUE: Value = Value(u32::MAX);

impl Default for Value {
    fn default() -> Value {
        NO_VALUE
    }
}

/// An opaque reference to a stack slot.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StackSlot(u32);

impl EntityRef for StackSlot {
    fn new(index: usize) -> StackSlot {
        assert!(index < (u32::MAX as usize));
        StackSlot(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Display a `StackSlot` reference as "ss12".
impl Display for StackSlot {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "ss{}", self.0)
    }
}

/// A guaranteed invalid stack slot reference.
pub const NO_STACK_SLOT: StackSlot = StackSlot(u32::MAX);

impl Default for StackSlot {
    fn default() -> StackSlot {
        NO_STACK_SLOT
    }
}

/// An opaque reference to a jump table.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct JumpTable(u32);

impl EntityRef for JumpTable {
    fn new(index: usize) -> JumpTable {
        assert!(index < (u32::MAX as usize));
        JumpTable(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Display a `JumpTable` reference as "jt12".
impl Display for JumpTable {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "jt{}", self.0)
    }
}

/// A guaranteed invalid jump table reference.
pub const NO_JUMP_TABLE: JumpTable = JumpTable(u32::MAX);

impl Default for JumpTable {
    fn default() -> JumpTable {
        NO_JUMP_TABLE
    }
}

/// A reference to an external function.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FuncRef(u32);

impl EntityRef for FuncRef {
    fn new(index: usize) -> FuncRef {
        assert!(index < (u32::MAX as usize));
        FuncRef(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Display a `FuncRef` reference as "fn12".
impl Display for FuncRef {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "fn{}", self.0)
    }
}

/// A guaranteed invalid function reference.
pub const NO_FUNC_REF: FuncRef = FuncRef(u32::MAX);

impl Default for FuncRef {
    fn default() -> FuncRef {
        NO_FUNC_REF
    }
}

/// A reference to a function signature.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SigRef(u32);

impl EntityRef for SigRef {
    fn new(index: usize) -> SigRef {
        assert!(index < (u32::MAX as usize));
        SigRef(index as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Display a `SigRef` reference as "sig12".
impl Display for SigRef {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "sig{}", self.0)
    }
}

/// A guaranteed invalid function reference.
pub const NO_SIG_REF: SigRef = SigRef(u32::MAX);

impl Default for SigRef {
    fn default() -> SigRef {
        NO_SIG_REF
    }
}

/// A reference to any of the entities defined in this module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AnyEntity {
    /// The whole function.
    Function,
    /// An extended basic block.
    Ebb(Ebb),
    /// An instruction.
    Inst(Inst),
    /// An SSA value.
    Value(Value),
    /// A stack slot.
    StackSlot(StackSlot),
    /// A jump table.
    JumpTable(JumpTable),
    /// An external function.
    FuncRef(FuncRef),
    /// A function call signature.
    SigRef(SigRef),
}

impl Display for AnyEntity {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            AnyEntity::Function => write!(fmt, "function"),
            AnyEntity::Ebb(r) => r.fmt(fmt),
            AnyEntity::Inst(r) => r.fmt(fmt),
            AnyEntity::Value(r) => r.fmt(fmt),
            AnyEntity::StackSlot(r) => r.fmt(fmt),
            AnyEntity::JumpTable(r) => r.fmt(fmt),
            AnyEntity::FuncRef(r) => r.fmt(fmt),
            AnyEntity::SigRef(r) => r.fmt(fmt),
        }
    }
}

impl From<Ebb> for AnyEntity {
    fn from(r: Ebb) -> AnyEntity {
        AnyEntity::Ebb(r)
    }
}

impl From<Inst> for AnyEntity {
    fn from(r: Inst) -> AnyEntity {
        AnyEntity::Inst(r)
    }
}

impl From<Value> for AnyEntity {
    fn from(r: Value) -> AnyEntity {
        AnyEntity::Value(r)
    }
}

impl From<StackSlot> for AnyEntity {
    fn from(r: StackSlot) -> AnyEntity {
        AnyEntity::StackSlot(r)
    }
}

impl From<JumpTable> for AnyEntity {
    fn from(r: JumpTable) -> AnyEntity {
        AnyEntity::JumpTable(r)
    }
}

impl From<FuncRef> for AnyEntity {
    fn from(r: FuncRef) -> AnyEntity {
        AnyEntity::FuncRef(r)
    }
}

impl From<SigRef> for AnyEntity {
    fn from(r: SigRef) -> AnyEntity {
        AnyEntity::SigRef(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::u32;
    use entity_map::EntityRef;

    #[test]
    fn value_with_number() {
        assert_eq!(Value::direct_with_number(0).unwrap().to_string(), "v0");
        assert_eq!(Value::direct_with_number(1).unwrap().to_string(), "v1");
        assert_eq!(Value::table_with_number(0).unwrap().to_string(), "vx0");
        assert_eq!(Value::table_with_number(1).unwrap().to_string(), "vx1");

        assert_eq!(Value::direct_with_number(u32::MAX / 2), None);
        assert_eq!(match Value::direct_with_number(u32::MAX / 2 - 1).unwrap().expand() {
                       ExpandedValue::Direct(i) => i.index() as u32,
                       _ => u32::MAX,
                   },
                   u32::MAX / 2 - 1);

        assert_eq!(Value::table_with_number(u32::MAX / 2), None);
        assert_eq!(match Value::table_with_number(u32::MAX / 2 - 1).unwrap().expand() {
                       ExpandedValue::Table(i) => i as u32,
                       _ => u32::MAX,
                   },
                   u32::MAX / 2 - 1);
    }
}
