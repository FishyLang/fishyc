//! Canonical, finite semantic type properties. A managed representation does
//! not itself decide whether a logical value may be copied.
use std::collections::{ HashMap, HashSet };
use std::{ cell::RefCell, rc::Rc };
use crate::{ ast::Type, identity::TypeId, type_checker::StructInfo };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Trivial,
    NonCopy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    Scalar,
    Inline,
    Managed,
    Address,
    Reference,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Properties {
    pub copy_kind: CopyKind,
    pub needs_drop: bool,
    pub contains_borrow: bool,
    pub representation: Representation,
}
impl Properties {
    pub fn is_copy(self) -> bool {
        self.copy_kind == CopyKind::Trivial
    }
}

#[derive(Debug, Default)]
struct Canonical {
    ids: HashMap<Type, TypeId>,
    types: Vec<Type>,
    cache: HashMap<TypeId, Properties>,
}
#[derive(Debug, Default, Clone)]
pub struct TypeProperties {
    canonical: Rc<RefCell<Canonical>>,
    fields: HashMap<String, Vec<Type>>,
    frozen: HashSet<String>,
    erased: HashSet<String>,
}
impl TypeProperties {
    pub fn new(nominals: &HashMap<String, StructInfo>) -> Self {
        let fields = nominals
            .iter()
            .map(|(name, info)| {
                let mut fields = info.fields
                    .values()
                    .map(|(_, ty, _)| ty.clone())
                    .collect::<Vec<_>>();
                for case in &info.case_names {
                    fields.extend(info.static_methods[case].0.clone());
                }
                if let Some(element) = &info.resource_element {
                    fields.push(element.clone());
                }
                (name.clone(), fields)
            })
            .collect();
        let frozen = nominals
            .iter()
            .filter(|(_, info)| info.immutable_payload)
            .map(|(name, _)| name.clone())
            .collect();
        let erased = nominals
            .iter()
            .filter(|(_, info)| info.erased_payload)
            .map(|(name, _)| name.clone())
            .collect();
        Self { fields, frozen, erased, ..Self::default() }
    }
    pub fn intern(&mut self, ty: &Type) -> TypeId {
        let mut canonical = self.canonical.borrow_mut();
        if let Some(id) = canonical.ids.get(ty) {
            return *id;
        }
        let id = TypeId(canonical.types.len());
        canonical.types.push(ty.clone());
        canonical.ids.insert(ty.clone(), id);
        id
    }
    pub fn resolve(&self, id: TypeId) -> Option<Type> {
        self.canonical.borrow().types.get(id.0).cloned()
    }
    pub fn contains_exclusive(&self, ty: &Type) -> bool {
        fn visit(p: &TypeProperties, t: &Type, seen: &mut HashSet<Type>) -> bool {
            if !seen.insert(t.clone()) {
                return false;
            }
            match t {
                Type::MutReference(_) | Type::Function(..) => true,
                Type::Reference(_) | Type::Pointer(_) => false,
                Type::Custom(n) =>
                    p.erased.contains(n) ||
                        p.fields.get(n).is_some_and(|fs| fs.iter().any(|t| visit(p, t, seen))),
                Type::Array(_, t) | Type::Slice(t) => visit(p, t, seen),
                Type::Tuple(ts) | Type::Generic(_, ts) => ts.iter().any(|t| visit(p, t, seen)),
                _ => false,
            }
        }
        visit(self, ty, &mut HashSet::new())
    }
    pub fn immutable_payload(&self, ty: &Type) -> bool {
        match ty {
            Type::Reference(t) | Type::MutReference(t) => self.immutable_payload(t),
            Type::Custom(n) => self.frozen.contains(n),
            _ => false,
        }
    }
    pub fn get(&mut self, ty: &Type) -> Properties {
        let id = self.intern(ty);
        if let Some(p) = self.canonical.borrow().cache.get(&id) {
            return *p;
        }
        use Type::*;
        let (copy_kind, needs_drop, representation) = match ty {
            Reference(_) => (CopyKind::Trivial, false, Representation::Reference),
            MutReference(_) => (CopyKind::NonCopy, false, Representation::Reference),
            Pointer(_) | String => (CopyKind::Trivial, false, Representation::Address),
            Tuple(ts) if ts.is_empty() => (CopyKind::Trivial, false, Representation::Scalar),
            Range(_) => (CopyKind::Trivial, false, Representation::Inline),
            Array(..) | Slice(_) | Custom(_) | Generic(..) | Function(..) | Tuple(_) =>
                (CopyKind::NonCopy, true, Representation::Managed),
            TypeVar(_) | Union(_) => (CopyKind::NonCopy, false, Representation::Inline),
            _ => (CopyKind::Trivial, false, Representation::Scalar),
        };
        let contains_borrow = self.contains_borrow(ty, &mut HashSet::new());
        let p = Properties { copy_kind, needs_drop, contains_borrow, representation };
        self.canonical.borrow_mut().cache.insert(id, p);
        p
    }
    fn contains_borrow(&self, ty: &Type, visited: &mut HashSet<Type>) -> bool {
        if !visited.insert(ty.clone()) {
            return false;
        }
        match ty {
            Type::Reference(_) | Type::MutReference(_) => true,
            Type::Pointer(_) => false,
            Type::Array(_, t) | Type::Slice(t) | Type::Range(t) => self.contains_borrow(t, visited),
            Type::Custom(n) =>
                self.erased.contains(n) ||
                    self.fields
                        .get(n)
                        .is_some_and(|fs| fs.iter().any(|t| self.contains_borrow(t, visited))),
            Type::Generic(_, ts) | Type::Tuple(ts) | Type::Union(ts) =>
                ts.iter().any(|t| self.contains_borrow(t, visited)),
            // The signature does not reveal an environment's capture types.
            // Callable values therefore carry an opaque capture-origin component
            // through parameters, aggregates, moves and result summaries.
            Type::Function(..) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_is_independent_of_managed_storage_and_borrow_permission() {
        let mut p = TypeProperties::default();
        assert!(p.get(&Type::I64).is_copy());
        assert!(p.get(&Type::Pointer(Box::new(Type::Custom("String".into())))).is_copy());
        assert!(p.get(&Type::Reference(Box::new(Type::I64))).is_copy());
        let mutable = p.get(&Type::MutReference(Box::new(Type::I64)));
        assert!(!mutable.is_copy());
        assert!(!mutable.needs_drop);
        assert!(mutable.contains_borrow);
        assert!(!p.get(&Type::Array(3, Box::new(Type::I64))).is_copy());
        assert!(p.get(&Type::Custom("String".into())).needs_drop);
    }
    #[test]
    fn recursive_nominal_queries_are_finite_and_find_borrowed_fields() {
        let mut p = TypeProperties::default();
        let node = Type::Custom("Node".into());
        p.fields.insert("Node".into(), vec![node.clone(), Type::Reference(Box::new(Type::U8))]);
        assert!(p.get(&node).contains_borrow);
        assert_eq!(p.intern(&node), p.intern(&node.clone()));
        assert_eq!(p.canonical.borrow().types.len(), 1);
    }
}
