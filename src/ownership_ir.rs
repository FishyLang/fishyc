//! Typed ownership CFG. This precedes physical ARC and uses source places,
//! explicit access modes, lexical scopes, and finite provenance sets.
use std::collections::{ BTreeMap, BTreeSet, VecDeque };
use crate::{ ast::Type, identity::{ BindingId, ExprId, PlaceId, ScopeId, TypeId }, token::Token };

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Projection {
    Field(usize),
    Index(Option<usize>),
    Dereference,
}
pub type Component = Vec<Projection>;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    pub root: BindingId,
    pub path: Component,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    Static,
    Local(Place),
    Parameter(usize, Component),
    Unknown,
}
pub type ReferenceEffects = BTreeMap<Origin, BTreeSet<Origin>>;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    pub components: BTreeMap<Component, BTreeSet<Origin>>,
    pub loans: BTreeSet<ExprId>,
}
impl Provenance {
    // Recursive references may repeatedly project through a nominal cycle.
    // Widen to an explicit unprovable origin rather than growing an unbounded
    // path or guessing that a prefix denotes the same storage.
    fn normalize(&mut self) {
        const MAX_PATH: usize = 8;
        if
            self.components.iter().any(
                |(path, origins)|
                    path.len() > MAX_PATH ||
                    origins.iter().any(|o| {
                        match o {
                            Origin::Unknown => true,
                            Origin::Local(p) => p.path.len() > MAX_PATH,
                            Origin::Parameter(_, p) => p.len() > MAX_PATH,
                            _ => false,
                        }
                    })
            )
        {
            self.components = BTreeMap::from([(vec![], BTreeSet::from([Origin::Unknown]))]);
        }
    }
    pub fn join(&mut self, other: &Self) -> bool {
        let before = self.clone();
        for (p, origins) in &other.components {
            self.components.entry(p.clone()).or_default().extend(origins.clone());
        }
        self.loans.extend(&other.loans);
        self.normalize();
        *self != before
    }
    pub fn all_origins(&self) -> BTreeSet<Origin> {
        self.components.values().flatten().cloned().collect()
    }
    pub fn field(&self, projection: &Projection) -> Self {
        if self.all_origins().contains(&Origin::Unknown) {
            return self.clone();
        }
        let mut components = BTreeMap::<Component, BTreeSet<Origin>>::new();
        for (path, origins) in &self.components {
            let compatible =
                path.first() == Some(projection) ||
                matches!(
                    (path.first(), projection),
                    (Some(Projection::Index(None)), Projection::Index(_)) |
                        (Some(Projection::Index(_)), Projection::Index(None))
                );
            if compatible {
                components
                    .entry(path[1..].to_vec())
                    .or_default()
                    .extend(origins.clone());
            }
            // Opaque callable/constructor contracts conservatively attach the
            // union to the whole value. Every reference-bearing projection
            // must preserve that bound until a component contract narrows it.
            if path.is_empty() {
                components
                    .entry(vec![])
                    .or_default()
                    .extend(origins.clone());
            }
        }
        let mut result = Self { components, loans: self.loans.clone() };
        result.normalize();
        result
    }
    pub fn at(&self, component: Component) -> Self {
        let components = self.components
            .iter()
            .map(|(path, origins)| {
                let mut p = component.clone();
                p.extend(path.clone());
                (p, origins.clone())
            })
            .collect();
        let mut result = Self { components, loans: self.loans.clone() };
        result.normalize();
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallMode {
    Copy,
    Consume,
    Shared,
    Exclusive,
    Raw,
}
impl CallMode {
    pub fn for_type(ty: &Type, copy: bool) -> Self {
        match ty {
            Type::Reference(_) => Self::Shared,
            Type::MutReference(_) => Self::Exclusive,
            Type::Pointer(_) | Type::String => Self::Raw,
            _ if copy => Self::Copy,
            _ => Self::Consume,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Read,
    Consume,
    Write,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoanKind {
    Shared,
    Exclusive,
}
#[derive(Debug, Clone)]
pub struct BindingInfo {
    pub id: BindingId,
    pub declaration: Token,
    pub ty: TypeId,
    pub mutable: bool,
    pub scope: ScopeId,
    pub owning: bool,
    pub global: bool,
}
#[derive(Debug, Clone)]
pub struct PlaceInfo {
    pub id: PlaceId,
    pub place: Place,
    pub writable: bool,
    /// A dereference through this value may exercise its loan; that permission
    /// does not permit access through other aliases of the same root.
    pub through: Option<ExprId>,
    pub raw: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitState(pub u8);
impl InitState {
    pub const UNINITIALIZED: Self = Self(1);
    pub const LIVE: Self = Self(2);
    pub const MOVED: Self = Self(4);
    pub fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loan {
    pub id: ExprId,
    pub origins: BTreeSet<Origin>,
    pub kind: LoanKind,
    pub scopes: BTreeSet<ScopeId>,
    pub parents: BTreeSet<ExprId>,
}
#[derive(Debug, Clone)]
pub enum Operation {
    Reject {
        span: Token,
        message: String,
    },
    Declare {
        binding: BindingId,
        initialized: bool,
        provenance: Provenance,
    },
    Parameter {
        binding: BindingId,
        provenance: Provenance,
        loans: Vec<Loan>,
    },
    Access {
        place: PlaceId,
        access: Access,
        value: ExprId,
        span: Token,
    },
    Borrow {
        place: PlaceId,
        kind: LoanKind,
        value: ExprId,
        scope: ScopeId,
        span: Token,
    },
    Bind {
        binding: BindingId,
        value: ExprId,
        initialization: bool,
        span: Token,
    },
    Assign {
        place: PlaceId,
        value: ExprId,
        span: Token,
    },
    StoreReference {
        destination: ExprId,
        value: ExprId,
        span: Token,
    },
    Value {
        value: ExprId,
        sources: Vec<(ExprId, Component)>,
    },
    Project {
        value: ExprId,
        source: ExprId,
        projection: Projection,
    },
    Contents {
        value: ExprId,
        reference: ExprId,
    },
    View {
        value: ExprId,
        source: ExprId,
        projection: Projection,
    },
    Call {
        result: ExprId,
        arguments: Vec<ExprId>,
        summary: Provenance,
        target: Option<String>,
        effects: BTreeMap<BindingId, bool>,
        reference_effects: ReferenceEffects,
        effect_loan: ExprId,
        effect_kind: LoanKind,
        result_loan: Option<LoanKind>,
        scope: ScopeId,
        span: Token,
    },
    Return {
        value: Option<ExprId>,
        span: Token,
    },
    EndScope(ScopeId),
    EndLoan(ExprId),
}
#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(usize),
    Branch(usize, usize),
    Return,
    Abort,
}
#[derive(Debug, Clone)]
pub struct Block {
    pub operations: Vec<Operation>,
    pub terminator: Terminator,
}
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub bindings: BTreeMap<BindingId, BindingInfo>,
    pub places: Vec<PlaceInfo>,
    pub scopes: BTreeMap<ScopeId, Option<ScopeId>>,
    pub blocks: Vec<Block>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    pub initialized: BTreeMap<BindingId, InitState>,
    pub bindings: BTreeMap<BindingId, Provenance>,
    pub values: BTreeMap<ExprId, Provenance>,
    pub loans: BTreeMap<ExprId, Loan>,
}
impl State {
    pub fn join(&mut self, other: &Self) -> bool {
        let before = self.clone();
        // A declaration absent on a predecessor is uninitialized, never Live.
        let roots = self.initialized
            .keys()
            .chain(other.initialized.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        for root in roots {
            let a = self.initialized.get(&root).copied().unwrap_or(InitState::UNINITIALIZED);
            let b = other.initialized.get(&root).copied().unwrap_or(InitState::UNINITIALIZED);
            self.initialized.insert(root, a.join(b));
        }
        for (id, p) in &other.bindings {
            self.bindings.entry(*id).or_default().join(p);
        }
        for (id, p) in &other.values {
            self.values.entry(*id).or_default().join(p);
        }
        for (id, loan) in &other.loans {
            self.loans
                .entry(*id)
                .and_modify(|l| {
                    l.origins.extend(loan.origins.clone());
                    l.scopes.extend(&loan.scopes);
                    l.parents.extend(&loan.parents);
                })
                .or_insert_with(|| loan.clone());
        }
        *self != before
    }
}
#[derive(Debug, Clone)]
pub struct OwnershipError {
    pub span: Token,
    pub message: String,
    pub note: Option<Token>,
}
#[derive(Debug, Default)]
pub struct Analysis {
    pub incoming: Vec<Option<State>>,
    pub errors: Vec<OwnershipError>,
    pub returned: Provenance,
    pub stores: ReferenceEffects,
}

impl Function {
    #[cfg(test)]
    pub fn entry_trace(&self) -> String {
        let mut state = State::default();
        let mut text = String::new();
        for op in &self.blocks[0].operations {
            let mut analysis = Analysis::default();
            self.transfer(op, &mut state, &mut analysis);
            text.push_str(&format!("{op:?}\nloans={:?}\n", state.loans));
        }
        text
    }
    pub fn verify(&self) -> Result<(), String> {
        if self.blocks.is_empty() {
            return Err("ownership function has no entry".into());
        }
        for (n, p) in self.places.iter().enumerate() {
            if p.id != PlaceId(n) || !self.bindings.contains_key(&p.place.root) {
                return Err("invalid ownership place".into());
            }
        }
        for (id, b) in &self.bindings {
            if b.id != *id || !self.scopes.contains_key(&b.scope) {
                return Err("invalid binding identity/scope".into());
            }
        }
        for block in &self.blocks {
            let targets = match block.terminator {
                Terminator::Goto(t) => vec![t],
                Terminator::Branch(a, b) => vec![a, b],
                _ => vec![],
            };
            if targets.iter().any(|t| *t >= self.blocks.len()) {
                return Err("missing ownership CFG target".into());
            }
            for op in &block.operations {
                let p = match op {
                    | Operation::Access { place, .. }
                    | Operation::Borrow { place, .. }
                    | Operation::Assign { place, .. } => Some(*place),
                    _ => None,
                };
                if p.is_some_and(|p| p.0 >= self.places.len()) {
                    return Err("unknown ownership place".into());
                }
            }
        }
        Ok(())
    }
    pub fn analyze(&self) -> Analysis {
        self.verify().expect("verified ownership CFG construction");
        let mut incoming = vec![None;self.blocks.len()];
        incoming[0] = Some(State::default());
        let mut work = VecDeque::from([0]);
        // First solve the monotone finite dataflow. Diagnostics only inspect the
        // final states: intermediate worklist order cannot suppress a moved path.
        while let Some(id) = work.pop_front() {
            let mut state = incoming[id].clone().unwrap();
            for op in &self.blocks[id].operations {
                self.transfer(op, &mut state, &mut Analysis::default());
            }
            let successors = match self.blocks[id].terminator {
                Terminator::Goto(t) => vec![t],
                Terminator::Branch(a, b) => vec![a, b],
                _ => vec![],
            };
            for target in successors {
                let changed = if let Some(old) = &mut incoming[target] {
                    old.join(&state)
                } else {
                    incoming[target] = Some(state.clone());
                    true
                };
                if changed && !work.contains(&target) {
                    work.push_back(target);
                }
            }
        }
        let mut result = Analysis { incoming: incoming.clone(), ..Analysis::default() };
        for (id, entry) in incoming.into_iter().enumerate() {
            if let Some(mut state) = entry {
                for op in &self.blocks[id].operations {
                    self.transfer(op, &mut state, &mut result);
                }
            }
        }
        result
    }
    fn error(&self, result: &mut Analysis, span: &Token, message: String, root: Option<BindingId>) {
        if !result.errors.iter().any(|e| e.span.id == span.id && e.message == message) {
            result.errors.push(OwnershipError {
                span: span.clone(),
                message,
                note: root.and_then(|r| self.bindings.get(&r).map(|b| b.declaration.clone())),
            });
        }
    }
    fn origins(&self, place: &PlaceInfo, state: &State) -> BTreeSet<Origin> {
        if let Some(value) = place.through {
            if let Some(p) = state.values.get(&value) {
                return p.all_origins();
            }
        }
        BTreeSet::from([
            if self.bindings[&place.place.root].global {
                Origin::Static
            } else {
                Origin::Local(place.place.clone())
            },
        ])
    }
    fn provenance(&self, place: &PlaceInfo, state: &State) -> Provenance {
        if let Some(reference) = place.through.and_then(|id| state.values.get(&id)) {
            let rest = place.place.path
                .iter()
                .skip_while(|p| **p != Projection::Dereference)
                .skip(1)
                .cloned()
                .collect::<Vec<_>>();
            let mut result = Provenance::default();
            for origin in reference.all_origins() {
                match origin {
                    Origin::Local(mut target) => {
                        target.path.extend(rest.clone());
                        let mut p = state.bindings.get(&target.root).cloned().unwrap_or_default();
                        for projection in &target.path {
                            p = p.field(projection);
                        }
                        result.join(&p);
                    }
                    Origin::Parameter(index, path) => {
                        // A projection through a reference retains the lifetime
                        // of its referent root. It is not another stored-reference
                        // component of the function argument.
                        result.components
                            .entry(vec![])
                            .or_default()
                            .insert(Origin::Parameter(index, path));
                    }
                    Origin::Static => {
                        result.components
                            .entry(vec![])
                            .or_default()
                            .insert(Origin::Static);
                    }
                    Origin::Unknown => {
                        result.components
                            .entry(vec![])
                            .or_default()
                            .insert(Origin::Unknown);
                    }
                }
            }
            return result;
        }
        let mut p = state.bindings.get(&place.place.root).cloned().unwrap_or_default();
        for projection in &place.place.path {
            p = p.field(projection);
        }
        p
    }
    fn live(&self, place: &PlaceInfo, state: &State, span: &Token, result: &mut Analysis) {
        let binding = &self.bindings[&place.place.root];
        let initialized = state.initialized
            .get(&binding.id)
            .copied()
            .unwrap_or(InitState::UNINITIALIZED);
        if initialized != InitState::LIVE {
            let message = match initialized {
                InitState::MOVED => format!("value '{}' was moved", binding.declaration.lexeme),
                InitState::UNINITIALIZED =>
                    format!("value '{}' is not initialized", binding.declaration.lexeme),
                s if (s.0 & InitState::MOVED.0) != 0 =>
                    format!(
                        "value '{}' may have been moved in this branch",
                        binding.declaration.lexeme
                    ),
                _ => format!("value '{}' may be uninitialized", binding.declaration.lexeme),
            };
            self.error(result, span, message, Some(binding.id));
        }
    }
    fn conflicts(&self, p: &PlaceInfo, kind: LoanKind, state: &State) -> bool {
        let origins = self.origins(p, state);
        let mut permitted = p.through
            .and_then(|id| state.values.get(&id))
            .map(|v| v.loans.clone())
            .unwrap_or_default();
        let mut work = permitted.iter().copied().collect::<Vec<_>>();
        while let Some(id) = work.pop() {
            if let Some(loan) = state.loans.get(&id) {
                for parent in &loan.parents {
                    if permitted.insert(*parent) {
                        work.push(*parent);
                    }
                }
            }
        }
        state.loans.values().any(|loan| {
            if permitted.contains(&loan.id) {
                return false;
            }
            let overlap = origins.iter().any(|a|
                loan.origins.iter().any(|b| {
                    match (a, b) {
                        (Origin::Local(a), Origin::Local(b)) => a.root == b.root,
                        (Origin::Parameter(a, _), Origin::Parameter(b, _)) => a == b,
                        (Origin::Static, Origin::Static) => true,
                        _ => false,
                    }
                })
            );
            overlap && (kind == LoanKind::Exclusive || loan.kind == LoanKind::Exclusive)
        })
    }
    fn extend_loans(&self, p: &Provenance, scope: ScopeId, state: &mut State) {
        for id in &p.loans {
            if let Some(loan) = state.loans.get_mut(id) {
                loan.scopes.insert(scope);
            }
        }
    }
    fn escapes(
        &self,
        p: &Provenance,
        destination: Option<ScopeId>,
        state: &State
    ) -> Option<BindingId> {
        for origin in p.all_origins() {
            if let Origin::Local(place) = origin {
                let owner_scope = self.bindings[&place.root].scope;
                let mut current = destination;
                let mut valid = false;
                while let Some(scope) = current {
                    if scope == owner_scope {
                        valid = true;
                        break;
                    }
                    current = self.scopes[&scope];
                }
                if !valid {
                    return Some(place.root);
                }
                if state.initialized.get(&place.root) != Some(&InitState::LIVE) {
                    return Some(place.root);
                }
            }
        }
        None
    }
    fn store_reference(
        &self,
        destinations: BTreeSet<Origin>,
        source: &Provenance,
        state: &mut State,
        result: &mut Analysis,
        span: &Token
    ) {
        for destination in destinations {
            if source.all_origins().contains(&destination) {
                self.error(
                    result,
                    span,
                    "cannot store a reference into its own borrowed storage".into(),
                    None
                );
            }
            match destination {
                Origin::Local(target) => {
                    let scope = self.bindings[&target.root].scope;
                    if let Some(root) = self.escapes(source, Some(scope), state) {
                        self.error(
                            result,
                            span,
                            format!(
                                "reference to '{}' escapes its owner",
                                self.bindings[&root].declaration.lexeme
                            ),
                            Some(root)
                        );
                    }
                    self.extend_loans(source, scope, state);
                    state.bindings.entry(target.root).or_default().join(source);
                }
                Origin::Parameter(..) => {
                    let scope = *self.scopes
                        .iter()
                        .find(|(_, parent)| parent.is_none())
                        .unwrap().0;
                    self.extend_loans(source, scope, state);
                    if let Some(root) = self.escapes(source, None, state) {
                        self.error(
                            result,
                            span,
                            format!(
                                "reference to '{}' escapes its owner",
                                self.bindings[&root].declaration.lexeme
                            ),
                            Some(root)
                        );
                    }
                    result.stores
                        .entry(destination)
                        .or_default()
                        .extend(
                            source
                                .all_origins()
                                .into_iter()
                                .filter(|o| !matches!(o, Origin::Local(_)))
                        );
                }
                Origin::Static => {
                    if
                        source
                            .all_origins()
                            .iter()
                            .any(|o| *o != Origin::Static)
                    {
                        self.error(
                            result,
                            span,
                            "reference stored in global storage must have a static origin".into(),
                            None
                        );
                    }
                }
                Origin::Unknown =>
                    self.error(
                        result,
                        span,
                        "cannot prove this reference remains valid".into(),
                        None
                    ),
            }
        }
    }
    fn referenced_contents(&self, reference: &Provenance, state: &State) -> Provenance {
        let mut contents = Provenance::default();
        for origin in reference.all_origins() {
            match origin {
                Origin::Local(place) => {
                    let mut p = state.bindings.get(&place.root).cloned().unwrap_or_default();
                    for projection in &place.path {
                        p = p.field(projection);
                    }
                    contents.join(&p);
                }
                Origin::Parameter(index, mut path) => {
                    path.push(Projection::Dereference);
                    contents.components
                        .entry(vec![])
                        .or_default()
                        .insert(Origin::Parameter(index, path));
                }
                Origin::Static => {
                    contents.components
                        .entry(vec![])
                        .or_default()
                        .insert(Origin::Static);
                }
                Origin::Unknown => {
                    contents.components
                        .entry(vec![])
                        .or_default()
                        .insert(Origin::Unknown);
                }
            }
        }
        contents.normalize();
        contents
    }
    fn substitute(&self, origin: &Origin, arguments: &[ExprId], state: &State) -> Provenance {
        if let Origin::Parameter(index, component) = origin {
            if let Some(original) = arguments.get(*index).and_then(|id| state.values.get(id)) {
                let mut arg = original.clone();
                for projection in component {
                    arg = if *projection == Projection::Dereference {
                        self.referenced_contents(&arg, state)
                    } else {
                        arg.field(projection)
                    };
                }
                if
                    arg.components.is_empty() &&
                    !component.is_empty() &&
                    !component.contains(&Projection::Dereference)
                {
                    arg = original.clone();
                }
                // A summary origin denotes the bounds of one result component,
                // not the input's nested physical shape. Preserve its union at
                // the result component even when a constructor/callable changes
                // the aggregate shape around the borrowed value.
                let origins = arg.all_origins();
                return Provenance {
                    components: if origins.is_empty() {
                        BTreeMap::new()
                    } else {
                        BTreeMap::from([(vec![], origins)])
                    },
                    loans: arg.loans,
                };
            }
            return Provenance {
                components: BTreeMap::from([(vec![], BTreeSet::from([Origin::Unknown]))]),
                loans: BTreeSet::new(),
            };
        }
        Provenance {
            components: BTreeMap::from([(vec![], BTreeSet::from([origin.clone()]))]),
            loans: BTreeSet::new(),
        }
    }
    fn transfer(&self, op: &Operation, state: &mut State, result: &mut Analysis) {
        match op {
            Operation::Reject { span, message } => self.error(result, span, message.clone(), None),
            Operation::Declare { binding, initialized, provenance } => {
                state.initialized.insert(*binding, if *initialized {
                    InitState::LIVE
                } else {
                    InitState::UNINITIALIZED
                });
                state.bindings.insert(*binding, provenance.clone());
            }
            Operation::Parameter { binding, provenance, loans } => {
                state.bindings.insert(*binding, provenance.clone());
                for loan in loans {
                    state.loans.insert(loan.id, loan.clone());
                }
            }
            Operation::Access { place, access, value, span } => {
                let p = &self.places[place.0];
                let root = p.place.root;
                let b = &self.bindings[&root];
                self.live(p, state, span, result);
                let capability = state.bindings
                    .get(&root)
                    .map(|p| p.loans.clone())
                    .unwrap_or_default();
                if
                    state.loans
                        .values()
                        .any(
                            |l|
                                !capability.contains(&l.id) &&
                                (*access != Access::Read || l.kind == LoanKind::Exclusive) &&
                                l.parents
                                    .iter()
                                    .any(
                                        |id|
                                            capability.contains(id) &&
                                            state.loans
                                                .get(id)
                                                .is_some_and(
                                                    |parent| parent.kind == LoanKind::Exclusive
                                                )
                                    )
                        )
                {
                    self.error(
                        result,
                        span,
                        format!("mutable access to '{}' is already borrowed", b.declaration.lexeme),
                        Some(root)
                    );
                }
                if
                    self.conflicts(
                        p,
                        if *access == Access::Read {
                            LoanKind::Shared
                        } else {
                            LoanKind::Exclusive
                        },
                        state
                    )
                {
                    self.error(
                        result,
                        span,
                        if *access == Access::Consume {
                            format!("cannot move '{}' while it is borrowed", b.declaration.lexeme)
                        } else {
                            format!(
                                "cannot access '{}' while mutable access is borrowed",
                                b.declaration.lexeme
                            )
                        },
                        Some(root)
                    );
                }
                let provenance = self.provenance(p, state);
                state.values.insert(*value, provenance);
                if *access == Access::Consume {
                    if b.global {
                        self.error(
                            result,
                            span,
                            "cannot move an owned value out of global storage".into(),
                            Some(root)
                        );
                    }
                    if !p.place.path.is_empty() {
                        self.error(
                            result,
                            span,
                            format!("cannot move field '{}' out of this value here", span.lexeme),
                            Some(root)
                        );
                    } else {
                        state.initialized.insert(root, InitState::MOVED);
                    }
                }
                if *access == Access::Write && !p.writable {
                    self.error(
                        result,
                        span,
                        format!("cannot modify immutable binding '{}'", b.declaration.lexeme),
                        Some(root)
                    );
                }
            }
            Operation::Borrow { place, kind, value, scope, span } => {
                let p = &self.places[place.0];
                let b = &self.bindings[&p.place.root];
                if p.raw {
                    self.error(
                        result,
                        span,
                        "cannot create a safe reference from raw storage".into(),
                        Some(b.id)
                    );
                }
                self.live(p, state, span, result);
                if *kind == LoanKind::Exclusive && !p.writable {
                    self.error(
                        result,
                        span,
                        format!("cannot modify immutable binding '{}'", b.declaration.lexeme),
                        Some(b.id)
                    );
                }
                if self.conflicts(p, *kind, state) {
                    let message = if *kind == LoanKind::Exclusive {
                        format!(
                            "cannot borrow '{}' mutably while another borrow is active",
                            b.declaration.lexeme
                        )
                    } else {
                        format!("mutable access to '{}' is already borrowed", b.declaration.lexeme)
                    };
                    self.error(result, span, message, Some(b.id));
                }
                let origins = self.origins(p, state);
                let parents = p.through
                    .and_then(|id| state.values.get(&id))
                    .map(|v| v.loans.clone())
                    .unwrap_or_default();
                state.loans.insert(*value, Loan {
                    id: *value,
                    origins: origins.clone(),
                    kind: *kind,
                    scopes: BTreeSet::from([*scope]),
                    parents,
                });
                state.values.insert(*value, Provenance {
                    components: BTreeMap::from([(vec![], origins)]),
                    loans: BTreeSet::from([*value]),
                });
            }
            Operation::Bind { binding, value, initialization, span } => {
                let b = &self.bindings[binding];
                let old = state.initialized
                    .get(binding)
                    .copied()
                    .unwrap_or(InitState::UNINITIALIZED);
                if !*initialization && !b.mutable && old != InitState::UNINITIALIZED {
                    self.error(
                        result,
                        span,
                        format!("cannot reassign immutable binding '{}'", b.declaration.lexeme),
                        Some(*binding)
                    );
                }
                let p = state.values.get(value).cloned().unwrap_or_default();
                if p.all_origins().contains(&Origin::Unknown) {
                    self.error(
                        result,
                        span,
                        "cannot prove this reference remains valid".into(),
                        None
                    );
                }
                if let Some(root) = self.escapes(&p, Some(b.scope), state) {
                    self.error(
                        result,
                        span,
                        format!(
                            "reference to '{}' escapes its owner",
                            self.bindings[&root].declaration.lexeme
                        ),
                        Some(root)
                    );
                }
                self.extend_loans(&p, b.scope, state);
                state.bindings.insert(*binding, p);
                state.initialized.insert(*binding, InitState::LIVE);
            }
            Operation::Assign { place, value, span } => {
                let p = &self.places[place.0];
                let b = &self.bindings[&p.place.root];
                if !p.writable {
                    self.error(
                        result,
                        span,
                        format!("cannot modify immutable binding '{}'", b.declaration.lexeme),
                        Some(b.id)
                    );
                }
                if self.conflicts(p, LoanKind::Exclusive, state) {
                    self.error(
                        result,
                        span,
                        format!(
                            "cannot replace '{}' while a reference to it is active",
                            b.declaration.lexeme
                        ),
                        Some(b.id)
                    );
                }
                let provenance = state.values.get(value).cloned().unwrap_or_default();
                if p.through.is_some() && !provenance.components.is_empty() {
                    self.store_reference(self.origins(p, state), &provenance, state, result, span);
                }
                if let Some(root) = self.escapes(&provenance, Some(b.scope), state) {
                    self.error(
                        result,
                        span,
                        format!(
                            "reference to '{}' escapes its owner",
                            self.bindings[&root].declaration.lexeme
                        ),
                        Some(root)
                    );
                }
                self.extend_loans(&provenance, b.scope, state);
                if p.place.path.is_empty() {
                    state.bindings.insert(b.id, provenance);
                    state.initialized.insert(b.id, InitState::LIVE);
                } else {
                    state.bindings
                        .entry(b.id)
                        .or_default()
                        .join(&provenance.at(p.place.path.clone()));
                }
            }
            Operation::StoreReference { destination, value, span } => {
                let destination = state.values
                    .get(destination)
                    .cloned()
                    .unwrap_or_default()
                    .all_origins();
                let value = state.values.get(value).cloned().unwrap_or_default();
                self.store_reference(destination, &value, state, result, span);
            }
            Operation::Value { value, sources } => {
                let mut p = Provenance::default();
                for (id, path) in sources {
                    if let Some(source) = state.values.get(id) {
                        p.join(&source.at(path.clone()));
                    }
                }
                state.values.insert(*value, p);
            }
            Operation::Contents { value, reference } => {
                let p = state.values.get(reference).cloned().unwrap_or_default();
                state.values.insert(*value, self.referenced_contents(&p, state));
            }
            Operation::Project { value, source, projection } => {
                let p = state.values.get(source).cloned().unwrap_or_default().field(projection);
                state.values.insert(*value, p);
            }
            Operation::View { value, source, projection } => {
                let source = state.values.get(source).cloned().unwrap_or_default();
                let _ = projection;
                let origins = source.all_origins();
                state.values.insert(*value, Provenance {
                    components: BTreeMap::from([(vec![], origins)]),
                    loans: source.loans,
                });
            }
            Operation::Call {
                result: value,
                arguments,
                summary,
                effects,
                reference_effects,
                effect_loan,
                effect_kind,
                result_loan,
                scope,
                span,
                ..
            } => {
                for (root, write) in effects {
                    if let Some(binding) = self.bindings.get(root) {
                        let place = PlaceInfo {
                            id: PlaceId(0),
                            place: Place { root: *root, path: vec![] },
                            writable: binding.mutable,
                            through: None,
                            raw: false,
                        };
                        self.live(&place, state, span, result);
                        if
                            self.conflicts(
                                &place,
                                if *write {
                                    LoanKind::Exclusive
                                } else {
                                    LoanKind::Shared
                                },
                                state
                            )
                        {
                            self.error(
                                result,
                                span,
                                format!(
                                    "call cannot access global '{}' while it is borrowed",
                                    binding.declaration.lexeme
                                ),
                                Some(*root)
                            );
                        }
                    }
                }
                let mut p = Provenance::default();
                for (path, origins) in &summary.components {
                    for origin in origins {
                        p.join(&self.substitute(origin, arguments, state).at(path.clone()));
                    }
                }
                for (destination, sources) in reference_effects {
                    let destinations = self.substitute(destination, arguments, state).all_origins();
                    let mut source = Provenance::default();
                    for origin in sources {
                        source.join(&self.substitute(origin, arguments, state));
                    }
                    let origins = source.all_origins();
                    if !origins.is_empty() {
                        let kind = if
                            *effect_kind == LoanKind::Exclusive &&
                            origins.contains(&Origin::Static) &&
                            !self.bindings.values().any(|b| b.global && b.mutable)
                        {
                            LoanKind::Shared
                        } else {
                            *effect_kind
                        };
                        state.loans
                            .entry(*effect_loan)
                            .and_modify(|l| {
                                l.origins.extend(origins.clone());
                                l.parents.extend(&source.loans);
                            })
                            .or_insert(Loan {
                                id: *effect_loan,
                                origins,
                                kind,
                                scopes: BTreeSet::new(),
                                parents: source.loans.clone(),
                            });
                        source.loans.insert(*effect_loan);
                    }
                    self.store_reference(destinations, &source, state, result, span);
                }
                if let Some(kind) = result_loan {
                    let origins = p.all_origins();
                    if !origins.is_empty() {
                        state.loans.insert(*value, Loan {
                            id: *value,
                            origins,
                            kind: *kind,
                            scopes: BTreeSet::from([*scope]),
                            parents: p.loans.clone(),
                        });
                        p.loans.insert(*value);
                    }
                }
                self.extend_loans(&p, *scope, state);
                state.values.insert(*value, p);
            }
            Operation::Return { value, span } => if let Some(value) = value {
                let p = state.values.get(value).cloned().unwrap_or_default();
                if p.all_origins().contains(&Origin::Unknown) {
                    self.error(
                        result,
                        span,
                        "cannot prove this reference remains valid".into(),
                        None
                    );
                }
                if let Some(root) = self.escapes(&p, None, state) {
                    self.error(
                        result,
                        span,
                        format!(
                            "returned reference points to local storage '{}'",
                            self.bindings[&root].declaration.lexeme
                        ),
                        Some(root)
                    );
                }
                result.returned.join(&p);
            }
            Operation::EndScope(scope) => {
                state.loans.retain(|_, loan| {
                    loan.scopes.remove(scope);
                    !loan.scopes.is_empty()
                });
                for b in self.bindings.values().filter(|b| b.scope == *scope) {
                    state.initialized.remove(&b.id);
                    state.bindings.remove(&b.id);
                }
            }
            Operation::EndLoan(loan) => {
                state.loans.remove(loan);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;
    fn function() -> Function {
        let span = Token::synthetic(TokenType::Identifier, "x");
        let id = BindingId(span.id);
        Function {
            name: "test".into(),
            bindings: BTreeMap::from([
                (
                    id,
                    BindingInfo {
                        id,
                        declaration: span,
                        ty: TypeId(0),
                        mutable: true,
                        scope: ScopeId(0),
                        owning: true,
                        global: false,
                    },
                ),
            ]),
            places: vec![PlaceInfo {
                id: PlaceId(0),
                place: Place { root: id, path: vec![] },
                writable: true,
                through: None,
                raw: false,
            }],
            scopes: BTreeMap::from([(ScopeId(0), None)]),
            blocks: vec![],
        }
    }
    fn access(f: &Function, access: Access) -> Operation {
        Operation::Access {
            place: PlaceId(0),
            access,
            value: ExprId::fresh(),
            span: f.bindings.values().next().unwrap().declaration.clone(),
        }
    }
    fn declare(f: &Function) -> Operation {
        Operation::Declare {
            binding: *f.bindings.keys().next().unwrap(),
            initialized: true,
            provenance: Provenance::default(),
        }
    }
    #[test]
    fn conditional_move_and_returning_branch_have_distinct_joins() {
        let mut f = function();
        f.blocks = vec![
            Block { operations: vec![declare(&f)], terminator: Terminator::Branch(1, 2) },
            Block {
                operations: vec![access(&f, Access::Consume)],
                terminator: Terminator::Goto(2),
            },
            Block { operations: vec![access(&f, Access::Read)], terminator: Terminator::Return }
        ];
        assert!(
            f
                .analyze()
                .errors.iter()
                .any(|e| e.message.contains("may have been moved"))
        );
        f.blocks[1].terminator = Terminator::Return;
        assert!(f.analyze().errors.is_empty());
    }
    #[test]
    fn loop_fixed_point_rejects_repeated_consumption() {
        let mut f = function();
        f.blocks = vec![
            Block { operations: vec![declare(&f)], terminator: Terminator::Goto(1) },
            Block {
                operations: vec![access(&f, Access::Consume)],
                terminator: Terminator::Branch(1, 2),
            },
            Block { operations: vec![], terminator: Terminator::Return }
        ];
        assert!(
            f
                .analyze()
                .errors.iter()
                .any(|e| e.message.contains("may have been moved"))
        );
        let binding = *f.bindings.keys().next().unwrap();
        let span = f.bindings[&binding].declaration.clone();
        f.blocks[1].operations.push(Operation::Bind {
            binding,
            value: ExprId::fresh(),
            initialization: false,
            span,
        });
        assert!(f.analyze().errors.is_empty());
    }
    #[test]
    fn lexical_borrow_prevents_move_until_scope_ends() {
        let mut f = function();
        let span = f.bindings.values().next().unwrap().declaration.clone();
        f.scopes.insert(ScopeId(1), Some(ScopeId(0)));
        f.blocks = vec![Block {
            operations: vec![
                declare(&f),
                Operation::Borrow {
                    place: PlaceId(0),
                    kind: LoanKind::Shared,
                    value: ExprId::fresh(),
                    scope: ScopeId(1),
                    span,
                },
                access(&f, Access::Consume)
            ],
            terminator: Terminator::Return,
        }];
        assert!(
            f
                .analyze()
                .errors.iter()
                .any(|e| e.message.contains("while it is borrowed"))
        );
        f.blocks[0].operations.insert(2, Operation::EndScope(ScopeId(1)));
        assert!(f.analyze().errors.is_empty());
    }
    #[test]
    fn component_provenance_does_not_merge_independent_fields() {
        let mut p = Provenance::default();
        p.components.insert(
            vec![Projection::Field(0)],
            BTreeSet::from([Origin::Parameter(0, vec![])])
        );
        p.components.insert(
            vec![Projection::Field(1)],
            BTreeSet::from([Origin::Parameter(1, vec![])])
        );
        assert_eq!(
            p.field(&Projection::Field(0)).all_origins(),
            BTreeSet::from([Origin::Parameter(0, vec![])])
        );
    }
}
