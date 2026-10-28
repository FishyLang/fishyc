//! Verified physical lifetime IR and its deterministic, non-optimizing lowering.
//! No source legality, lifetime inference or ARC optimization belongs here.
use crate::ir::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadMode {
    Copy,
    Move,
    Observe,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    ImmutableBacking,
    Evaluation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropRecipe {
    Trivial,
    Managed,
    /// A compiler result can carry either a moved logical value or a bounded
    /// evaluation lease. Both discharge one verified edge; only the former is
    /// a semantic value destruction. No user destructor is introduced here.
    Temporary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Local,
    Temporary,
    Construction,
}

#[derive(Debug, Clone)]
pub enum Operation {
    ReadValue {
        dest: VReg,
        source: VReg,
        ty: IrType,
        mode: ReadMode,
    },
    BorrowValue {
        dest: VReg,
        source: VReg,
        ty: IrType,
        mutable: bool,
    },
    /// Registration is declarative; the alloca/store order remains explicit.
    OwnerSlot {
        slot: VReg,
        ty: IrType,
        kind: SlotKind,
    },
    ViewSlot {
        slot: VReg,
        ty: IrType,
    },
    /// A cached value is handed back from its registered slot to a consumer.
    TransferSlot {
        slot: VReg,
        value: VReg,
        ty: IrType,
    },
    DropSlot {
        slot: VReg,
        ty: IrType,
        guarded: bool,
    },
    /// Ends a logical value/temporary; managed representation selects ARC glue.
    DropValue {
        value: VReg,
        recipe: DropRecipe,
    },
    /// Explicit immutable backing/evaluation edge, never a source Copy.
    AcquireSharedEdge {
        value: VReg,
        kind: EdgeKind,
    },
    OwnedCast {
        dest: VReg,
        value: VReg,
        source_ty: IrType,
        target_ty: IrType,
    },
    SwapValue {
        dest: VReg,
        slot: VReg,
        value: VReg,
        ty: IrType,
    },
    StoreOwned {
        slot: VReg,
        value: VReg,
        ty: IrType,
    },
    Construction {
        slot: VReg,
        fields: Option<Vec<usize>>,
    },
    InitializeField {
        owner: VReg,
        index: Option<usize>,
        slot: VReg,
        value: VReg,
        ty: IrType,
    },
    Resource {
        resource: String,
        owner: VReg,
        slot: VReg,
        ty: IrType,
        action: ResourceAction,
    },
}
#[derive(Debug, Clone)]
pub enum ResourceAction {
    Initialize(VReg),
    Take(VReg),
    Borrow(VReg),
    Drop(VReg),
}

pub fn managed(ty: &IrType) -> bool {
    match ty {
        IrType::Array(..) | IrType::FatPtr | IrType::Closure(..) => true,
        IrType::Ptr(t) => {
            matches!(t.as_ref(),IrType::Struct(n,_) if !n.starts_with("__fishy_range_"))
        }
        // Bare structs are layout descriptors/inline values, not header handles.
        IrType::Struct(..) => false,
        _ => false,
    }
}
impl Operation {
    pub fn destination(&self) -> Option<VReg> {
        match self {
            Self::ReadValue { dest, .. }
            | Self::BorrowValue { dest, .. }
            | Self::SwapValue { dest, .. }
            | Self::OwnedCast { dest, .. } => Some(*dest),
            Self::Resource {
                action:
                    ResourceAction::Take(r) | ResourceAction::Borrow(r) | ResourceAction::Drop(r),
                ..
            } => Some(*r),
            _ => None,
        }
    }
    pub fn operands(&self) -> Vec<VReg> {
        match self {
            Self::ReadValue { source, .. } | Self::BorrowValue { source, .. } => vec![*source],
            Self::OwnerSlot { slot, .. }
            | Self::ViewSlot { slot, .. }
            | Self::DropSlot { slot, .. }
            | Self::Construction { slot, .. } => vec![*slot],
            Self::TransferSlot { slot, value, .. }
            | Self::SwapValue { slot, value, .. }
            | Self::StoreOwned { slot, value, .. } => vec![*slot, *value],
            Self::InitializeField {
                owner, slot, value, ..
            } => vec![*owner, *slot, *value],
            Self::OwnedCast { value, .. } => vec![*value],
            Self::DropValue { value, .. } | Self::AcquireSharedEdge { value, .. } => vec![*value],
            Self::Resource {
                owner,
                slot,
                action,
                ..
            } => {
                let mut v = vec![*owner, *slot];
                if let ResourceAction::Initialize(value) = action {
                    v.push(*value);
                }
                v
            }
        }
    }
}

pub fn is_explicit(module: &ModuleIr) -> bool {
    module.ownership_phase == OwnershipPhase::Explicit
}

fn all_instructions(f: &FunctionIr) -> Vec<&Instruction> {
    f.blocks.iter().flat_map(|b| &b.instructions).collect()
}
fn construction_fields(module: &ModuleIr, all: &[&Instruction], root: VReg) -> Option<Vec<usize>> {
    for i in all {
        match i {
            Instruction::AllocStruct { dest, class_name } if *dest == root => {
                return Some(
                    module.structs[class_name]
                        .iter()
                        .enumerate()
                        .filter_map(|(i, t)| managed(t).then_some(i))
                        .collect(),
                )
            }
            Instruction::AllocArray { dest, size, ty } if *dest == root => {
                if !managed(ty) {
                    return Some(vec![]);
                }
                return all
                    .iter()
                    .find_map(|i| match i {
                        Instruction::ConstInt { dest, value } if dest == size => {
                            usize::try_from(*value).ok()
                        }
                        _ => None,
                    })
                    .map(|n| (0..n).collect());
            }
            _ => {}
        }
    }
    None
}

/// Materialize physical memory-edge contracts from registered cleanup cells and
/// concrete aggregate layouts. Ordinary Store remains for scalar/reference data
/// and non-owning default-argument views. No AST or ARC instructions are inspected.
pub fn formalize_storage(module: &mut ModuleIr) {
    let recipes = module.clone();
    for f in &mut module.functions {
        let constants = f
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter_map(|i| {
                if let Instruction::ConstInt { dest, value } = i {
                    Some((*dest, *value))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        let stored = f
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter_map(|i| match i {
                Instruction::Store { ptr, value, .. } => Some((*ptr, *value)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let mut construction = HashMap::new();
        let mut shells = HashMap::new();
        for i in f.blocks.iter().flat_map(|b| &b.instructions) {
            if let Instruction::Ownership(Operation::OwnerSlot {
                slot,
                kind: SlotKind::Construction,
                ..
            }) = i
            {
                let root = stored[slot];
                shells.insert(root, *slot);
                construction.insert(
                    *slot,
                    construction_fields(&recipes, &all_instructions(f), root),
                );
            }
        }
        let mut field_addresses = HashMap::new();
        for i in f.blocks.iter().flat_map(|b| &b.instructions) {
            if let Instruction::GetElementPtr {
                dest,
                base_ptr,
                indices,
                ..
            } = i
            {
                if let Some(owner) = shells.get(base_ptr) {
                    let index = indices
                        .last()
                        .and_then(|r| constants.get(r))
                        .and_then(|i| usize::try_from(*i).ok());
                    field_addresses.insert(*dest, (*owner, index));
                }
            }
        }
        let views = f
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter_map(|i| match i {
                Instruction::Ownership(Operation::ViewSlot { slot, .. }) => Some(*slot),
                _ => None,
            })
            .collect::<HashSet<_>>();
        for b in &mut f.blocks {
            let mut output = vec![];
            for i in std::mem::take(&mut b.instructions) {
                match i {
                    Instruction::Store { ty, ptr, value }
                        if managed(&ty) && !views.contains(&ptr) =>
                    {
                        if let Some((owner, index)) = field_addresses.get(&ptr) {
                            output.push(Instruction::Ownership(Operation::InitializeField {
                                owner: *owner,
                                index: *index,
                                slot: ptr,
                                value,
                                ty,
                            }));
                        } else {
                            output.push(Instruction::Ownership(Operation::StoreOwned {
                                slot: ptr,
                                value,
                                ty,
                            }));
                        }
                    }
                    Instruction::Ownership(Operation::OwnerSlot {
                        slot,
                        ty,
                        kind: SlotKind::Construction,
                    }) => {
                        output.push(Instruction::Ownership(Operation::OwnerSlot {
                            slot,
                            ty,
                            kind: SlotKind::Construction,
                        }));
                        output.push(Instruction::Ownership(Operation::Construction {
                            slot,
                            fields: construction[&slot].clone(),
                        }));
                    }
                    other => output.push(other),
                }
            }
            b.instructions = output;
        }
    }
}

/// Reject lowering bugs before ownership operations are erased. The structural
/// verifier independently checks the expanded SSA/types/CFG.
pub fn verify(module: &ModuleIr) -> Result<(), String> {
    for f in &module.functions {
        if f.blocks.is_empty() {
            continue;
        }
        let all = all_instructions(f);
        if all
            .iter()
            .any(|i| matches!(i, Instruction::Retain { .. } | Instruction::Release { .. }))
        {
            return Err(format!(
                "{}: ARC instruction before physical lifetime lowering",
                f.name
            ));
        }
        let allocations = all
            .iter()
            .filter_map(|i| match i {
                Instruction::Alloca { dest, ty, .. } => Some((*dest, ty)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        for i in &all {
            if let Instruction::Ownership(
                Operation::OwnerSlot { slot, ty, .. } | Operation::ViewSlot { slot, ty },
            ) = i
            {
                if allocations
                    .get(slot)
                    .copied()
                    .or_else(|| module.globals.get(slot))
                    != Some(ty)
                {
                    return Err(format!(
                        "{}: cleanup/view slot has no matching storage",
                        f.name
                    ));
                }
            }
        }
        verify_values(module, f).map_err(|e| format!("{e}\n{f}"))?;
        let fail = |s: String| format!("{}: physical ownership: {s}", f.name);
        let mut slots = HashMap::new();
        let mut sources = HashMap::new();
        let mut zeros = HashSet::new();
        let mut borrows = HashSet::new();
        for i in f.blocks.iter().flat_map(|b| &b.instructions) {
            match i {
                Instruction::Ownership(Operation::OwnerSlot { slot, ty, kind }) => {
                    if !managed(ty) {
                        return Err(fail(
                            "non-owning representation registered for managed cleanup".into(),
                        ));
                    }
                    if slots.insert(*slot, (ty.clone(), *kind)).is_some() {
                        return Err(fail(format!("duplicate owner slot {slot}")));
                    }
                }
                Instruction::Ownership(Operation::ReadValue {
                    dest,
                    source,
                    ty,
                    mode,
                }) => {
                    sources.insert(*dest, *source);
                    if *mode == ReadMode::Copy && managed(ty) {
                        return Err(fail("managed handle cannot implement CopyValue".into()));
                    }
                }
                Instruction::Ownership(Operation::BorrowValue { dest, .. }) => {
                    borrows.insert(*dest);
                }
                Instruction::Load { dest, src_ptr, .. } => {
                    sources.insert(*dest, *src_ptr);
                }
                Instruction::ConstInt { dest, value: 0 } => {
                    zeros.insert(*dest);
                }
                _ => {}
            }
        }
        loop {
            let old = zeros.len();
            for i in f.blocks.iter().flat_map(|b| &b.instructions) {
                if let Instruction::Cast { dest, value, .. } = i {
                    if zeros.contains(value) {
                        zeros.insert(*dest);
                    }
                }
            }
            if old == zeros.len() {
                break;
            }
        }
        for i in f.blocks.iter().flat_map(|b| &b.instructions) {
            if let Instruction::Ownership(op) = i {
                match op {
                    Operation::DropValue {
                        value,
                        recipe: DropRecipe::Managed,
                    }
                    | Operation::AcquireSharedEdge { value, .. }
                        if borrows.contains(value) =>
                    {
                        return Err(fail("safe borrow used as an owning strong edge".into()))
                    }
                    Operation::Resource { resource, ty, .. }
                        if module.resources.get(resource) != Some(ty) =>
                    {
                        return Err(fail(format!(
                            "unknown or mismatched initialized resource {resource}"
                        )))
                    }
                    Operation::TransferSlot { slot, .. }
                        if !slots.contains_key(slot) && !module.globals.contains_key(slot) =>
                    {
                        return Err(fail(format!(
                            "transfer of unregistered cleanup slot {slot}"
                        )))
                    }
                    _ => {}
                }
            }
        }
        // Empty/Live alternatives. Cleanup slots are conditional by design;
        // a take, unlike a guarded drop, requires a definitely live obligation.
        let by_id = f
            .blocks
            .iter()
            .map(|b| (b.id, b))
            .collect::<HashMap<_, _>>();
        let mut incoming = HashMap::<BlockId, HashMap<VReg, u8>>::new();
        incoming.insert(f.blocks[0].id, slots.keys().map(|s| (*s, 1)).collect());
        let mut work = VecDeque::from([f.blocks[0].id]);
        while let Some(id) = work.pop_front() {
            let mut state = incoming[&id].clone();
            for i in &by_id[&id].instructions {
                match i {
                    Instruction::Alloca { dest, .. } if slots.contains_key(dest) => {
                        state.insert(*dest, 1);
                    }
                    Instruction::Ownership(Operation::StoreOwned { slot, value, .. })
                        if slots.contains_key(slot) =>
                    {
                        state.insert(*slot, if zeros.contains(value) { 1 } else { 2 });
                    }
                    Instruction::Store { ptr, value, .. } if slots.contains_key(ptr) => {
                        state.insert(*ptr, if zeros.contains(value) { 1 } else { 2 });
                    }
                    Instruction::Ownership(Operation::ReadValue {
                        source,
                        mode: ReadMode::Move,
                        ..
                    }) if slots.contains_key(source) => {
                        state.insert(*source, 1);
                    }
                    Instruction::Ownership(Operation::TransferSlot { slot, .. }) => {
                        state.insert(*slot, 1);
                    }
                    Instruction::Ownership(Operation::DropSlot { slot, .. }) => {
                        state.insert(*slot, 1);
                    }
                    Instruction::Ownership(Operation::DropValue { value, .. }) => {
                        if let Some(slot) = sources.get(value).filter(|s| slots.contains_key(s)) {
                            state.insert(*slot, 1);
                        }
                    }
                    Instruction::Ret { .. } => {
                        for (slot, live) in &state {
                            if *live & 2 != 0 && !module.globals.contains_key(slot) {
                                return Err(fail(format!(
                                    "unresolved cleanup obligation for {slot}"
                                )));
                            }
                        }
                    }
                    _ => {}
                }
            }
            let successors = match by_id[&id].instructions.last() {
                Some(Instruction::Br { target }) => vec![*target],
                Some(Instruction::CondBr {
                    if_true, if_false, ..
                }) => vec![*if_true, *if_false],
                _ => vec![],
            };
            for to in successors {
                if !by_id.contains_key(&to) {
                    return Err(fail("missing ownership CFG successor".into()));
                }
                let changed = if let Some(old) = incoming.get_mut(&to) {
                    let before = old.clone();
                    for (s, v) in &state {
                        *old.entry(*s).or_insert(1) |= *v;
                    }
                    *old != before
                } else {
                    incoming.insert(to, state.clone());
                    true
                };
                if changed {
                    work.push_back(to);
                }
            }
        }
    }
    Ok(())
}

/// Sole builder-IR -> physical ARC/drop boundary. No cancellation, sinking,
/// hoisting or lifetime inference. The allocator/header/destructor ABI is reused.
pub fn lower(module: &mut ModuleIr) -> Result<(), String> {
    if !is_explicit(module) {
        return Ok(());
    }
    crate::ir_verify::verify(module)?;
    lower_unchecked(module);
    Ok(())
}
pub(crate) fn lower_unchecked(module: &mut ModuleIr) {
    let mut next_reg = module.globals.keys().map(|r| r.0 + 1).max().unwrap_or(0);
    let mut next_block = 0;
    for f in &module.functions {
        for (r, _) in &f.args {
            next_reg = next_reg.max(r.0 + 1);
        }
        for b in &f.blocks {
            next_block = next_block.max(b.id.0 + 1);
            for i in &b.instructions {
                for r in i.operands().into_iter().chain(i.destination()) {
                    next_reg = next_reg.max(r.0 + 1);
                }
            }
        }
    }
    let mut fresh = || {
        let r = VReg(next_reg);
        next_reg += 1;
        r
    };
    for f in &mut module.functions {
        let old = std::mem::take(&mut f.blocks);
        for b in old {
            let mut current = BasicBlock {
                id: b.id,
                name: b.name,
                instructions: vec![],
            };
            for i in b.instructions {
                let Instruction::Ownership(op) = i else {
                    current.instructions.push(i);
                    continue;
                };
                let output = &mut current.instructions;
                match op {
                    Operation::OwnerSlot { .. }
                    | Operation::ViewSlot { .. }
                    | Operation::Construction { .. } => {}
                    Operation::ReadValue {
                        dest,
                        source,
                        ty,
                        mode,
                    } => {
                        output.push(Instruction::Load {
                            dest,
                            ty: ty.clone(),
                            src_ptr: source,
                        });
                        if mode == ReadMode::Move {
                            let zero = fresh();
                            output.push(Instruction::ConstInt {
                                dest: zero,
                                value: 0,
                            });
                            output.push(Instruction::Store {
                                ty,
                                value: zero,
                                ptr: source,
                            });
                        }
                    }
                    Operation::BorrowValue {
                        dest, source, ty, ..
                    } => {
                        output.push(Instruction::Cast {
                            dest,
                            value: source,
                            source_ty: ty.clone(),
                            target_ty: ty,
                        });
                    }
                    Operation::TransferSlot { slot, ty, .. } => {
                        let zero = fresh();
                        output.push(Instruction::ConstInt {
                            dest: zero,
                            value: 0,
                        });
                        output.push(Instruction::Store {
                            ty,
                            value: zero,
                            ptr: slot,
                        });
                    }
                    Operation::DropValue {
                        value,
                        recipe: DropRecipe::Managed | DropRecipe::Temporary,
                    } => output.push(Instruction::Release { ptr: value }),
                    Operation::DropValue {
                        recipe: DropRecipe::Trivial,
                        ..
                    } => {}
                    Operation::AcquireSharedEdge { value, .. } => {
                        output.push(Instruction::Retain { ptr: value })
                    }
                    Operation::OwnedCast {
                        dest,
                        value,
                        source_ty,
                        target_ty,
                    } => output.push(Instruction::Cast {
                        dest,
                        value,
                        source_ty,
                        target_ty,
                    }),
                    Operation::StoreOwned { slot, value, ty }
                    | Operation::InitializeField {
                        slot, value, ty, ..
                    } => output.push(Instruction::Store {
                        ty,
                        value,
                        ptr: slot,
                    }),
                    Operation::SwapValue {
                        dest,
                        slot,
                        value,
                        ty,
                    } => {
                        output.push(Instruction::Load {
                            dest,
                            ty: ty.clone(),
                            src_ptr: slot,
                        });
                        output.push(Instruction::Store {
                            ty,
                            value,
                            ptr: slot,
                        });
                    }
                    Operation::DropSlot { slot, ty, guarded } => {
                        let value = fresh();
                        output.push(Instruction::Load {
                            dest: value,
                            ty: ty.clone(),
                            src_ptr: slot,
                        });
                        let zero = fresh();
                        output.push(Instruction::ConstInt {
                            dest: zero,
                            value: 0,
                        });
                        let null = fresh();
                        output.push(Instruction::Cast {
                            dest: null,
                            value: zero,
                            source_ty: IrType::I64,
                            target_ty: ty.clone(),
                        });
                        output.push(Instruction::Store {
                            ty,
                            value: null,
                            ptr: slot,
                        });
                        if guarded {
                            let cond = fresh();
                            output.push(Instruction::CmpNeq {
                                dest: cond,
                                left: value,
                                right: zero,
                            });
                            let release = BlockId(next_block);
                            let skip = BlockId(next_block + 1);
                            next_block += 2;
                            output.push(Instruction::CondBr {
                                cond,
                                if_true: release,
                                if_false: skip,
                            });
                            f.blocks.push(current);
                            f.blocks.push(BasicBlock {
                                id: release,
                                name: "drop.value.release".into(),
                                instructions: vec![
                                    Instruction::Release { ptr: value },
                                    Instruction::Br { target: skip },
                                ],
                            });
                            current = BasicBlock {
                                id: skip,
                                name: "drop.value.continue".into(),
                                instructions: vec![],
                            };
                        } else {
                            output.push(Instruction::Release { ptr: value });
                        }
                    }
                    Operation::Resource {
                        slot, ty, action, ..
                    } => match action {
                        ResourceAction::Borrow(dest) => output.push(Instruction::Cast {
                            dest,
                            value: slot,
                            source_ty: IrType::Ptr(Box::new(ty.clone())),
                            target_ty: IrType::Ptr(Box::new(ty)),
                        }),
                        ResourceAction::Initialize(value) => output.push(Instruction::Store {
                            ty,
                            value,
                            ptr: slot,
                        }),
                        ResourceAction::Take(dest) | ResourceAction::Drop(dest) => {
                            let drop = matches!(action, ResourceAction::Drop(_));
                            output.push(Instruction::Load {
                                dest,
                                ty: ty.clone(),
                                src_ptr: slot,
                            });
                            if managed(&ty) {
                                let zero = fresh();
                                output.push(Instruction::ConstInt {
                                    dest: zero,
                                    value: 0,
                                });
                                output.push(Instruction::Store {
                                    ty,
                                    value: zero,
                                    ptr: slot,
                                });
                                if drop {
                                    output.push(Instruction::Release { ptr: dest });
                                }
                            }
                        }
                    },
                }
            }
            f.blocks.push(current);
        }
    }
    module.ownership_phase = OwnershipPhase::Lowered;
}

/// A linear register obligation is either absent or an independently acquired
/// strong edge. Counts are bounded alternatives (0..3); joins union alternatives.
/// This checks lowering, not source loans/provenance. Payload/resource cell
/// initialization is governed by its explicit typed operation/recipe contract.
#[derive(Clone, Default, PartialEq, Eq)]
struct Flow {
    values: HashMap<VReg, u8>,
    cells: HashMap<VReg, u8>,
    leases: HashMap<VReg, u8>,
    cell_leases: HashMap<VReg, u8>,
    progress: HashMap<(VReg, usize), u8>,
}
impl Flow {
    fn join(&mut self, other: &Self) -> bool {
        let before = self.clone();
        for (a, b) in [
            (&mut self.values, &other.values),
            (&mut self.cells, &other.cells),
            (&mut self.leases, &other.leases),
            (&mut self.cell_leases, &other.cell_leases),
        ] {
            let keys = a.keys().chain(b.keys()).copied().collect::<HashSet<_>>();
            for k in keys {
                let x = a.get(&k).copied().unwrap_or(1) | b.get(&k).copied().unwrap_or(1);
                a.insert(k, x);
            }
        }
        let keys = self
            .progress
            .keys()
            .chain(other.progress.keys())
            .copied()
            .collect::<HashSet<_>>();
        for k in keys {
            let v = self.progress.get(&k).copied().unwrap_or(1)
                | other.progress.get(&k).copied().unwrap_or(1);
            self.progress.insert(k, v);
        }
        *self != before
    }
}
fn verify_values(module: &ModuleIr, f: &FunctionIr) -> Result<(), String> {
    let functions = module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect::<HashMap<_, _>>();
    let all = f
        .blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .collect::<Vec<_>>();
    let mut types = f.args.iter().cloned().collect::<HashMap<_, _>>();
    let mut zeros = HashSet::new();
    let mut slots = HashSet::new();
    let mut slot_kinds = HashMap::new();
    let mut constructions = HashMap::new();
    let mut views = HashSet::new();
    let mut pin_values = HashMap::new();
    let mut load_sources = HashMap::new();
    for i in &all {
        match i {
            Instruction::Ownership(Operation::ViewSlot { slot, .. }) => {
                views.insert(*slot);
            }
            Instruction::Ownership(Operation::StoreOwned { slot, value, .. }) => {
                pin_values
                    .entry(*slot)
                    .or_insert_with(HashSet::new)
                    .insert(*value);
            }
            Instruction::Ownership(Operation::Construction { slot, fields }) => {
                constructions.insert(*slot, fields.clone());
            }
            Instruction::ConstInt { dest, value: 0 } => {
                zeros.insert(*dest);
            }
            Instruction::Ownership(Operation::OwnerSlot { slot, kind, .. }) => {
                slots.insert(*slot);
                slot_kinds.insert(*slot, *kind);
            }
            Instruction::Load { dest, src_ptr, .. } => {
                load_sources.insert(*dest, *src_ptr);
            }
            Instruction::Ownership(Operation::ReadValue { dest, source, .. }) => {
                load_sources.insert(*dest, *source);
            }
            _ => {}
        }
    }
    loop {
        let old = (types.len(), zeros.len());
        for i in &all {
            let ty = match i {
                Instruction::ConstInt { .. } => Some(IrType::I64),
                Instruction::Ownership(Operation::OwnedCast { target_ty, .. }) => {
                    Some(target_ty.clone())
                }
                Instruction::ConstBool { .. } => Some(IrType::Bool),
                Instruction::ConstFloat { ty, .. } => Some(ty.clone()),
                Instruction::ConstString { .. } => Some(IrType::Ptr(Box::new(IrType::I8))),
                Instruction::Alloca { ty, .. } => Some(IrType::Ptr(Box::new(ty.clone()))),
                Instruction::Load { ty, .. }
                | Instruction::Ownership(
                    Operation::ReadValue { ty, .. }
                    | Operation::BorrowValue { ty, .. }
                    | Operation::SwapValue { ty, .. },
                ) => Some(ty.clone()),
                Instruction::Ownership(Operation::Resource { ty, action, .. }) => {
                    Some(if matches!(action, ResourceAction::Borrow(_)) {
                        IrType::Ptr(Box::new(ty.clone()))
                    } else {
                        ty.clone()
                    })
                }
                Instruction::AllocStruct { class_name, .. } => Some(IrType::Ptr(Box::new(
                    IrType::Struct(class_name.clone(), vec![]),
                ))),
                Instruction::AllocArray { ty, .. } => Some(IrType::Array(0, Box::new(ty.clone()))),
                Instruction::MakeFatPtr { .. } => Some(IrType::FatPtr),
                Instruction::MakeClosure { .. } => {
                    Some(IrType::Closure(vec![], Box::new(IrType::Void)))
                }
                Instruction::Cast {
                    dest,
                    value,
                    target_ty,
                    ..
                } => {
                    if zeros.contains(value) {
                        zeros.insert(*dest);
                    }
                    Some(target_ty.clone())
                }
                Instruction::Call { func_name, .. } => functions
                    .get(func_name.as_str())
                    .map(|f| f.ret_type.clone()),
                Instruction::DynamicCall { ret_type, .. }
                | Instruction::CallClosure { ret_type, .. }
                | Instruction::IndirectCall { ret_type, .. } => Some(ret_type.clone()),
                _ => None,
            };
            if let (Some(dest), Some(ty)) = (i.destination(), ty) {
                types.insert(dest, ty);
            }
        }
        if old == (types.len(), zeros.len()) {
            break;
        }
    }
    let fail = |message: String| format!("{}: physical ownership: {message}", f.name);
    let take = |value: VReg, state: &mut Flow, why: &str| -> Result<(), String> {
        if zeros.contains(&value) {
            return Ok(());
        }
        let counts = state.values.get(&value).copied().unwrap_or(1);
        if counts & 1 != 0 {
            return Err(fail(format!("{why} consumes non-owning value {value}")));
        }
        state.values.insert(value, counts >> 1);
        Ok(())
    };
    let produce = |value: VReg, state: &mut Flow, why: &str| -> Result<(), String> {
        if state.values.get(&value).copied().unwrap_or(1) & !1 != 0 {
            return Err(fail(format!("{why} duplicates live owner {value}")));
        }
        state.values.insert(value, 2);
        state.leases.insert(value, 1);
        Ok(())
    };
    let by_id = f
        .blocks
        .iter()
        .map(|b| (b.id, b))
        .collect::<HashMap<_, _>>();
    let mut entry = Flow::default();
    for (r, t) in &f.args {
        if managed(t) {
            entry.values.insert(*r, 2);
        }
    }
    for s in &slots {
        entry.cells.insert(*s, 1);
    }
    let mut incoming = HashMap::from([(f.blocks[0].id, entry)]);
    let mut work = VecDeque::from([f.blocks[0].id]);
    while let Some(id) = work.pop_front() {
        let mut state = incoming[&id].clone();
        for i in &by_id[&id].instructions {
            match i {
                Instruction::Alloca { dest, .. } if slots.contains(dest) => {
                    if state.cells.get(dest).copied().unwrap_or(1) & 2 != 0 {
                        return Err(fail(format!(
                            "cleanup slot {dest} reentered while still owning a value"
                        )));
                    }
                    state.cells.insert(*dest, 1);
                }
                Instruction::Store { ty, ptr, .. } if managed(ty) && !views.contains(ptr) => {
                    return Err(fail("owning memory write lacks StoreOwned contract".into()))
                }
                Instruction::Ownership(op) => match op {
                    Operation::Construction { slot, fields } => {
                        if state.cells.get(slot) != Some(&2) {
                            return Err(fail(format!("construction has no shell owner {slot}")));
                        }
                        for field in fields.iter().flatten() {
                            state.progress.insert((*slot, *field), 1);
                        }
                    }
                    Operation::InitializeField {
                        owner,
                        index,
                        value,
                        ..
                    } => {
                        if state.cells.get(owner) != Some(&2) {
                            return Err(fail(format!(
                                "field initialization has no shell owner {owner}"
                            )));
                        }
                        if state.leases.get(value).copied().unwrap_or(1) & 2 != 0 {
                            return Err(fail(
                                "physical lease cannot initialize an owning field".into(),
                            ));
                        }
                        take(*value, &mut state, "construction field transfer")?;
                        if let Some(index) = index {
                            if constructions
                                .get(owner)
                                .and_then(|f| f.as_ref())
                                .is_some_and(|f| !f.contains(index))
                            {
                                return Err(fail("field outside construction recipe".into()));
                            }
                            if state.progress.get(&(*owner, *index)).copied().unwrap_or(1) & 2 != 0
                            {
                                return Err(fail(
                                    "duplicate initialized construction field".into(),
                                ));
                            }
                            state.progress.insert(
                                (*owner, *index),
                                if zeros.contains(value) { 1 } else { 2 },
                            );
                        }
                    }
                    Operation::StoreOwned { slot, value, .. } => {
                        let lease = state.leases.get(value).copied().unwrap_or(1);
                        if lease & 2 != 0 && slot_kinds.get(slot) != Some(&SlotKind::Temporary) {
                            return Err(fail(
                                "physical evaluation lease cannot initialize a logical owner"
                                    .into(),
                            ));
                        }
                        state.cell_leases.insert(*slot, lease);
                        if slots.contains(slot)
                            && state.cells.get(slot).copied().unwrap_or(1) & 2 != 0
                        {
                            return Err(fail(format!(
                                "overwrite of live owner slot {slot} without DropValue"
                            )));
                        }
                        take(*value, &mut state, "StoreOwned")?;
                        if slots.contains(slot) {
                            state
                                .cells
                                .insert(*slot, if zeros.contains(value) { 1 } else { 2 });
                        }
                    }
                    Operation::ReadValue {
                        dest,
                        source,
                        ty,
                        mode: ReadMode::Move,
                    } => {
                        if views.contains(source) {
                            return Err(fail("transfer from a non-owning forwarding view".into()));
                        }
                        if slots.contains(source) {
                            if state.cells.get(source) != Some(&2) {
                                return Err(fail(format!(
                                    "take of non-owning or conditional slot {source}"
                                )));
                            }
                            state.cells.insert(*source, 1);
                        }
                        if managed(ty) {
                            produce(*dest, &mut state, "MoveValue")?;
                            let role = state.cell_leases.get(source).copied().unwrap_or(1);
                            state.leases.insert(*dest, role);
                        }
                    }
                    Operation::TransferSlot { slot, value, .. } => {
                        if !pin_values
                            .get(slot)
                            .is_some_and(|values| values.contains(value))
                        {
                            return Err(fail(
                                "cached transfer value does not belong to its slot".into(),
                            ));
                        }
                        if let Some(Some(fields)) = constructions.get(slot) {
                            if fields
                                .iter()
                                .any(|i| state.progress.get(&(*slot, *i)) != Some(&2))
                            {
                                return Err(fail(format!("construction {slot} transferred before every managed field initialized")));
                            }
                        }
                        for ((owner, _), v) in &mut state.progress {
                            if owner == slot {
                                *v = 1;
                            }
                        }
                        if state.cells.get(slot) != Some(&2) {
                            return Err(fail(format!(
                                "transfer of non-owning or conditional slot {slot}"
                            )));
                        }
                        state.cells.insert(*slot, 1);
                        produce(*value, &mut state, "TransferSlot")?;
                        let role = state.cell_leases.get(slot).copied().unwrap_or(1);
                        state.leases.insert(*value, role);
                    }
                    Operation::DropSlot { slot, .. } => {
                        if slots.contains(slot) {
                            state.cells.insert(*slot, 1);
                        }
                        for ((owner, _), v) in &mut state.progress {
                            if owner == slot {
                                *v = 1;
                            }
                        }
                    }
                    Operation::OwnedCast {
                        dest,
                        value,
                        target_ty,
                        ..
                    } => {
                        take(*value, &mut state, "owned representation transfer")?;
                        if managed(target_ty) {
                            let role = state.leases.get(value).copied().unwrap_or(1);
                            produce(*dest, &mut state, "owned representation transfer")?;
                            state.leases.insert(*dest, role);
                        }
                    }
                    Operation::DropValue { value, recipe } => match recipe {
                        DropRecipe::Managed => {
                            if state.leases.get(value).copied().unwrap_or(1) & 2 != 0 {
                                return Err(fail(
                                    "semantic drop applied to a physical lease".into(),
                                ));
                            }
                            take(*value, &mut state, "DropValue")?;
                        }
                        DropRecipe::Temporary => take(*value, &mut state, "temporary result end")?,
                        DropRecipe::Trivial if types.get(value).is_none_or(managed) => {
                            return Err(fail("invalid trivial drop recipe".into()))
                        }
                        DropRecipe::Trivial => {}
                    },
                    Operation::AcquireSharedEdge { value, kind } => {
                        if *kind == EdgeKind::ImmutableBacking {
                            let authorized = matches!(types.get(value),Some(IrType::Ptr(t)) if matches!(t.as_ref(),IrType::Struct(name,_) if module.shared_types.contains(name)));
                            if !authorized {
                                return Err(fail(
                                    "immutable shared edge lacks a private shared-type contract"
                                        .into(),
                                ));
                            }
                        }
                        state
                            .leases
                            .insert(*value, if *kind == EdgeKind::Evaluation { 2 } else { 1 });
                        if !types.get(value).is_some_and(managed) {
                            return Err(fail(format!(
                                "strong edge for non-managed/borrow representation {value}"
                            )));
                        }
                        if !zeros.contains(value) {
                            let old = state.values.get(value).copied().unwrap_or(1);
                            if old & 8 != 0 {
                                return Err(fail("unbounded strong-edge accumulation".into()));
                            }
                            state.values.insert(*value, old << 1);
                        }
                    }
                    Operation::SwapValue {
                        dest,
                        slot,
                        value,
                        ty,
                    } => {
                        if managed(ty) {
                            if state.leases.get(value).copied().unwrap_or(1) & 2 != 0 {
                                return Err(fail(
                                    "physical lease cannot replace a logical owner".into(),
                                ));
                            }
                            take(*value, &mut state, "SwapValue")?;
                            produce(*dest, &mut state, "SwapValue")?;
                            if slots.contains(slot) {
                                state.cells.insert(*slot, 2);
                            }
                        }
                    }
                    Operation::Resource {
                        resource,
                        owner,
                        ty,
                        action,
                        ..
                    } if module.resources.contains_key(resource) => {
                        let expected = IrType::Ptr(Box::new(IrType::Ptr(Box::new(
                            IrType::Struct(resource.clone(), vec![]),
                        ))));
                        if types.get(owner) != Some(&expected) {
                            return Err(fail(
                                "initialized-resource operation has no matching owner capability"
                                    .into(),
                            ));
                        }
                        if managed(ty) {
                            match action {
                                ResourceAction::Initialize(value) => {
                                    take(*value, &mut state, "ResourceInitialize")?
                                }
                                ResourceAction::Take(value) => {
                                    produce(*value, &mut state, "ResourceTake")?
                                }
                                _ => {}
                            }
                        }
                    }
                    Operation::Resource { ty, action, .. } if managed(ty) => match action {
                        ResourceAction::Initialize(value) => {
                            take(*value, &mut state, "ResourceInitialize")?
                        }
                        ResourceAction::Take(value) => produce(*value, &mut state, "ResourceTake")?,
                        _ => {}
                    },
                    _ => {}
                },
                Instruction::AllocStruct { dest, .. } | Instruction::AllocArray { dest, .. } => {
                    produce(*dest, &mut state, "allocation")?
                }
                Instruction::MakeFatPtr { dest, data_ptr, .. } => {
                    if state.leases.get(data_ptr).copied().unwrap_or(1) & 2 != 0 {
                        return Err(fail("physical lease cannot become a trait owner".into()));
                    }
                    take(*data_ptr, &mut state, "trait payload adoption")?;
                    produce(*dest, &mut state, "trait wrapper")?;
                }
                Instruction::MakeClosure { dest, env_ptr, .. } => {
                    take(*env_ptr, &mut state, "environment adoption")?;
                    produce(*dest, &mut state, "closure wrapper")?;
                }
                Instruction::Cast { value, .. }
                    if state.values.get(value).copied().unwrap_or(1) & !1 != 0 =>
                {
                    return Err(fail(
                        "owning representation cast lacks an explicit transfer".into(),
                    ));
                }
                Instruction::Call {
                    dest,
                    func_name,
                    args,
                } => {
                    if let Some(target) = functions.get(func_name.as_str()) {
                        for (value, (_, ty)) in args.iter().zip(&target.args) {
                            if managed(ty) {
                                if state.leases.get(value).copied().unwrap_or(1) & 2 != 0 {
                                    return Err(fail(
                                        "consuming call received an evaluation lease".into(),
                                    ));
                                }
                                take(*value, &mut state, "by-value call")?;
                            }
                        }
                        if managed(&target.ret_type) {
                            produce(*dest, &mut state, "owned call result")?;
                        }
                    }
                }
                Instruction::DynamicCall {
                    dest,
                    args,
                    arg_types,
                    ret_type,
                    ..
                }
                | Instruction::CallClosure {
                    dest,
                    args,
                    arg_types,
                    ret_type,
                    ..
                }
                | Instruction::IndirectCall {
                    dest,
                    args,
                    arg_types,
                    ret_type,
                    ..
                } => {
                    if let Instruction::DynamicCall {
                        receiver_mode: ReceiverMode::Consume,
                        fat_ptr,
                        ..
                    } = i
                    {
                        if state.values.get(fat_ptr) != Some(&2) {
                            return Err(fail(
                                "consuming dynamic receiver lacks its unique wrapper obligation"
                                    .into(),
                            ));
                        }
                    }
                    for (value, ty) in args.iter().zip(arg_types) {
                        if managed(ty) {
                            if state.leases.get(value).copied().unwrap_or(1) & 2 != 0 {
                                return Err(fail(
                                    "consuming indirect call received an evaluation lease".into(),
                                ));
                            }
                            take(*value, &mut state, "indirect by-value call")?;
                        }
                    }
                    if managed(ret_type) {
                        produce(*dest, &mut state, "owned indirect result")?;
                    }
                }
                Instruction::Ret { value } => {
                    if managed(&f.ret_type) {
                        if let Some(value) = value {
                            if state.leases.get(value).copied().unwrap_or(1) & 2 != 0 {
                                return Err(fail("owned return exports a physical lease".into()));
                            }
                            take(*value, &mut state, "owned return")?;
                        }
                    }
                    for (value, counts) in &state.values {
                        if counts & !1 != 0 {
                            return Err(fail(format!(
                                "normal return leaves temporary {value} unresolved"
                            )));
                        }
                    }
                }
                _ => {}
            }
        }
        let successors = match by_id[&id].instructions.last() {
            Some(Instruction::Br { target }) => vec![*target],
            Some(Instruction::CondBr {
                if_true, if_false, ..
            }) => vec![*if_true, *if_false],
            _ => vec![],
        };
        for next in successors {
            let changed = if let Some(old) = incoming.get_mut(&next) {
                old.join(&state)
            } else {
                incoming.insert(next, state.clone());
                true
            };
            if changed {
                work.push_back(next);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/support/physical_ownership_tests.rs"]
mod tests;
