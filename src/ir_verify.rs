//! Structural SSA/CFG validation; LLVM verification additionally checks LLVM types.
use crate::ir::*;
use std::collections::{HashMap, HashSet};
pub fn verify(module: &ModuleIr) -> Result<(), String> {
    let functions: HashMap<_, _> = module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    if functions.len() != module.functions.len() {
        return Err("Duplicate function names".into());
    }
    for f in &module.functions {
        if f.blocks.is_empty() {
            continue;
        }
        let fail = |msg: String| format!("{}: {}", f.name, msg);
        let ids: HashSet<_> = f.blocks.iter().map(|b| b.id).collect();
        if ids.len() != f.blocks.len() {
            return Err(fail("Duplicate block IDs".into()));
        }
        let mut definitions = HashMap::new();
        let mut available: HashSet<_> = module.globals.keys().copied().collect();
        available.extend(f.args.iter().map(|(r, _)| *r));
        let mut predecessors = HashMap::<BlockId, Vec<BlockId>>::new();
        let mut successors = HashMap::<BlockId, Vec<BlockId>>::new();
        for b in &f.blocks {
            for (n, i) in b.instructions.iter().enumerate() {
                let terminal = matches!(
                    i,
                    Instruction::Br { .. }
                        | Instruction::CondBr { .. }
                        | Instruction::Ret { .. }
                        | Instruction::Unreachable
                );
                if terminal != (n + 1 == b.instructions.len()) {
                    return Err(fail(format!("Malformed terminator in {:?}", b.id)));
                }
                if let Some(d) = i.destination() {
                    if definitions.insert(d, (b.id, n)).is_some() {
                        return Err(fail(format!("Duplicate definition {d}")));
                    }
                }
                if let Instruction::Call {
                    func_name, args, ..
                } = i
                {
                    let callee = functions
                        .get(func_name.as_str())
                        .ok_or_else(|| fail(format!("Unknown function {func_name}")))?;
                    if args.len() < callee.args.len()
                        || (!callee.is_variadic && args.len() != callee.args.len())
                    {
                        return Err(fail(format!("Wrong argument count for {func_name}")));
                    }
                }
                if let Instruction::Ret { value } = i {
                    if value.is_none() != (f.ret_type == IrType::Void) {
                        return Err(fail("Return value/signature mismatch".into()));
                    }
                }
            }
            if b.instructions.is_empty() {
                return Err(fail("Empty unterminated block".into()));
            }
            let targets = match b.instructions.last().unwrap() {
                Instruction::Br { target } => vec![*target],
                Instruction::CondBr {
                    if_true, if_false, ..
                } => vec![*if_true, *if_false],
                _ => vec![],
            };
            for t in &targets {
                if !ids.contains(t) {
                    return Err(fail(format!("Missing branch target {t:?}")));
                }
                predecessors.entry(*t).or_default().push(b.id);
            }
            successors.insert(b.id, targets);
        }
        let entry = f.blocks[0].id;
        let mut reachable = HashSet::new();
        let mut work = vec![entry];
        while let Some(b) = work.pop() {
            if reachable.insert(b) {
                work.extend(&successors[&b]);
            }
        }
        let mut dom: HashMap<_, HashSet<_>> = reachable
            .iter()
            .map(|b| {
                (
                    *b,
                    if *b == entry {
                        HashSet::from([entry])
                    } else {
                        reachable.clone()
                    },
                )
            })
            .collect();
        loop {
            let mut changed = false;
            for b in reachable.iter().filter(|b| **b != entry) {
                let mut d = reachable.clone();
                for p in predecessors
                    .get(b)
                    .into_iter()
                    .flatten()
                    .filter(|p| reachable.contains(p))
                {
                    d.retain(|x| dom[p].contains(x));
                }
                d.insert(*b);
                if dom[b] != d {
                    dom.insert(*b, d);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for b in &f.blocks {
            for (n, i) in b.instructions.iter().enumerate() {
                for r in i.operands() {
                    if available.contains(&r) {
                        continue;
                    }
                    let (db, dn) = definitions
                        .get(&r)
                        .ok_or_else(|| fail(format!("Undefined register {r}")))?;
                    if reachable.contains(&b.id)
                        && ((*db == b.id && *dn >= n) || (*db != b.id && !dom[&b.id].contains(db)))
                    {
                        return Err(fail(format!("Definition of {r} does not dominate its use")));
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn module(instructions: Vec<Instruction>) -> ModuleIr {
        ModuleIr {
            name: "test".into(),
            globals: HashMap::new(),
            structs: HashMap::new(),
            vtables: HashMap::new(),
            functions: vec![FunctionIr {
                name: "main".into(),
                args: vec![],
                ret_type: IrType::I64,
                is_variadic: false,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    name: "entry".into(),
                    instructions,
                }],
            }],
        }
    }
    #[test]
    fn missing_register_and_after_terminator_are_rejected() {
        assert!(verify(&module(vec![Instruction::Ret {
            value: Some(VReg(7))
        }]))
        .unwrap_err()
        .contains("Undefined register"));
        assert!(verify(&module(vec![
            Instruction::Ret {
                value: Some(VReg(0))
            },
            Instruction::ConstInt {
                dest: VReg(0),
                value: 0
            }
        ]))
        .unwrap_err()
        .contains("terminator"));
    }
    #[test]
    fn nondominating_definition_is_rejected() {
        let mut m = module(vec![
            Instruction::ConstBool {
                dest: VReg(0),
                value: true,
            },
            Instruction::CondBr {
                cond: VReg(0),
                if_true: BlockId(1),
                if_false: BlockId(2),
            },
        ]);
        m.functions[0].blocks.extend([
            BasicBlock {
                id: BlockId(1),
                name: "then".into(),
                instructions: vec![
                    Instruction::ConstInt {
                        dest: VReg(1),
                        value: 7,
                    },
                    Instruction::Br { target: BlockId(2) },
                ],
            },
            BasicBlock {
                id: BlockId(2),
                name: "exit".into(),
                instructions: vec![Instruction::Ret {
                    value: Some(VReg(1)),
                }],
            },
        ]);
        assert!(verify(&m).unwrap_err().contains("dominate"));
    }
}
