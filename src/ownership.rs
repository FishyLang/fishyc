//! Ownership elaboration over materialized typed source, before ARC lowering.
use std::collections::{ BTreeMap, BTreeSet, HashMap, HashSet };
use crate::{
    ast::{ Expr, Stmt, Type, FunctionParam },
    captures::CapturePlans,
    identity::{ BindingId, ExprId, PlaceId, ScopeId },
    token::{ Token, TokenType, Literal },
    type_checker::{ StructInfo, CallType, DefaultCall },
    type_properties::TypeProperties,
    ownership_ir::*,
};

#[derive(Debug, Default)]
pub struct Plan {
    pub types: TypeProperties,
    pub functions: Vec<Function>,
    pub consumed: HashSet<ExprId>,
    pub borrowed: HashMap<ExprId, LoanKind>,
    pub borrowed_temporaries: HashSet<ExprId>,
    pub receiver_values: HashSet<ExprId>,
    pub moved_captures: HashSet<(usize, usize)>,
    pub pattern_bindings: HashMap<usize, Type>,
    pub pattern_views: HashSet<usize>,
    pub errors: Vec<OwnershipError>,
}
pub struct Context<'a> {
    pub types: &'a HashMap<String, StructInfo>,
    pub pattern_bindings: &'a HashMap<usize, Type>,
    pub pattern_views: &'a HashSet<usize>,
    pub parametric_bindings: &'a HashSet<usize>,
    pub calls: &'a HashMap<Token, CallType>,
    pub fields: &'a HashMap<Token, usize>,
    pub captures: &'a CapturePlans,
    pub defaults: &'a HashMap<Token, DefaultCall>,
    pub specialized: &'a HashMap<Token, (String, bool)>,
    pub traits: &'a HashSet<String>,
}
impl Plan {
    pub fn checked_program(
        statements: &[Stmt],
        checker: &crate::type_checker::TypeChecker
    ) -> Self {
        Self::build(
            statements,
            &(Context {
                types: &checker.user_types,
                pattern_bindings: &checker.pattern_bindings,
                pattern_views: &checker.pattern_views,
                parametric_bindings: &checker.parametric_bindings,
                calls: &checker.resolved_calls,
                fields: &checker.property_indices,
                captures: &checker.capture_plans,
                defaults: &checker.default_calls,
                specialized: &checker.specialized_methods,
                traits: &checker.traits,
            })
        )
    }
    pub fn build(statements: &[Stmt], context: &Context<'_>) -> Self {
        let properties = TypeProperties::new(context.types);
        fn contracts(
            stmts: &[Stmt],
            owner: Option<&Type>,
            out: &mut HashMap<String, (Vec<Type>, Type, bool)>
        ) {
            for s in stmts {
                match s {
                    Stmt::Function { name, params, return_type, ownership_signature, .. } => {
                        let symbol = if let Some(Type::Custom(owner)) = owner {
                            crate::symbols::member(owner, &name.lexeme)
                        } else {
                            name.lexeme.clone()
                        };
                        let (ps, ret) = ownership_signature.clone().unwrap_or_else(|| (
                            params
                                .iter()
                                .map(|p| p.type_annotation.clone().unwrap_or(Type::Void))
                                .collect(),
                            return_type.clone().unwrap_or(Type::Void),
                        ));
                        out.insert(symbol, (ps, ret, ownership_signature.is_some()));
                    }
                    Stmt::Impl { target_type, methods, .. } =>
                        contracts(methods, Some(target_type), out),
                    _ => {}
                }
            }
        }
        let mut source_contracts = HashMap::new();
        contracts(statements, None, &mut source_contracts);
        let mut plan = Self {
            pattern_bindings: context.pattern_bindings.clone(),
            pattern_views: context.pattern_views.clone(),
            ..Self::default()
        };
        fn safe_boundary(ty: &Type, properties: &mut TypeProperties) -> bool {
            properties.get(ty).contains_borrow ||
                (match ty {
                    Type::Function(ps, ret, _) =>
                        ps.iter().any(|p| safe_boundary(p, properties)) ||
                            safe_boundary(ret, properties),
                    _ => false,
                })
        }
        for stmt in statements {
            if let Stmt::ExternFunction { name, params, return_type, .. } = stmt {
                let mut properties = TypeProperties::new(context.types);
                if
                    params
                        .iter()
                        .filter_map(|p| p.type_annotation.as_ref())
                        .chain(return_type.iter())
                        .any(|t| safe_boundary(t, &mut properties))
                {
                    plan.errors.push(OwnershipError {
                        span: name.clone(),
                        message: "external safe-reference contracts require a declared trusted boundary; use the existing raw-address ABI".into(),
                        note: None,
                    });
                }
            }
        }
        let globals = statements
            .iter()
            .filter_map(|s| (
                if let Stmt::Var { name, mutable, type_annotation, initializer, .. } = s {
                    Some((
                        name.clone(),
                        *mutable,
                        type_annotation
                            .clone()
                            .or_else(||
                                initializer.as_ref().and_then(|e| e.checked_type().cloned())
                            )
                            .unwrap_or(Type::Void),
                        initializer.is_some(),
                    ))
                } else {
                    None
                }
            ))
            .collect::<Vec<_>>();
        for (name, _, ty, _) in &globals {
            if properties.clone().get(ty).contains_borrow {
                plan.errors.push(OwnershipError {
                    span: name.clone(),
                    message: "persistent global storage containing references or opaque captures requires a program-wide loan contract; use lexical storage".into(),
                    note: None,
                });
            }
        }
        fn visit(
            stmts: &[Stmt],
            owner: Option<&Type>,
            globals: &[(Token, bool, Type, bool)],
            context: &Context<'_>,
            contracts: &HashMap<String, (Vec<Type>, Type, bool)>,
            properties: &TypeProperties,
            plan: &mut Plan
        ) {
            for stmt in stmts {
                match stmt {
                    Stmt::Function {
                        name,
                        type_params,
                        params,
                        return_type,
                        ownership_signature,
                        body: Some(body),
                        ..
                    } if type_params.is_empty() => {
                        let mut params = params.clone();
                        if let Some(owner) = owner {
                            if let Some(p) = params.first_mut().filter(|p| p.name.lexeme == "self") {
                                if p.type_annotation.is_none() {
                                    p.type_annotation = Some(
                                        Type::Reference(Box::new(owner.clone()))
                                    );
                                }
                            }
                        }
                        let name = if let Some(Type::Custom(owner)) = owner {
                            crate::symbols::member(owner, &name.lexeme)
                        } else {
                            name.lexeme.clone()
                        };
                        let mut builder = Builder::new(
                            name,
                            context,
                            globals,
                            contracts,
                            properties
                        );
                        builder.parameters(&params);
                        if let Some((source, _)) = ownership_signature {
                            builder.generic_body = true;
                            for (p, ty) in params.iter().zip(source) {
                                builder.source_types.insert(BindingId(p.name.id), ty.clone());
                                if matches!(ty, Type::TypeVar(_)) {
                                    builder.parametric.insert(BindingId(p.name.id));
                                }
                            }
                        }
                        builder.statements(body);
                        let _ = return_type;
                        plan.functions.extend(builder.nested.drain(..));
                        plan.consumed.extend(builder.consumed);
                        plan.borrowed.extend(builder.borrowed);
                        plan.borrowed_temporaries.extend(builder.borrowed_temporaries);
                        plan.receiver_values.extend(builder.receiver_values);
                        plan.moved_captures.extend(builder.moved_captures);
                        plan.functions.push(builder.function);
                    }
                    Stmt::Function { name, params, return_type, body: None, .. } => {
                        let mut properties = properties.clone();
                        if
                            params
                                .iter()
                                .filter_map(|p| p.type_annotation.as_ref())
                                .chain(return_type.iter())
                                .any(|t| safe_boundary(t, &mut properties))
                        {
                            plan.errors.push(OwnershipError {
                                span: name.clone(),
                                message: "external safe-reference contracts require a declared trusted boundary; use the existing raw-address ABI".into(),
                                note: None,
                            });
                        }
                    }
                    Stmt::Impl { type_params, target_type, methods, .. } if type_params.is_empty() =>
                        visit(
                            methods,
                            Some(target_type),
                            globals,
                            context,
                            contracts,
                            properties,
                            plan
                        ),
                    _ => {}
                }
            }
        }
        visit(statements, None, &globals, context, &source_contracts, &properties, &mut plan);
        let mut init = Builder::new(
            "__global_init".into(),
            context,
            &globals,
            &source_contracts,
            &properties
        );
        for block in &mut init.function.blocks {
            for op in &mut block.operations {
                if let Operation::Declare { initialized, .. } = op {
                    *initialized = false;
                }
            }
        }
        for stmt in statements {
            match stmt {
                Stmt::Var { name, initializer: Some(expr), .. } => {
                    let value = init.expression(expr, Use::Consume);
                    let binding = init.resolve(&name.lexeme).unwrap();
                    init.emit(Operation::Bind {
                        binding,
                        value,
                        initialization: true,
                        span: name.clone(),
                    });
                }
                | Stmt::Var { .. }
                | Stmt::Function { .. }
                | Stmt::ExternFunction { .. }
                | Stmt::Struct { .. }
                | Stmt::Enum { .. }
                | Stmt::Trait { .. }
                | Stmt::Impl { .. }
                | Stmt::Alias { .. } => {}
                _ => init.statement(stmt),
            }
        }
        plan.consumed.extend(init.consumed);
        plan.borrowed.extend(init.borrowed);
        plan.borrowed_temporaries.extend(init.borrowed_temporaries);
        plan.moved_captures.extend(init.moved_captures);
        plan.receiver_values.extend(init.receiver_values);
        plan.functions.extend(init.nested);
        plan.functions.push(init.function);
        // Reference summaries are finite sets of formal-input component origins.
        // Repeated monotone union handles recursive call SCCs as well as acyclic
        // calls; no lifetime names or dynamic-target guesses enter the source.
        let mut summaries = plan.functions
            .iter()
            .map(|f| (f.name.clone(), Provenance::default()))
            .collect::<HashMap<_, _>>();
        let mut effects = HashMap::<String, BTreeMap<BindingId, bool>>::new();
        let mut reference_effects = HashMap::<String, ReferenceEffects>::new();
        for f in &plan.functions {
            let mut globals = BTreeMap::new();
            for op in f.blocks.iter().flat_map(|b| &b.operations) {
                let access = match op {
                    Operation::Access { place, access, .. } =>
                        Some((*place, *access != Access::Read)),
                    Operation::Borrow { place, kind, .. } =>
                        Some((*place, *kind == LoanKind::Exclusive)),
                    Operation::Assign { place, .. } => Some((*place, true)),
                    _ => None,
                };
                if let Some((place, write)) = access {
                    let root = f.places[place.0].place.root;
                    if f.bindings[&root].global {
                        globals
                            .entry(root)
                            .and_modify(|w| {
                                *w |= write;
                            })
                            .or_insert(write);
                    }
                }
            }
            effects.insert(f.name.clone(), globals);
        }
        loop {
            let mut changed = false;
            for function in &mut plan.functions {
                for block in &mut function.blocks {
                    for op in &mut block.operations {
                        if
                            let Operation::Call {
                                summary,
                                effects: call_effects,
                                reference_effects: call_stores,
                                target: Some(target),
                                ..
                            } = op
                        {
                            if let Some(p) = summaries.get(target) {
                                *summary = p.clone();
                            }
                            if let Some(e) = effects.get(target) {
                                for (root, write) in e {
                                    call_effects
                                        .entry(*root)
                                        .and_modify(|w| {
                                            *w |= *write;
                                        })
                                        .or_insert(*write);
                                }
                            }
                            if let Some(e) = reference_effects.get(target) {
                                *call_stores = e.clone();
                            }
                        }
                    }
                }
                let result = function.analyze();
                // Local origins are errors at the defining return. Exporting
                // them as a caller's local binding would be an invalid identity.
                let mut p = result.returned;
                for origins in p.components.values_mut() {
                    origins.retain(|o| !matches!(o, Origin::Local(_)));
                }
                p.loans.clear();
                changed |= summaries.entry(function.name.clone()).or_default().join(&p);
                let stores = reference_effects.entry(function.name.clone()).or_default();
                let before = stores.clone();
                for (destination, origins) in result.stores {
                    stores.entry(destination).or_default().extend(origins);
                }
                changed |= *stores != before;
                let out = effects.entry(function.name.clone()).or_default();
                let before = out.clone();
                for op in function.blocks.iter().flat_map(|b| &b.operations) {
                    if let Operation::Call { effects, .. } = op {
                        for (root, write) in effects {
                            out.entry(*root)
                                .and_modify(|w| {
                                    *w |= *write;
                                })
                                .or_insert(*write);
                        }
                    }
                }
                changed |= *out != before;
            }
            if !changed {
                break;
            }
        }
        for function in &plan.functions {
            assert!(
                function.bindings.values().all(|b| properties.resolve(b.ty).is_some()),
                "canonical ownership type identity"
            );
            plan.errors.extend(function.analyze().errors);
        }
        plan.types = properties;
        plan
    }
}

struct Builder<'a> {
    context: &'a Context<'a>,
    contracts: &'a HashMap<String, (Vec<Type>, Type, bool)>,
    properties: TypeProperties,
    function: Function,
    current: Option<usize>,
    scopes: Vec<(ScopeId, HashMap<String, BindingId>)>,
    binding_types: HashMap<BindingId, Type>,
    global_initialization: HashMap<BindingId, bool>,
    source_types: HashMap<BindingId, Type>,
    consumed: HashSet<ExprId>,
    borrowed: HashMap<ExprId, LoanKind>,
    borrowed_temporaries: HashSet<ExprId>,
    receiver_values: HashSet<ExprId>,
    moved_captures: HashSet<(usize, usize)>,
    captured: HashSet<BindingId>,
    readonly_callables: HashSet<BindingId>,
    readonly_expressions: HashSet<ExprId>,
    generic_body: bool,
    parametric: HashSet<BindingId>,
    parametric_values: HashSet<ExprId>,
    bounded_calls: HashSet<ScopeId>,
    nested: Vec<Function>,
}
#[derive(Clone, Copy)]
enum Use {
    Read,
    Consume,
    Borrow(LoanKind),
}
impl<'a> Builder<'a> {
    fn new(
        name: String,
        context: &'a Context<'a>,
        globals: &[(Token, bool, Type, bool)],
        contracts: &'a HashMap<String, (Vec<Type>, Type, bool)>,
        properties: &TypeProperties
    ) -> Self {
        let scope = ScopeId(Token::fresh_id());
        let function = Function {
            name,
            bindings: BTreeMap::new(),
            places: vec![],
            scopes: BTreeMap::from([(scope, None)]),
            blocks: vec![Block { operations: vec![], terminator: Terminator::Return }],
        };
        let mut builder = Self {
            context,
            contracts,
            properties: properties.clone(),
            function,
            current: Some(0),
            scopes: vec![(scope, HashMap::new())],
            binding_types: HashMap::new(),
            global_initialization: HashMap::new(),
            source_types: HashMap::new(),
            consumed: HashSet::new(),
            borrowed: HashMap::new(),
            borrowed_temporaries: HashSet::new(),
            receiver_values: HashSet::new(),
            moved_captures: HashSet::new(),
            captured: HashSet::new(),
            readonly_callables: HashSet::new(),
            readonly_expressions: HashSet::new(),
            generic_body: false,
            parametric: HashSet::new(),
            parametric_values: HashSet::new(),
            bounded_calls: HashSet::new(),
            nested: vec![],
        };
        for (name, mutable, ty, initialized) in globals {
            let root = builder.declare(name, *mutable, ty.clone(), *initialized);
            builder.function.bindings.get_mut(&root).unwrap().global = true;
            builder.global_initialization.insert(root, *initialized);
        }
        builder.begin_scope();
        builder
    }
    fn emit(&mut self, op: Operation) {
        if let Some(block) = self.current {
            self.function.blocks[block].operations.push(op);
        }
    }
    fn block(&mut self) -> usize {
        let id = self.function.blocks.len();
        self.function.blocks.push(Block { operations: vec![], terminator: Terminator::Return });
        id
    }
    fn terminate(&mut self, term: Terminator) {
        if let Some(block) = self.current.take() {
            self.function.blocks[block].terminator = term;
        }
    }
    fn begin_scope(&mut self) -> ScopeId {
        let scope = ScopeId(Token::fresh_id());
        self.function.scopes.insert(
            scope,
            self.scopes.last().map(|s| s.0)
        );
        self.scopes.push((scope, HashMap::new()));
        scope
    }
    fn end_scope(&mut self) {
        let scope = self.scopes.pop().unwrap().0;
        self.emit(Operation::EndScope(scope));
    }
    fn scope(&self) -> ScopeId {
        self.scopes.last().unwrap().0
    }
    fn declare(&mut self, name: &Token, mutable: bool, ty: Type, initialized: bool) -> BindingId {
        let id = BindingId(name.id);
        let type_id = self.properties.intern(&ty);
        let owning = self.properties.get(&ty).needs_drop;
        self.function.bindings.insert(id, BindingInfo {
            id,
            declaration: name.clone(),
            mutable,
            ty: type_id,
            scope: self.scope(),
            owning,
            global: false,
        });
        self.binding_types.insert(id, ty);
        self.scopes.last_mut().unwrap().1.insert(name.lexeme.clone(), id);
        self.emit(Operation::Declare {
            binding: id,
            initialized,
            provenance: Provenance::default(),
        });
        id
    }
    fn parameters(&mut self, params: &[FunctionParam]) {
        for (index, p) in params.iter().enumerate() {
            let ty = p.type_annotation.clone().unwrap_or(Type::Void);
            let id = self.declare(&p.name, p.mutable, ty.clone(), true);
            if self.properties.get(&ty).contains_borrow {
                self.parameter_capabilities(id, &ty, index);
            }
        }
    }
    fn parameter_capabilities(&mut self, binding: BindingId, ty: &Type, index: usize) {
        let mut provenance = Provenance::default();
        self.parameter_origins(ty, index, vec![], &mut provenance, &mut HashSet::new());
        let kind = if self.properties.contains_exclusive(ty) {
            LoanKind::Exclusive
        } else {
            LoanKind::Shared
        };
        let mut loans = vec![];
        for origins in provenance.components.values() {
            let loan = ExprId::fresh();
            provenance.loans.insert(loan);
            loans.push(Loan {
                id: loan,
                origins: origins.clone(),
                kind,
                scopes: BTreeSet::from([self.scope()]),
                parents: BTreeSet::new(),
            });
        }
        self.emit(Operation::Parameter { binding, provenance, loans });
    }
    fn abstract_source(ty: &Type) -> bool {
        match ty {
            Type::TypeVar(_) => true,
            Type::Array(_, t) | Type::Slice(t) | Type::MutReference(t) => Self::abstract_source(t),
            Type::Tuple(ts) | Type::Union(ts) | Type::Generic(_, ts) =>
                ts.iter().any(Self::abstract_source),
            _ => false,
        }
    }
    fn parameter_origins(
        &self,
        ty: &Type,
        index: usize,
        path: Component,
        out: &mut Provenance,
        visited: &mut HashSet<Type>
    ) {
        if !visited.insert(ty.clone()) {
            // Recursive owners can carry references at arbitrarily deep runtime
            // nodes. Preserve the union of the entire formal input rather than
            // dropping origins when the finite nominal graph closes a cycle.
            if self.properties.clone().get(ty).contains_borrow {
                out.components
                    .entry(vec![])
                    .or_default()
                    .insert(Origin::Parameter(index, vec![]));
            }
            return;
        }
        match ty {
            Type::Reference(_) | Type::MutReference(_) | Type::Function(..) => {
                out.components.insert(
                    path.clone(),
                    BTreeSet::from([Origin::Parameter(index, path)])
                );
            }
            Type::Tuple(ts) =>
                for (i, t) in ts.iter().enumerate() {
                    let mut p = path.clone();
                    p.push(Projection::Field(i));
                    self.parameter_origins(t, index, p, out, &mut visited.clone());
                }
            Type::Array(_, t) | Type::Slice(t) => {
                let mut p = path;
                p.push(Projection::Index(None));
                self.parameter_origins(t, index, p, out, visited);
            }
            Type::Custom(n) => if let Some(info) = self.context.types.get(n) {
                if info.erased_payload {
                    out.components.insert(
                        path.clone(),
                        BTreeSet::from([Origin::Parameter(index, path.clone())])
                    );
                }
                for (i, t, _) in info.fields.values() {
                    let mut p = path.clone();
                    p.push(Projection::Field(*i));
                    self.parameter_origins(t, index, p, out, &mut visited.clone());
                }
                for name in &info.case_names {
                    for (i, t) in info.static_methods[name].0.iter().enumerate() {
                        let mut p = path.clone();
                        p.push(Projection::Field(i + 1));
                        self.parameter_origins(t, index, p, out, &mut visited.clone());
                    }
                }
                if let Some(t) = &info.resource_element {
                    let mut p = path.clone();
                    p.push(Projection::Index(None));
                    self.parameter_origins(t, index, p, out, &mut visited.clone());
                }
            }
            _ => {}
        }
    }
    fn resolve(&self, name: &str) -> Option<BindingId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|(_, s)| s.get(name).copied())
    }
    fn span(expr: &Expr) -> Token {
        match expr.untyped() {
            Expr::Variable(n) | Expr::This(n) => n.clone(),
            Expr::Get { name, .. } | Expr::Set { name, .. } | Expr::Assign { name, .. } =>
                name.clone(),
            | Expr::Unary { operator, .. }
            | Expr::Binary { operator, .. }
            | Expr::Logical { operator, .. }
            | Expr::AddressOf { operator, .. }
            | Expr::Dereference { operator, .. }
            | Expr::DereferenceSet { operator, .. }
            | Expr::Cast { operator, .. } => operator.clone(),
            Expr::Call { paren, .. } => paren.clone(),
            | Expr::Array { bracket, .. }
            | Expr::SubscriptGet { bracket, .. }
            | Expr::SubscriptSet { bracket, .. } => bracket.clone(),
            Expr::StructInit { brace, .. } => brace.clone(),
            Expr::Match { keyword, .. } | Expr::Lambda { keyword, .. } => keyword.clone(),
            Expr::Ternary { true_token, .. } => true_token.clone(),
            Expr::TupleLiteral { token, .. } | Expr::Interpolation { token, .. } => token.clone(),
            Expr::Grouping(e) => Self::span(e),
            _ => Token::synthetic(TokenType::Identifier, "value"),
        }
    }
    fn id(expr: &Expr) -> ExprId {
        expr.identity().unwrap_or_else(ExprId::fresh)
    }
    fn base_place(&mut self, expr: &Expr) -> Option<PlaceInfo> {
        match expr.untyped() {
            Expr::Variable(n) | Expr::This(n) => {
                let root = self.resolve(&n.lexeme)?;
                let writable = self.function.bindings[&root].mutable;
                Some(PlaceInfo {
                    id: PlaceId(0),
                    place: Place { root, path: vec![] },
                    writable,
                    through: None,
                    raw: false,
                })
            }
            Expr::Grouping(e) => self.base_place(e),
            Expr::Get { object, name } => {
                let mut p = self.object_place(object)?;
                if object.checked_type().is_some_and(|ty| self.properties.immutable_payload(ty)) {
                    p.writable = false;
                }
                p.place.path.push(Projection::Field(*self.context.fields.get(name)?));
                Some(p)
            }
            Expr::SubscriptGet { indexee, index, .. } => {
                let mut p = self.object_place(indexee)?;
                let lease = self.place_lease(p.clone(), LoanKind::Shared, Self::span(expr));
                self.expression(index, Use::Read);
                self.emit(Operation::EndLoan(lease));
                let constant = match index.untyped() {
                    Expr::Literal(Literal::Integer(n)) => usize::try_from(*n).ok(),
                    _ => None,
                };
                p.place.path.push(Projection::Index(constant));
                Some(p)
            }
            Expr::Dereference { operand, .. } => {
                let mut p = self.base_place(operand)?;
                let value = self.expression(operand, Use::Read);
                let permitted = p.writable || p.place.path.is_empty();
                p.place.path.push(Projection::Dereference);
                p.through = Some(value);
                p.raw |= matches!(operand.checked_type(), Some(Type::Pointer(_)));
                p.writable =
                    matches!(operand.checked_type(), Some(Type::Pointer(_))) ||
                    (permitted && matches!(operand.checked_type(), Some(Type::MutReference(_))));
                Some(p)
            }
            _ => None,
        }
    }
    fn object_place(&mut self, expr: &Expr) -> Option<PlaceInfo> {
        self.object_place_access(expr, true)
    }
    fn object_place_access(&mut self, expr: &Expr, raw_access: bool) -> Option<PlaceInfo> {
        let mut evaluated = None;
        let mut p = if let Some(p) = self.base_place(expr) {
            p
        } else if matches!(expr.checked_type(), Some(Type::Reference(_) | Type::MutReference(_))) {
            let value = self.expression(expr, Use::Read);
            let mut token = Self::span(expr);
            token.id = Token::fresh_id();
            evaluated = Some(value);
            let root = self.declare(&token, false, expr.checked_type().unwrap().clone(), true);
            self.emit(Operation::Bind { binding: root, value, initialization: true, span: token });
            PlaceInfo {
                id: PlaceId(0),
                place: Place { root, path: vec![] },
                writable: false,
                through: None,
                raw: false,
            }
        } else {
            return None;
        };
        if raw_access && matches!(expr.checked_type(), Some(Type::Pointer(_) | Type::String)) {
            p.writable = true;
            p.raw = true;
            p.place.path.push(Projection::Dereference);
            p.through = Some(evaluated.unwrap_or_else(|| self.expression(expr, Use::Read)));
        }
        if matches!(expr.checked_type(), Some(Type::Reference(_) | Type::MutReference(_))) {
            let permitted = p.writable || p.place.path.is_empty();
            let value = evaluated.unwrap_or_else(|| self.expression(expr, Use::Read));
            p.through = Some(value);
            p.place.path.push(Projection::Dereference);
            p.writable = permitted && matches!(expr.checked_type(), Some(Type::MutReference(_)));
        }
        let mut ty = expr.checked_type();
        while let Some(Type::Reference(t) | Type::MutReference(t)) = ty {
            ty = Some(t);
        }
        if raw_access && matches!(ty, Some(Type::Pointer(_) | Type::String)) {
            p.raw = true;
            p.writable = true;
        }
        Some(p)
    }
    fn place_lease(&mut self, p: PlaceInfo, kind: LoanKind, span: Token) -> ExprId {
        if p.raw {
            return ExprId::fresh();
        }
        let place = self.place(p);
        let value = ExprId::fresh();
        self.emit(Operation::Borrow { place, kind, value, scope: self.scope(), span });
        value
    }
    fn place(&mut self, mut p: PlaceInfo) -> PlaceId {
        let id = PlaceId(self.function.places.len());
        p.id = id;
        self.function.places.push(p);
        id
    }
    fn expression(&mut self, expr: &Expr, usage: Use) -> ExprId {
        let id = self.expression_inner(expr, usage);
        if
            !matches!(usage, Use::Borrow(_)) &&
            expr
                .checked_type()
                .is_some_and(
                    |t| !matches!(t, Type::Function(..)) && !self.properties.get(t).contains_borrow
                )
        {
            self.emit(Operation::Value { value: id, sources: vec![] });
        }
        id
    }
    fn condition(&mut self, expr: &Expr) {
        self.begin_scope();
        self.expression(expr, Use::Read);
        self.end_scope();
    }
    fn expression_inner(&mut self, expr: &Expr, usage: Use) -> ExprId {
        let id = Self::id(expr);
        if self.current.is_none() {
            return id;
        }
        // Consumption also transfers freshly produced owners. Restricting this
        // fact to local loads makes consuming methods on returned temporaries
        // incorrectly clean up the already-adopted receiver in the caller.
        if
            matches!(usage, Use::Consume) &&
            expr.checked_type().is_some_and(|t| !self.properties.get(t).is_copy())
        {
            self.consumed.insert(id);
        }
        if
            matches!(usage, Use::Consume) &&
            matches!(
                expr.untyped(),
                Expr::Assign { .. } |
                    Expr::Set { .. } |
                    Expr::SubscriptSet { .. } |
                    Expr::DereferenceSet { .. }
            ) &&
            expr.checked_type().is_some_and(|t| !self.properties.get(t).is_copy())
        {
            self.emit(Operation::Reject {
                span: Self::span(expr),
                message: "assignment does not create another owned value".into(),
            });
        }
        if let Use::Borrow(kind) = usage {
            let operand = if let Expr::AddressOf { operand, .. } = expr.untyped() {
                operand.as_ref()
            } else {
                expr
            };
            let p = if
                !matches!(expr.untyped(), Expr::AddressOf { .. }) &&
                matches!(operand.checked_type(), Some(Type::Reference(_) | Type::MutReference(_)))
            {
                self.object_place_access(operand, false)
            } else {
                self.base_place(operand)
            };
            if let Some(p) = p {
                let place = self.place(p);
                self.emit(Operation::Borrow {
                    place,
                    kind,
                    value: id,
                    scope: self.scope(),
                    span: Self::span(expr),
                });
                self.borrowed.insert(id, kind);
                return id;
            }
            if
                matches!(
                    operand.checked_type(),
                    Some(Type::Reference(_) | Type::MutReference(_))
                ) &&
                !matches!(expr.untyped(), Expr::AddressOf { .. })
            {
                if
                    kind == LoanKind::Exclusive &&
                    matches!(operand.checked_type(), Some(Type::Reference(_)))
                {
                    self.emit(Operation::Reject {
                        span: Self::span(expr),
                        message: "cannot modify through a shared reference".into(),
                    });
                }
                return self.expression(expr, Use::Read);
            }
            // A bounded temporary borrow is made explicit later by construction
            // elaboration; until then no owner lifetime is fabricated here.
            if
                matches!(expr.untyped(), Expr::AddressOf { .. }) &&
                (!self.bounded_calls.contains(&self.scope()) ||
                    matches!(
                        operand.untyped(),
                        Expr::Get { .. } | Expr::SubscriptGet { .. } | Expr::Dereference { .. }
                    ) ||
                    kind == LoanKind::Exclusive)
            {
                self.emit(Operation::Reject {
                    span: Self::span(expr),
                    message: "cannot prove this reference remains valid for temporary storage".into(),
                });
                return id;
            }
            self.expression(operand, Use::Consume);
            self.borrowed_temporaries.insert(id);
            let token = Self::span(expr);
            let root = self.declare(
                &token,
                false,
                operand.checked_type().cloned().unwrap_or(Type::Void),
                true
            );
            let p = PlaceInfo {
                id: PlaceId(0),
                place: Place { root, path: vec![] },
                writable: false,
                through: None,
                raw: false,
            };
            let place = self.place(p);
            self.emit(Operation::Borrow {
                place,
                kind,
                value: id,
                scope: self.scope(),
                span: token,
            });
            self.borrowed.insert(id, kind);
            return id;
        }
        if let Some(p) = self.base_place(expr) {
            if
                p.raw &&
                expr.checked_type().is_some_and(|ty| self.properties.get(ty).contains_borrow)
            {
                self.emit(Operation::Reject {
                    span: Self::span(expr),
                    message: "cannot prove a safe reference loaded from raw storage remains valid".into(),
                });
            }
            let abstract_value =
                self.parametric.contains(&p.place.root) ||
                (self.generic_body && matches!(self.source_type(expr), Some(Type::TypeVar(_)))) ||
                (self.generic_body && self.parametric_projection(expr));
            if abstract_value {
                self.parametric_values.insert(id);
            }
            let copy =
                !abstract_value &&
                expr.checked_type().is_some_and(|t| self.properties.get(t).is_copy());
            let access = if matches!(usage, Use::Consume) && !copy {
                self.consumed.insert(id);
                Access::Consume
            } else {
                Access::Read
            };
            if access == Access::Consume && self.captured.contains(&p.place.root) {
                self.emit(Operation::Reject {
                    span: Self::span(expr),
                    message: "cannot move an owned capture out of a repeatable closure".into(),
                });
            }
            let place = self.place(p);
            self.emit(Operation::Access { place, access, value: id, span: Self::span(expr) });
            return id;
        }
        match expr.untyped() {
            Expr::Memory { kind, element, arguments, token } => {
                use crate::ast::MemoryOp;
                if
                    matches!(kind, MemoryOp::SharedClone | MemoryOp::ReplaceValue) &&
                    self.properties.get(element).contains_borrow
                {
                    self.emit(Operation::Reject {
                        span: token.clone(),
                        message: "cannot prove reference storage remains valid through this private storage operation".into(),
                    });
                }
                let mut values = vec![];
                for (i, argument) in arguments.iter().enumerate() {
                    let usage = if
                        (*kind == MemoryOp::RawInitialize && i == 3) ||
                        (*kind == MemoryOp::ReplaceValue && i == 1)
                    {
                        Use::Consume
                    } else {
                        Use::Read
                    };
                    values.push(self.expression(argument, usage));
                }
                if *kind == MemoryOp::RawInitialize && self.properties.get(element).contains_borrow {
                    let element = ExprId::fresh();
                    self.emit(Operation::Value {
                        value: element,
                        sources: vec![(values[3], vec![Projection::Index(None)])],
                    });
                    self.emit(Operation::StoreReference {
                        destination: values[0],
                        value: element,
                        span: token.clone(),
                    });
                }
                let sources = if matches!(kind, MemoryOp::RawBorrow | MemoryOp::RawTake) {
                    values
                        .first()
                        .map(|v| vec![(*v, vec![])])
                        .unwrap_or_default()
                } else {
                    vec![]
                };
                if self.generic_body && matches!(kind, MemoryOp::RawTake | MemoryOp::ReplaceValue) {
                    self.parametric_values.insert(id);
                }
                if *kind == MemoryOp::RawTake {
                    let contents = ExprId::fresh();
                    self.emit(Operation::Contents { value: contents, reference: values[0] });
                    self.emit(Operation::Project {
                        value: id,
                        source: contents,
                        projection: Projection::Index(None),
                    });
                } else {
                    self.emit(Operation::Value { value: id, sources });
                }
            }
            Expr::Interpolation { parts, .. } => {
                for part in parts {
                    self.expression(part, Use::Read);
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::Grouping(e) => {
                let source = self.expression(e, usage);
                if self.parametric_values.contains(&source) {
                    self.parametric_values.insert(id);
                }
                self.emit(Operation::Value { value: id, sources: vec![(source, vec![])] });
            }
            Expr::AddressOf { .. } => {
                if matches!(expr.checked_type(), Some(Type::Pointer(_))) {
                    if let Expr::AddressOf { operand, .. } = expr.untyped() {
                        self.expression(operand, Use::Read);
                    }
                    self.emit(Operation::Value { value: id, sources: vec![] });
                    return id;
                }
                let kind = if matches!(expr.checked_type(), Some(Type::MutReference(_))) {
                    LoanKind::Exclusive
                } else {
                    LoanKind::Shared
                };
                return self.expression(expr, Use::Borrow(kind));
            }
            Expr::Assign { name, value } => {
                let value = self.expression(value, Use::Consume);
                if let Some(binding) = self.resolve(&name.lexeme) {
                    let writable = self.function.bindings[&binding].mutable;
                    let place = self.place(PlaceInfo {
                        id: PlaceId(0),
                        place: Place { root: binding, path: vec![] },
                        writable,
                        through: None,
                        raw: false,
                    });
                    self.emit(Operation::Assign { place, value, span: name.clone() });
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::Set { object, name, value } => {
                if let Some(mut p) = self.object_place(object) {
                    let lease = self.place_lease(p.clone(), LoanKind::Shared, name.clone());
                    if
                        object
                            .checked_type()
                            .is_some_and(|ty| self.properties.immutable_payload(ty))
                    {
                        p.writable = false;
                    }
                    p.place.path.push(
                        Projection::Field(*self.context.fields.get(name).unwrap_or(&0))
                    );
                    let place = self.place(p);
                    let value = self.expression(value, Use::Consume);
                    self.emit(Operation::EndLoan(lease));
                    self.emit(Operation::Assign { place, value, span: name.clone() });
                } else {
                    self.expression(object, Use::Read);
                    self.expression(value, Use::Consume);
                    self.emit(Operation::Reject {
                        span: name.clone(),
                        message: "mutation requires a mutable owner place or an exclusive reference".into(),
                    });
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::SubscriptSet { indexee, index, value, bracket } => {
                if let Some(mut p) = self.object_place(indexee) {
                    let lease = self.place_lease(p.clone(), LoanKind::Shared, bracket.clone());
                    self.expression(index, Use::Read);
                    p.place.path.push(Projection::Index(None));
                    let place = self.place(p);
                    let value = self.expression(value, Use::Consume);
                    self.emit(Operation::EndLoan(lease));
                    self.emit(Operation::Assign { place, value, span: bracket.clone() });
                } else {
                    self.expression(indexee, Use::Read);
                    self.expression(index, Use::Read);
                    self.expression(value, Use::Consume);
                    if !matches!(indexee.checked_type(), Some(Type::Pointer(_) | Type::String)) {
                        self.emit(Operation::Reject {
                            span: bracket.clone(),
                            message: "mutation requires a mutable owner place or an exclusive reference".into(),
                        });
                    }
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::DereferenceSet { ptr, value, operator } => {
                if let Some(mut p) = self.base_place(ptr) {
                    let permitted = p.writable || p.place.path.is_empty();
                    let through = self.expression(ptr, Use::Read);
                    p.through = Some(through);
                    p.place.path.push(Projection::Dereference);
                    p.writable =
                        matches!(ptr.checked_type(), Some(Type::Pointer(_))) ||
                        (permitted && matches!(ptr.checked_type(), Some(Type::MutReference(_))));
                    let place = self.place(p);
                    let value = self.expression(value, Use::Consume);
                    self.emit(Operation::Assign { place, value, span: operator.clone() });
                } else if
                    matches!(ptr.checked_type(), Some(Type::Reference(_) | Type::MutReference(_)))
                {
                    if let Some(p) = self.object_place_access(ptr, false) {
                        let place = self.place(p);
                        let value = self.expression(value, Use::Consume);
                        self.emit(Operation::Assign { place, value, span: operator.clone() });
                    }
                } else {
                    self.expression(ptr, Use::Read);
                    self.expression(value, Use::Consume);
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::Call { callee, arguments, paren } =>
                self.call(expr, id, callee, arguments, paren),
            Expr::Ternary { condition, then_branch, else_branch, .. } => {
                self.condition(condition);
                let a = self.block();
                let b = self.block();
                let end = self.block();
                self.terminate(Terminator::Branch(a, b));
                for (block, e) in [
                    (a, then_branch),
                    (b, else_branch),
                ] {
                    self.current = Some(block);
                    let value = self.expression(e, usage);
                    if self.parametric_values.contains(&value) {
                        self.parametric_values.insert(id);
                    }
                    self.emit(Operation::Value { value: id, sources: vec![(value, vec![])] });
                    self.terminate(Terminator::Goto(end));
                }
                self.current = Some(end);
            }
            Expr::Logical { left, right, .. } => {
                self.condition(left);
                let rhs = self.block();
                let end = self.block();
                self.terminate(Terminator::Branch(rhs, end));
                self.current = Some(rhs);
                self.condition(right);
                self.terminate(Terminator::Goto(end));
                self.current = Some(end);
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::Match { value, cases, keyword } => {
                let consuming =
                    !matches!(
                        value.checked_type(),
                        Some(Type::Reference(_) | Type::MutReference(_))
                    ) && value.checked_type().is_some_and(|t| !self.properties.get(t).is_copy());
                if consuming && cases.iter().any(|c| c.guard.is_some()) {
                    self.emit(Operation::Reject {
                        span: keyword.clone(),
                        message: "guarded consuming aggregate matches require a borrowed match".into(),
                    });
                }
                let source_type = self.source_type(value);
                let actual_type = value.checked_type().cloned().unwrap_or(Type::Void);
                let abstract_value =
                    self.generic_body && source_type.as_ref().is_some_and(Self::abstract_source);
                let value = self.expression(value, if consuming {
                    Use::Consume
                } else {
                    Use::Read
                });
                let end = self.block();
                for case in cases {
                    let yes = self.block();
                    let next = self.block();
                    self.terminate(Terminator::Branch(yes, next));
                    self.current = Some(yes);
                    self.begin_scope();
                    self.pattern(
                        &case.pattern,
                        value,
                        abstract_value,
                        &actual_type,
                        source_type.clone()
                    );
                    if let Some(guard) = &case.guard {
                        self.condition(guard);
                        let body = self.block();
                        let fail = self.block();
                        self.terminate(Terminator::Branch(body, fail));
                        self.current = Some(fail);
                        self.emit(Operation::EndScope(self.scope()));
                        self.terminate(Terminator::Goto(next));
                        self.current = Some(body);
                    }
                    let body = if let [Stmt::Block(b)] = case.body.as_slice() {
                        b.as_slice()
                    } else {
                        case.body.as_slice()
                    };
                    for (index, stmt) in body.iter().enumerate() {
                        if index + 1 == body.len() {
                            if let Stmt::Expression(e) = stmt {
                                let source = self.expression(e, usage);
                                if self.parametric_values.contains(&source) {
                                    self.parametric_values.insert(id);
                                }
                                self.emit(Operation::Value {
                                    value: id,
                                    sources: vec![(source, vec![])],
                                });
                                continue;
                            }
                        }
                        self.statement(stmt);
                    }
                    self.end_scope();
                    self.terminate(Terminator::Goto(end));
                    self.current = Some(next);
                }
                self.terminate(Terminator::Abort);
                self.current = Some(end);
                let _ = keyword;
            }
            Expr::StructInit { properties, class_name, .. } => {
                let real = match self.context.calls.get(class_name) {
                    Some(CallType::Static(n) | CallType::Instance(n)) => n,
                    _ => &class_name.lexeme,
                };
                let mut properties = properties.iter().collect::<Vec<_>>();
                properties.sort_by_key(|(name, _)|
                    self.context.types
                        .get(real)
                        .and_then(|i| i.fields.get(&name.lexeme))
                        .map(|(i, _, _)| *i)
                        .unwrap_or(0)
                );
                let mut sources = vec![];
                for (i, (_, e)) in properties.into_iter().enumerate() {
                    let value = self.expression(e, Use::Consume);
                    sources.push((value, vec![Projection::Field(i)]));
                }
                if sources.iter().any(|(value, _)| self.parametric_values.contains(value)) {
                    self.parametric_values.insert(id);
                }
                self.emit(Operation::Value { value: id, sources });
            }
            Expr::Array { elements, .. } | Expr::TupleLiteral { elements, .. } => {
                let array = matches!(expr.untyped(), Expr::Array { .. });
                let mut sources = vec![];
                for (i, e) in elements.iter().enumerate() {
                    let value = self.expression(e, Use::Consume);
                    sources.push((
                        value,
                        vec![if array { Projection::Index(Some(i)) } else { Projection::Field(i) }],
                    ));
                }
                if sources.iter().any(|(value, _)| self.parametric_values.contains(value)) {
                    self.parametric_values.insert(id);
                }
                self.emit(Operation::Value { value: id, sources });
            }
            Expr::Unary { operator, right } => {
                if matches!(operator.token_type, TokenType::PlusPlus | TokenType::MinusMinus) {
                    if let Some(p) = self.base_place(right) {
                        let place = self.place(p);
                        self.emit(Operation::Access {
                            place,
                            access: Access::Write,
                            value: id,
                            span: operator.clone(),
                        });
                    } else {
                        self.expression(right, Use::Read);
                        self.emit(Operation::Reject {
                            span: operator.clone(),
                            message: "mutation requires a mutable owner place or an exclusive reference".into(),
                        });
                    }
                } else {
                    let value = self.expression(right, Use::Consume);
                    if self.parametric_values.contains(&value) {
                        self.parametric_values.insert(id);
                    }
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::Binary { left, right, operator } => {
                let observing = matches!(
                    operator.token_type,
                    TokenType::EqualEqual |
                        TokenType::BangEqual |
                        TokenType::Less |
                        TokenType::LessEqual |
                        TokenType::Greater |
                        TokenType::GreaterEqual
                );
                let usage = if observing { Use::Read } else { Use::Consume };
                let a = self.expression(left, usage);
                let b = self.expression(right, usage);
                if
                    !observing &&
                    (self.parametric_values.contains(&a) || self.parametric_values.contains(&b))
                {
                    self.parametric_values.insert(id);
                }
                self.emit(Operation::Value { value: id, sources: vec![] });
            }
            Expr::Cast { value, target_type, .. } => {
                let consuming =
                    matches!((value.checked_type(),target_type),(Some(Type::Custom(a)),Type::Custom(b)) if a!=b);
                let value = self.expression(value, if consuming { Use::Consume } else { usage });
                if self.parametric_values.contains(&value) {
                    self.parametric_values.insert(id);
                }
                self.emit(Operation::Value { value: id, sources: vec![(value, vec![])] });
            }
            Expr::Spread { right: value, operator } => {
                let mut ty = value.checked_type();
                while let Some(Type::Reference(t) | Type::MutReference(t)) = ty {
                    ty = Some(t);
                }
                let source = self.source_type(value);
                let mut source = source.as_ref();
                while let Some(Type::Reference(t) | Type::MutReference(t)) = source {
                    source = Some(t);
                }
                let abstract_elements =
                    self.generic_body &&
                    (match source {
                        Some(Type::Array(_, t) | Type::Slice(t)) =>
                            matches!(t.as_ref(), Type::TypeVar(_)),
                        Some(Type::Tuple(ts)) => ts.iter().any(|t| matches!(t, Type::TypeVar(_))),
                        _ => false,
                    });
                let copy =
                    !abstract_elements &&
                    (match ty {
                        Some(Type::Array(_, t)) => self.properties.get(t).is_copy(),
                        Some(Type::Tuple(ts)) =>
                            ts.iter().all(|t| self.properties.get(t).is_copy()),
                        _ => false,
                    });
                if !copy {
                    self.emit(Operation::Reject {
                        span: operator.clone(),
                        message: "array spread requires Copy elements in the initial ownership model".into(),
                    });
                }
                let value = self.expression(value, usage);
                self.emit(Operation::Value { value: id, sources: vec![(value, vec![])] });
            }
            Expr::Lambda { keyword, params, body, ownership_signature, .. } => {
                let mut sources = vec![];
                for (i, capture) in self.context.captures
                    .get(&keyword.id)
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .enumerate() {
                    if let Some(root) = self.resolve(&capture.declaration.lexeme) {
                        let value = ExprId::fresh();
                        let copy =
                            !self.parametric.contains(&root) &&
                            self.properties.get(&capture.ty).is_copy();
                        if !copy {
                            self.moved_captures.insert((keyword.id, capture.declaration.id));
                        }
                        let place = self.place(PlaceInfo {
                            id: PlaceId(0),
                            place: Place { root, path: vec![] },
                            writable: self.function.bindings[&root].mutable,
                            through: None,
                            raw: false,
                        });
                        self.emit(Operation::Access {
                            place,
                            access: if copy {
                                Access::Read
                            } else {
                                Access::Consume
                            },
                            value,
                            span: keyword.clone(),
                        });
                        sources.push((value, vec![Projection::Field(i)]));
                    }
                }
                let globals = self.function.bindings
                    .values()
                    .filter(|b| b.global)
                    .map(|b| (
                        b.declaration.clone(),
                        b.mutable,
                        self.binding_types[&b.id].clone(),
                        self.global_initialization[&b.id],
                    ))
                    .collect::<Vec<_>>();
                let mut nested = Builder::new(
                    format!("lambda.{}", keyword.id),
                    self.context,
                    &globals,
                    self.contracts,
                    &self.properties
                );
                nested.generic_body = self.generic_body || ownership_signature.is_some();
                for (index, capture) in self.context.captures
                    .get(&keyword.id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate() {
                    let source = self.resolve(&capture.declaration.lexeme);
                    let mutable = source.is_some_and(|root| self.function.bindings[&root].mutable);
                    let binding = nested.declare(
                        &capture.declaration,
                        mutable,
                        capture.ty.clone(),
                        true
                    );
                    nested.captured.insert(binding);
                    if source.is_some_and(|root| self.readonly_callables.contains(&root)) {
                        nested.readonly_callables.insert(binding);
                    }
                    if
                        self
                            .resolve(&capture.declaration.lexeme)
                            .is_some_and(|root| self.parametric.contains(&root))
                    {
                        nested.parametric.insert(binding);
                    }
                    if
                        let Some(ty) = self
                            .resolve(&capture.declaration.lexeme)
                            .and_then(|root| self.source_types.get(&root))
                            .cloned()
                    {
                        nested.source_types.insert(binding, ty);
                    }
                    if nested.properties.get(&capture.ty).contains_borrow {
                        nested.parameter_capabilities(binding, &capture.ty, params.len() + index);
                    }
                }
                nested.parameters(params);
                if let Some((types, _)) = ownership_signature {
                    for (p, source) in params.iter().zip(types) {
                        let root = BindingId(p.name.id);
                        nested.source_types.insert(root, source.clone());
                        if matches!(source, Type::TypeVar(_)) {
                            nested.parametric.insert(root);
                            if matches!(p.type_annotation, Some(Type::MutReference(_))) {
                                nested.emit(Operation::Reject {
                                    span: p.name.clone(),
                                    message: "generic callable parameters containing exclusive capabilities require an explicit reference contract".into(),
                                });
                            }
                        }
                    }
                }
                nested.statements(body);
                let mutates_environment = nested.function.blocks
                    .iter()
                    .flat_map(|b| &b.operations)
                    .any(|op| {
                        let place = match op {
                            | Operation::Assign { place, .. }
                            | Operation::Access { place, access: Access::Write, .. }
                            | Operation::Borrow { place, kind: LoanKind::Exclusive, .. } =>
                                Some(*place),
                            _ => None,
                        };
                        place.is_some_and(|place| {
                            let p = &nested.function.places[place.0];
                            let root = p.place.root;
                            nested.captured.contains(&root) &&
                                !(
                                    matches!(nested.binding_types[&root], Type::MutReference(_)) &&
                                    !p.place.path.is_empty()
                                ) &&
                                !nested.readonly_callables.contains(&root)
                        })
                    });
                if !mutates_environment {
                    self.readonly_expressions.insert(id);
                }
                self.consumed.extend(nested.consumed);
                self.borrowed.extend(nested.borrowed);
                self.borrowed_temporaries.extend(nested.borrowed_temporaries);
                self.moved_captures.extend(nested.moved_captures);
                self.receiver_values.extend(nested.receiver_values);
                self.nested.extend(nested.nested);
                self.nested.push(nested.function);
                self.emit(Operation::Value { value: id, sources });
            }
            Expr::SubscriptGet { indexee, index, bracket } => {
                let source = self.expression(indexee, Use::Read);
                self.expression(index, Use::Read);
                if
                    matches!(indexee.checked_type(), Some(Type::Pointer(_) | Type::String)) &&
                    expr.checked_type().is_some_and(|t| self.properties.get(t).contains_borrow)
                {
                    self.emit(Operation::Reject {
                        span: bracket.clone(),
                        message: "cannot prove a safe reference loaded from raw storage remains valid".into(),
                    });
                }
                if
                    matches!(usage, Use::Consume) &&
                    expr.checked_type().is_some_and(|t| !self.properties.get(t).is_copy())
                {
                    self.emit(Operation::Reject {
                        span: bracket.clone(),
                        message: "cannot move an element out of this value here".into(),
                    });
                }
                self.emit(Operation::Project {
                    value: id,
                    source,
                    projection: Projection::Index(None),
                });
            }
            Expr::Dereference { operand, operator } => {
                let source = self.expression(operand, Use::Read);
                if
                    matches!(operand.checked_type(), Some(Type::Pointer(_))) &&
                    expr.checked_type().is_some_and(|t| self.properties.get(t).contains_borrow)
                {
                    self.emit(Operation::Reject {
                        span: operator.clone(),
                        message: "cannot prove a safe reference loaded from raw storage remains valid".into(),
                    });
                }
                if
                    matches!(usage, Use::Consume) &&
                    expr.checked_type().is_some_and(|t| !self.properties.get(t).is_copy())
                {
                    self.emit(Operation::Reject {
                        span: operator.clone(),
                        message: "cannot move an owned value out through this reference".into(),
                    });
                }
                self.emit(Operation::Value { value: id, sources: vec![(source, vec![])] });
            }
            Expr::Get { object, name } => {
                if let Some(CallType::Instance(owner)) = self.context.calls.get(name) {
                    let receiver = self.context.types
                        .get(owner)
                        .and_then(|i| i.methods.get(&name.lexeme))
                        .and_then(|(ps, _, _)| ps.first())
                        .cloned();
                    if !matches!(receiver, Some(Type::Reference(_) | Type::MutReference(_))) {
                        self.emit(Operation::Reject {
                            span: name.clone(),
                            message: "cannot bind a consuming method to a repeatable callable".into(),
                        });
                    }
                    let usage = if matches!(receiver, Some(Type::MutReference(_))) {
                        Use::Borrow(LoanKind::Exclusive)
                    } else {
                        Use::Borrow(LoanKind::Shared)
                    };
                    let source = self.expression(object, usage);
                    self.readonly_expressions.insert(id);
                    self.emit(Operation::Value { value: id, sources: vec![(source, vec![])] });
                } else {
                    let source = self.expression(object, Use::Read);
                    if
                        matches!(object.checked_type(), Some(Type::Pointer(_))) &&
                        expr.checked_type().is_some_and(|t| self.properties.get(t).contains_borrow)
                    {
                        self.emit(Operation::Reject {
                            span: name.clone(),
                            message: "cannot prove a safe reference loaded from raw storage remains valid".into(),
                        });
                    }
                    self.emit(Operation::Project {
                        value: id,
                        source,
                        projection: Projection::Field(*self.context.fields.get(name).unwrap_or(&0)),
                    });
                    if
                        matches!(usage, Use::Consume) &&
                        expr.checked_type().is_some_and(|t| !self.properties.get(t).is_copy())
                    {
                        self.emit(Operation::Reject {
                            span: name.clone(),
                            message: "cannot move a field out of this value here".into(),
                        });
                    }
                }
            }
            Expr::Lazy { expr: inner, .. } =>
                self.emit(Operation::Reject {
                    span: inner
                        .as_ref()
                        .map(|e| Self::span(e))
                        .unwrap_or_else(|| Self::span(expr)),
                    message: "lazy expressions have no supported synchronous ownership lowering".into(),
                }),
            | Expr::Literal(_)
            | Expr::Variable(_)
            | Expr::This(_)
            | Expr::Typeof(_)
            | Expr::StorageSize(_) => self.emit(Operation::Value { value: id, sources: vec![] }),
            | Expr::WildcardPattern(_)
            | Expr::UnionPattern { .. }
            | Expr::ListPattern { .. }
            | Expr::ObjectPattern { .. } =>
                self.emit(Operation::Reject {
                    span: Self::span(expr),
                    message: "pattern cannot be evaluated as a value".into(),
                }),
            Expr::Typed { .. } => unreachable!("untyped ownership expression"),
        }
        id
    }
    fn call(&mut self, expr: &Expr, id: ExprId, callee: &Expr, arguments: &[Expr], paren: &Token) {
        let mut callee = callee;
        while let Expr::Grouping(inner) = callee.untyped() {
            callee = inner;
        }
        let types = if let Some(Type::Function(ps, _, _)) = callee.checked_type() {
            ps.clone()
        } else {
            vec![]
        };
        let source_parameters = (
            match callee.untyped() {
                Expr::Variable(name) if self.resolve(&name.lexeme).is_none() => {
                    let symbol = match self.context.calls.get(name) {
                        Some(CallType::Static(symbol)) => symbol.as_str(),
                        _ => name.lexeme.as_str(),
                    };
                    self.contracts.get(symbol).map(|(ps, _, _)| ps.clone())
                }
                Expr::Get { name, .. } => {
                    let target = self.context.specialized
                        .get(name)
                        .cloned()
                        .or_else(||
                            self.context.calls.get(name).map(|call| {
                                match call {
                                    CallType::Instance(owner) =>
                                        (crate::symbols::member(owner, &name.lexeme), true),
                                    CallType::Static(owner) =>
                                        (crate::symbols::member(owner, &name.lexeme), false),
                                }
                            })
                        );
                    let constructor = if
                        let Some(CallType::Static(owner)) = self.context.calls.get(name)
                    {
                        self.context.types
                            .get(owner)
                            .filter(|i| i.case_names.contains(&name.lexeme))
                            .map(|info| {
                                info.static_methods[&name.lexeme].0
                                    .iter()
                                    .enumerate()
                                    .map(|(i, t)| {
                                        if
                                            info.parametric_value_fields.contains(
                                                &(name.lexeme.clone(), i)
                                            )
                                        {
                                            Type::TypeVar(0)
                                        } else {
                                            t.clone()
                                        }
                                    })
                                    .collect()
                            })
                    } else {
                        None
                    };
                    constructor.or_else(||
                        target.and_then(|(symbol, instance)|
                            self.contracts
                                .get(&symbol)
                                .map(|(ps, _, _)|
                                    ps.iter().skip(usize::from(instance)).cloned().collect()
                                )
                        )
                    )
                }
                _ => None,
            }
        ).or_else(|| {
            match self.source_type(callee) {
                Some(Type::Function(ps, _, _)) => Some(ps),
                _ => None,
            }
        });
        let callable_source = self.source_type(callee);
        let call_scope = self.begin_scope();
        let mut values = vec![];
        self.bounded_calls.insert(call_scope);
        for (index, arg) in arguments.iter().enumerate() {
            let usage = if
                matches!(
                    source_parameters.as_ref().and_then(|ps| ps.get(index)),
                    Some(Type::TypeVar(_))
                )
            {
                Use::Consume
            } else {
                match types.get(index) {
                    Some(Type::Reference(_)) => Use::Borrow(LoanKind::Shared),
                    Some(Type::MutReference(_)) => Use::Borrow(LoanKind::Exclusive),
                    _ => Use::Consume,
                }
            };
            values.push(self.expression(arg, usage));
        }
        let mut target = paren.clone();
        let mut direct = None;
        match callee.untyped() {
            Expr::Variable(name) => {
                target.lexeme = match self.context.calls.get(name) {
                    Some(CallType::Static(n)) => n.clone(),
                    _ => name.lexeme.clone(),
                };
                if self.resolve(&name.lexeme).is_some() {
                    values.push(self.callable(callee));
                } else {
                    direct = Some(target.lexeme.clone());
                }
            }
            Expr::Get { object, name } => {
                if let Some((symbol, instance)) = self.context.specialized.get(name) {
                    if *instance {
                        let receiver = self.contracts.get(symbol).and_then(|(ps, _, _)| ps.first());
                        let receiver = receiver.cloned();
                        let usage = self.receiver_usage(object, receiver.as_ref());
                        values.insert(0, self.expression(object, usage));
                    } else if object.checked_type().is_some() {
                        self.expression(object, Use::Read);
                    }
                    target.lexeme = symbol.clone();
                    direct = Some(symbol.clone());
                } else if let Some(CallType::Instance(owner)) = self.context.calls.get(name) {
                    let receiver = self.context.specialized
                        .get(name)
                        .and_then(|(n, _)| self.contracts.get(n))
                        .and_then(|(ps, _, _)| ps.first())
                        .cloned()
                        .or_else(||
                            self.context.types
                                .get(owner)
                                .and_then(|i| i.methods.get(&name.lexeme))
                                .and_then(|(ps, _, _)| ps.first())
                                .cloned()
                        );
                    let usage = self.receiver_usage(object, receiver.as_ref());
                    values.insert(0, self.expression(object, usage));
                    target.lexeme = crate::symbols::member(owner, &name.lexeme);
                    direct = if self.context.traits.contains(owner) {
                        None
                    } else {
                        Some(target.lexeme.clone())
                    };
                } else if let Some(CallType::Static(owner)) = self.context.calls.get(name) {
                    target.lexeme = crate::symbols::member(owner, &name.lexeme);
                    direct = Some(target.lexeme.clone());
                } else {
                    values.push(self.callable(callee));
                }
            }
            _ => {
                values.push(self.callable(callee));
            }
        }
        if let Expr::Get { name, .. } = callee.untyped() {
            if let Some((symbol, _)) = self.context.specialized.get(name) {
                target.lexeme = symbol.clone();
                direct = Some(symbol.clone());
            }
        }
        if
            self.generic_body &&
            direct
                .as_ref()
                .and_then(|n| self.contracts.get(n))
                .is_some_and(|(_, ret, generic)| *generic && matches!(ret, Type::TypeVar(_)))
        {
            self.parametric_values.insert(id);
        }
        if self.generic_body && matches!(self.source_type(expr), Some(Type::TypeVar(_))) {
            self.parametric_values.insert(id);
        }
        let uncertain = !matches!(callable_source, Some(Type::Function(..)));
        let return_copy = expr
            .checked_type()
            .is_some_and(|t| *t != Type::Void && self.properties.get(t).is_copy());
        if
            direct.is_none() &&
            uncertain &&
            (types.iter().any(|t| matches!(t, Type::Reference(_) | Type::MutReference(_))) ||
                (self.generic_body && return_copy))
        {
            self.emit(Operation::Reject {
                span: paren.clone(),
                message: "cannot prove this callable's ownership contract; give it an explicit function type".into(),
            });
        }
        if let Some(defaults) = self.context.defaults.get(paren) {
            let mut proxies = vec![];
            let supplied = values.len();
            for (index, (name, ty)) in defaults.parameters.iter().enumerate() {
                let value = if index < supplied {
                    values[index]
                } else {
                    let usage = match ty {
                        Type::Reference(_) => Use::Borrow(LoanKind::Shared),
                        Type::MutReference(_) => Use::Borrow(LoanKind::Exclusive),
                        _ => Use::Consume,
                    };
                    let value = self.expression(&defaults.defaults[index - supplied], usage);
                    values.push(value);
                    value
                };
                let mut proxy = name.clone();
                proxy.id = Token::fresh_id();
                let binding = self.declare(&proxy, false, ty.clone(), false);
                if defaults.parametric_parameters.contains(&index) {
                    self.parametric.insert(binding);
                }
                self.emit(Operation::Bind {
                    binding,
                    value,
                    initialization: true,
                    span: name.clone(),
                });
                proxies.push((binding, name.clone(), ty.clone()));
            }
            // Defaults may inspect earlier arguments, but cannot consume one
            // that is still required by the eventual source call.
            for (root, span, ty) in proxies {
                let place = self.place(PlaceInfo {
                    id: PlaceId(0),
                    place: Place { root, path: vec![] },
                    writable: false,
                    through: None,
                    raw: false,
                });
                let access = if
                    !self.parametric.contains(&root) &&
                    self.properties.get(&ty).is_copy()
                {
                    Access::Read
                } else {
                    Access::Consume
                };
                self.emit(Operation::Access { place, access, value: ExprId::fresh(), span });
            }
        }
        let mut summary = Provenance::default();
        if expr.checked_type().is_some_and(|t| self.properties.get(t).contains_borrow) {
            // A conservative callable contract when a narrower direct summary
            // has not yet been inferred. All compatible borrowed inputs matter.
            let mut origins: BTreeSet<_> = values
                .iter()
                .enumerate()
                .map(|(i, _)| Origin::Parameter(i, vec![]))
                .collect();
            if direct.is_none() {
                origins.insert(Origin::Static);
            }
            summary.components.insert(vec![], origins);
        }
        let scope = self.function.scopes[&call_scope].expect("call has a lexical parent");
        let effects = if direct.is_none() {
            self.function.bindings
                .values()
                .filter(|b| b.global)
                .map(|b| (b.id, b.mutable))
                .collect()
        } else {
            BTreeMap::new()
        };
        let result_loan = if
            expr.checked_type().is_some_and(|t| self.properties.get(t).contains_borrow)
        {
            Some(
                if matches!(expr.checked_type(), Some(Type::MutReference(_))) {
                    LoanKind::Exclusive
                } else {
                    LoanKind::Shared
                }
            )
        } else {
            None
        };
        let mut reference_effects = ReferenceEffects::new();
        if direct.is_none() {
            let mut modes = types.clone();
            if let Expr::Get { name, .. } = callee.untyped() {
                if let Some(CallType::Instance(owner)) = self.context.calls.get(name) {
                    if
                        let Some(receiver) = self.context.types
                            .get(owner)
                            .and_then(|i| i.methods.get(&name.lexeme))
                            .and_then(|(ps, _, _)| ps.first())
                    {
                        modes.insert(0, receiver.clone());
                    }
                }
            }
            for (index, ty) in modes.iter().enumerate() {
                if let Type::MutReference(inner) = ty {
                    if self.properties.get(inner).contains_borrow {
                        let mut sources: BTreeSet<_> = (0..values.len())
                            .filter(|i| *i != index)
                            .map(|i| Origin::Parameter(i, vec![]))
                            .collect();
                        sources.insert(Origin::Static);
                        reference_effects.insert(Origin::Parameter(index, vec![]), sources);
                    }
                }
            }
        }
        let receiver = if let Expr::Get { name, .. } = callee.untyped() {
            self.context.calls.get(name).and_then(|call| (
                if let CallType::Instance(owner) = call {
                    self.context.types
                        .get(owner)
                        .and_then(|info| info.methods.get(&name.lexeme))
                        .and_then(|(ps, _, _)| ps.first())
                } else {
                    None
                }
            ))
        } else {
            None
        };
        let effect_kind = if
            types
                .iter()
                .chain(receiver)
                .any(
                    |ty|
                        matches!(ty,Type::MutReference(inner) if self.properties.contains_exclusive(inner))
                )
        {
            LoanKind::Exclusive
        } else {
            LoanKind::Shared
        };
        self.emit(Operation::Call {
            result: id,
            arguments: values,
            summary,
            target: direct,
            effects,
            reference_effects,
            effect_loan: ExprId::fresh(),
            effect_kind,
            result_loan,
            scope,
            span: target.clone(),
        });
        self.emit(Operation::EndScope(call_scope));
        self.scopes.pop();
        if matches!(target.lexeme.as_str(), "panic" | "exit") {
            self.terminate(Terminator::Abort);
        }
    }
    fn parametric_projection(&self, expr: &Expr) -> bool {
        if let Expr::Get { object, name } = expr.untyped() {
            let mut ty = object.checked_type();
            while let Some(Type::Reference(t) | Type::MutReference(t)) = ty {
                ty = Some(t);
            }
            if let Some(Type::Custom(n)) = ty {
                return self.context.types
                    .get(n)
                    .is_some_and(|info|
                        self.context.fields
                            .get(name)
                            .is_some_and(|i| info.parametric_fields.contains(&(String::new(), *i)))
                    );
            }
        }
        false
    }
    fn source_type(&self, expr: &Expr) -> Option<Type> {
        match expr.untyped() {
            Expr::Variable(n) | Expr::This(n) => {
                if let Some(root) = self.resolve(&n.lexeme) {
                    self.source_types
                        .get(&root)
                        .cloned()
                        .or_else(|| (
                            if
                                self.parametric.contains(&root) &&
                                matches!(expr.checked_type(), Some(Type::Function(..)))
                            {
                                Some(Type::TypeVar(0))
                            } else {
                                expr.checked_type().cloned()
                            }
                        ))
                } else {
                    let symbol = match self.context.calls.get(n) {
                        Some(CallType::Static(s)) => s.as_str(),
                        _ => n.lexeme.as_str(),
                    };
                    self.contracts
                        .get(symbol)
                        .map(|(ps, ret, _)|
                            Type::Function(ps.clone(), Box::new(ret.clone()), false)
                        )
                        .or_else(|| expr.checked_type().cloned())
                }
            }
            Expr::Grouping(e) => self.source_type(e),
            Expr::Lambda { ownership_signature: Some((ps, ret)), .. } =>
                Some(Type::Function(ps.clone(), Box::new(ret.clone()), false)),
            Expr::Lambda { .. } => expr.checked_type().cloned(),
            Expr::Call { callee, .. } =>
                match self.source_type(callee) {
                    Some(Type::Function(_, ret, _)) => Some(*ret),
                    _ if matches!(expr.checked_type(), Some(Type::Function(..))) =>
                        Some(Type::TypeVar(0)),
                    _ => expr.checked_type().cloned(),
                }
            Expr::TupleLiteral { elements, .. } =>
                Some(
                    Type::Tuple(
                        elements
                            .iter()
                            .map(|e|
                                self
                                    .source_type(e)
                                    .or_else(|| e.checked_type().cloned())
                                    .unwrap_or(Type::Void)
                            )
                            .collect()
                    )
                ),
            Expr::Array { elements, .. } if !elements.is_empty() =>
                Some(
                    Type::Array(
                        elements.len(),
                        Box::new(
                            self
                                .source_type(&elements[0])
                                .or_else(|| elements[0].checked_type().cloned())?
                        )
                    )
                ),
            Expr::Dereference { operand, .. } =>
                match self.source_type(operand)? {
                    Type::Reference(t) | Type::MutReference(t) | Type::Pointer(t) => Some(*t),
                    _ => None,
                }
            Expr::AddressOf { operand, .. } =>
                self
                    .source_type(operand)
                    .map(|t| (
                        if matches!(expr.checked_type(), Some(Type::MutReference(_))) {
                            Type::MutReference(Box::new(t))
                        } else {
                            Type::Reference(Box::new(t))
                        }
                    )),
            Expr::Get { object, name } => {
                if let Some(mut ty) = self.source_type(object) {
                    while let Type::Reference(t) | Type::MutReference(t) = ty {
                        ty = *t;
                    }
                    if let Type::Tuple(ts) = ty {
                        return name.lexeme
                            .parse::<usize>()
                            .ok()
                            .and_then(|i| ts.get(i).cloned());
                    }
                }
                let target = self.context.specialized
                    .get(name)
                    .cloned()
                    .or_else(||
                        self.context.calls.get(name).map(|call| {
                            match call {
                                CallType::Instance(owner) =>
                                    (crate::symbols::member(owner, &name.lexeme), true),
                                CallType::Static(owner) =>
                                    (crate::symbols::member(owner, &name.lexeme), false),
                            }
                        })
                    );
                if let Some((symbol, instance)) = target {
                    if let Some((ps, ret, _)) = self.contracts.get(&symbol) {
                        return Some(
                            Type::Function(
                                ps.iter().skip(usize::from(instance)).cloned().collect(),
                                Box::new(ret.clone()),
                                false
                            )
                        );
                    }
                }
                if
                    let Some(Type::Custom(owner)) = object
                        .checked_type()
                        .map(Self::unreferenced_type)
                {
                    if
                        let Some(source) = self.context.fields
                            .get(name)
                            .and_then(|i|
                                self.context.types
                                    .get(owner)
                                    .and_then(|info|
                                        info.ownership_fields.get(&(String::new(), *i))
                                    )
                            )
                    {
                        return Some(source.clone());
                    }
                    if
                        self.context.types
                            .get(owner)
                            .is_some_and(|info|
                                self.context.fields
                                    .get(name)
                                    .is_some_and(|i|
                                        info.parametric_fields.contains(&(String::new(), *i))
                                    )
                            ) &&
                        matches!(expr.checked_type(), Some(Type::Function(..)))
                    {
                        return Some(Type::TypeVar(0));
                    }
                }
                expr.checked_type().cloned()
            }
            Expr::SubscriptGet { indexee, .. } => {
                let mut ty = self.source_type(indexee)?;
                while let Type::Reference(t) | Type::MutReference(t) = ty {
                    ty = *t;
                }
                match ty {
                    Type::Array(_, t) | Type::Slice(t) | Type::Pointer(t) => Some(*t),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    fn unreferenced_type(mut ty: &Type) -> &Type {
        while let Type::Reference(t) | Type::MutReference(t) = ty {
            ty = t;
        }
        ty
    }
    fn receiver_usage(&mut self, object: &Expr, receiver: Option<&Type>) -> Use {
        let mut actual = object.checked_type();
        while let Some(Type::Reference(t) | Type::MutReference(t)) = actual {
            actual = Some(t);
        }
        if matches!(actual, Some(Type::Pointer(_) | Type::String)) {
            match receiver {
                Some(Type::Reference(inner) | Type::MutReference(inner)) if
                    !matches!(inner.as_ref(), Type::Pointer(_) | Type::String | Type::TypeVar(_))
                =>
                    self.emit(Operation::Reject {
                        span: Self::span(object),
                        message: "cannot create a safe receiver reference from a raw address".into(),
                    }),
                Some(ty) if
                    !matches!(ty, Type::Reference(_) | Type::MutReference(_)) &&
                    !self.properties.get(ty).is_copy()
                =>
                    self.emit(Operation::Reject {
                        span: Self::span(object),
                        message: "cannot consume an owned receiver from a raw address".into(),
                    }),
                _ => {}
            }
        }
        match receiver {
            Some(Type::Reference(_)) => Use::Borrow(LoanKind::Shared),
            Some(Type::MutReference(_)) => Use::Borrow(LoanKind::Exclusive),
            Some(ty) => {
                self.receiver_values.insert(Self::id(object));
                if
                    !self.properties.get(ty).is_copy() &&
                    matches!(
                        object.checked_type(),
                        Some(Type::Reference(_) | Type::MutReference(_))
                    )
                {
                    self.emit(Operation::Reject {
                        span: Self::span(object),
                        message: "cannot consume an owned receiver through a reference".into(),
                    });
                }
                Use::Consume
            }
            None => Use::Consume,
        }
    }
    fn readonly_callable(&self, expr: &Expr) -> bool {
        match expr.untyped() {
            Expr::Grouping(e) => self.readonly_callable(e),
            Expr::Variable(n) | Expr::This(n) =>
                self
                    .resolve(&n.lexeme)
                    .map(|root| self.readonly_callables.contains(&root))
                    .unwrap_or(true),
            _ => self.readonly_expressions.contains(&Self::id(expr)),
        }
    }
    fn callable(&mut self, expr: &Expr) -> ExprId {
        let value = self.expression(expr, Use::Read);
        let readonly = self.readonly_callable(expr);
        if let Some(mut p) = self.base_place(expr) {
            // Repeatable invocation leases the environment; this is not a
            // mutable source binding or a transferable mutable reference.
            // Conservatively exclude overlapping invocations while a result
            // still borrows captures, including otherwise immutable captures.
            if !readonly && !p.writable {
                self.emit(Operation::Reject {
                    span: Self::span(expr),
                    message: format!(
                        "cannot invoke potentially stateful callable '{}' through an immutable binding",
                        self.function.bindings[&p.place.root].declaration.lexeme
                    ),
                });
            }
            p.writable = true;
            let place = self.place(p);
            let loan = ExprId::fresh();
            self.emit(Operation::Borrow {
                place,
                kind: LoanKind::Exclusive,
                value: loan,
                scope: self.scope(),
                span: Self::span(expr),
            });
            loan
        } else {
            // A reference derived from an anonymous callable cannot escape its
            // bounded invocation. Keep its captures' provenance for diagnostics.
            if !readonly {
                self.emit(Operation::Reject {
                    span: Self::span(expr),
                    message: "stateful callable requires a mutable binding".into(),
                });
            }
            value
        }
    }
    fn pattern_binding(
        &mut self,
        name: &Token,
        ty: Type,
        value: ExprId,
        abstract_value: bool,
        source: Option<Type>
    ) {
        let binding = self.declare(name, false, ty.clone(), true);
        if let Some(mut source) = source {
            if self.context.pattern_views.contains(&name.id) {
                source = if matches!(ty, Type::MutReference(_)) {
                    Type::MutReference(Box::new(source))
                } else {
                    Type::Reference(Box::new(source))
                };
            }
            self.source_types.insert(binding, source);
        } else if matches!(ty, Type::Function(..)) {
            self.source_types.insert(binding, Type::TypeVar(0));
        }
        if
            self.context.parametric_bindings.contains(&name.id) ||
            (abstract_value && !matches!(ty, Type::Reference(_) | Type::Pointer(_)))
        {
            self.parametric.insert(binding);
        }
        self.emit(Operation::Bind { binding, value, initialization: true, span: name.clone() });
    }
    fn pattern(
        &mut self,
        pattern: &Expr,
        value: ExprId,
        abstract_value: bool,
        actual: &Type,
        source: Option<Type>
    ) {
        match pattern.untyped() {
            Expr::Variable(name) => {
                let ty = pattern.checked_type().cloned().unwrap_or(Type::Void);
                self.pattern_binding(name, ty, value, abstract_value, source);
            }
            Expr::UnionPattern { case_name, bindings } => {
                let info = if let Type::Custom(n) = Self::unreferenced_type(actual) {
                    self.context.types.get(n)
                } else {
                    None
                };
                for (i, name) in bindings.iter().enumerate() {
                    let field = ExprId::fresh();
                    let projection = Projection::Field(i + 1);
                    if self.context.pattern_views.contains(&name.id) {
                        self.emit(Operation::View { value: field, source: value, projection });
                    } else {
                        self.emit(Operation::Project { value: field, source: value, projection });
                    }
                    let ty = self.context.pattern_bindings
                        .get(&name.id)
                        .cloned()
                        .unwrap_or(Type::Void);
                    let source = info.and_then(|info|
                        info.ownership_fields
                            .get(&(case_name.lexeme.clone(), i))
                            .cloned()
                            .or_else(||
                                info.static_methods
                                    .get(&case_name.lexeme)
                                    .and_then(|(ps, _, _)| ps.get(i).cloned())
                            )
                    );
                    self.pattern_binding(name, ty, field, abstract_value, source);
                }
            }
            Expr::ListPattern { elements, rest } => {
                for (i, p) in elements.iter().enumerate() {
                    let field = ExprId::fresh();
                    self.emit(Operation::Project {
                        value: field,
                        source: value,
                        projection: Projection::Index(Some(i)),
                    });
                    let actual = match Self::unreferenced_type(actual) {
                        Type::Array(_, t) | Type::Slice(t) => *t.clone(),
                        Type::Tuple(ts) => ts.get(i).cloned().unwrap_or(Type::Void),
                        _ => Type::Void,
                    };
                    let child = source.as_ref().and_then(|t| {
                        match Self::unreferenced_type(t) {
                            Type::Array(_, t) | Type::Slice(t) => Some(*t.clone()),
                            Type::Tuple(ts) => ts.get(i).cloned(),
                            Type::TypeVar(_) => Some(Type::TypeVar(0)),
                            _ => None,
                        }
                    });
                    self.pattern(p, field, abstract_value, &actual, child);
                }
                if let Some(rest) = rest {
                    self.pattern(rest, value, abstract_value, actual, source);
                }
            }
            Expr::ObjectPattern { properties, .. } => {
                let info = if let Type::Custom(n) = Self::unreferenced_type(actual) {
                    self.context.types.get(n)
                } else {
                    None
                };
                for (name, p) in properties {
                    let index = *self.context.fields.get(name).unwrap_or(&0);
                    let field = ExprId::fresh();
                    self.emit(Operation::Project {
                        value: field,
                        source: value,
                        projection: Projection::Field(index),
                    });
                    let actual = info
                        .and_then(|info| info.fields.get(&name.lexeme))
                        .map(|(_, ty, _)| ty.clone())
                        .unwrap_or(Type::Void);
                    let source = info
                        .and_then(|info| info.ownership_fields.get(&(String::new(), index)))
                        .cloned()
                        .or_else(|| Some(actual.clone()));
                    self.pattern(p, field, abstract_value, &actual, source);
                }
            }
            _ => {}
        }
    }
    fn statements(&mut self, statements: &[Stmt]) {
        for s in statements {
            self.statement(s);
        }
    }
    fn statement(&mut self, statement: &Stmt) {
        if self.current.is_none() {
            return;
        }
        match statement {
            Stmt::Var { name, mutable, type_annotation, initializer, ownership_type } => {
                let source = ownership_type
                    .clone()
                    .or_else(||
                        type_annotation
                            .as_ref()
                            .filter(|t| matches!(t, Type::Function(..)))
                            .cloned()
                    )
                    .or_else(|| initializer.as_ref().and_then(|e| self.source_type(e)));
                let temporary_scope = initializer.as_ref().map(|_| self.begin_scope());
                let value = initializer.as_ref().map(|e| self.expression(e, Use::Consume));
                let ty = type_annotation
                    .clone()
                    .or_else(|| initializer.as_ref().and_then(|e| e.checked_type().cloned()))
                    .unwrap_or(Type::Void);
                // Bind the result into its actual lexical scope before ending
                // expression temporaries. Only stored reference origins extend
                // their loans; scalar projections do not borrow for the block.
                if temporary_scope.is_some() {
                    self.scopes.pop();
                }
                let binding = self.declare(name, *mutable, ty, false);
                if !*mutable && initializer.as_ref().is_some_and(|e| self.readonly_callable(e)) {
                    self.readonly_callables.insert(binding);
                }
                if let Some(source) = source {
                    self.source_types.insert(binding, source);
                }
                if
                    value.is_some_and(|id| self.parametric_values.contains(&id)) ||
                    ownership_type.as_ref().is_some_and(|ty| matches!(ty, Type::TypeVar(_)))
                {
                    self.parametric.insert(binding);
                }
                if let Some(value) = value {
                    self.emit(Operation::Bind {
                        binding,
                        value,
                        initialization: true,
                        span: name.clone(),
                    });
                }
                if let Some(scope) = temporary_scope {
                    self.emit(Operation::EndScope(scope));
                }
            }
            Stmt::Expression(e) => {
                self.begin_scope();
                let usage = if
                    matches!(
                        e.untyped(),
                        Expr::Assign { .. } |
                            Expr::Set { .. } |
                            Expr::SubscriptSet { .. } |
                            Expr::DereferenceSet { .. }
                    )
                {
                    Use::Read
                } else {
                    Use::Consume
                };
                self.expression(e, usage);
                self.end_scope();
            }
            Stmt::Block(b) => {
                self.begin_scope();
                self.statements(b);
                self.end_scope();
            }
            Stmt::Return { keyword, value } => {
                let value = value.as_ref().map(|e| self.expression(e, Use::Consume));
                self.emit(Operation::Return { value, span: keyword.clone() });
                self.terminate(Terminator::Return);
            }
            Stmt::If { condition, then_branch, else_branch, .. } => {
                self.condition(condition);
                let yes = self.block();
                let no = self.block();
                let end = self.block();
                self.terminate(Terminator::Branch(yes, no));
                self.current = Some(yes);
                self.statement(then_branch);
                self.terminate(Terminator::Goto(end));
                self.current = Some(no);
                if let Some(b) = else_branch {
                    self.statement(b);
                }
                self.terminate(Terminator::Goto(end));
                self.current = Some(end);
            }
            Stmt::While { condition, body, .. } => {
                let test = self.block();
                let run = self.block();
                let end = self.block();
                self.terminate(Terminator::Goto(test));
                self.current = Some(test);
                self.condition(condition);
                self.terminate(Terminator::Branch(run, end));
                self.current = Some(run);
                self.statement(body);
                self.terminate(Terminator::Goto(test));
                self.current = Some(end);
            }
            Stmt::For { initializer, condition, increment, body, .. } => {
                self.begin_scope();
                if let Some(i) = initializer {
                    self.statement(i);
                }
                let test = self.block();
                let run = self.block();
                let end = self.block();
                self.terminate(Terminator::Goto(test));
                self.current = Some(test);
                if let Some(c) = condition {
                    self.condition(c);
                    self.terminate(Terminator::Branch(run, end));
                } else {
                    self.terminate(Terminator::Goto(run));
                }
                self.current = Some(run);
                self.begin_scope();
                self.statements(body);
                self.end_scope();
                if let Some(i) = increment {
                    self.expression(i, Use::Read);
                }
                self.terminate(Terminator::Goto(test));
                self.current = Some(end);
                self.end_scope();
            }
            Stmt::ForIn { key, value, iterable, body, .. } => {
                let source = self.source_type(iterable);
                let mut source = source.as_ref();
                while let Some(Type::Reference(t) | Type::MutReference(t)) = source {
                    source = Some(t);
                }
                if
                    self.generic_body &&
                    matches!(source,Some(Type::Array(_,t)|Type::Slice(t)) if matches!(t.as_ref(),Type::TypeVar(_)))
                {
                    self.emit(Operation::Reject {
                        span: key.clone(),
                        message: "generic array iteration requires an element borrow contract before specialization".into(),
                    });
                }
                self.begin_scope();
                let input = self.expression(iterable, Use::Read);
                let p = self.object_place(iterable).or_else(|| {
                    if matches!(iterable.checked_type(), Some(Type::Range(_))) {
                        return None;
                    }
                    let mut token = Self::span(iterable);
                    token.id = Token::fresh_id();
                    let root = self.declare(
                        &token,
                        false,
                        iterable.checked_type().cloned().unwrap_or(Type::Void),
                        true
                    );
                    self.emit(Operation::Bind {
                        binding: root,
                        value: input,
                        initialization: true,
                        span: token,
                    });
                    Some(PlaceInfo {
                        id: PlaceId(0),
                        place: Place { root, path: vec![] },
                        writable: false,
                        through: None,
                        raw: false,
                    })
                });
                let view = if let Some(p) = p {
                    let place = self.place(p);
                    let loan = ExprId::fresh();
                    self.emit(Operation::Borrow {
                        place,
                        kind: LoanKind::Shared,
                        value: loan,
                        scope: self.scope(),
                        span: Self::span(iterable),
                    });
                    loan
                } else {
                    input
                };
                let test = self.block();
                let run = self.block();
                let end = self.block();
                self.terminate(Terminator::Goto(test));
                self.current = Some(test);
                self.terminate(Terminator::Branch(run, end));
                self.current = Some(run);
                self.begin_scope();
                for name in std::iter::once(key).chain(value.iter()) {
                    let ty = self.context.pattern_bindings
                        .get(&name.id)
                        .cloned()
                        .unwrap_or(Type::I64);
                    let contains_borrow = self.properties.get(&ty).contains_borrow;
                    let root = self.declare(name, false, ty, true);
                    if
                        (value.is_none() || value.as_ref().is_some_and(|v| v.id == name.id)) &&
                        contains_borrow
                    {
                        self.emit(Operation::Bind {
                            binding: root,
                            value: view,
                            initialization: true,
                            span: name.clone(),
                        });
                    }
                }
                self.statements(body);
                self.end_scope();
                self.terminate(Terminator::Goto(test));
                self.current = Some(end);
                self.end_scope();
            }
            Stmt::ArrayDestructuring { bindings, initializer, .. } => {
                let source = self.source_type(initializer);
                let abstract_value =
                    self.generic_body && source.as_ref().is_some_and(Self::abstract_source);
                let value = self.expression(initializer, Use::Consume);
                for (i, name) in bindings.iter().enumerate() {
                    let field = ExprId::fresh();
                    let projection = if
                        matches!(initializer.checked_type(), Some(Type::Array(..) | Type::Slice(_)))
                    {
                        Projection::Index(Some(i))
                    } else {
                        Projection::Field(i)
                    };
                    self.emit(Operation::Project { value: field, source: value, projection });
                    let binding = self.declare(
                        name,
                        false,
                        self.context.pattern_bindings.get(&name.id).cloned().unwrap_or(Type::Void),
                        true
                    );
                    if
                        let Some(ty) = source.as_ref().and_then(|t| {
                            match t {
                                Type::Tuple(ts) => ts.get(i).cloned(),
                                Type::Array(_, t) | Type::Slice(t) => Some(*t.clone()),
                                _ => None,
                            }
                        })
                    {
                        self.source_types.insert(binding, ty);
                    } else if matches!(self.binding_types[&binding], Type::Function(..)) {
                        self.source_types.insert(binding, Type::TypeVar(0));
                    }
                    if
                        self.parametric_values.contains(&value) ||
                        (abstract_value &&
                            !matches!(
                                self.binding_types[&binding],
                                Type::Reference(_) | Type::Pointer(_)
                            ))
                    {
                        self.parametric.insert(binding);
                    }
                    self.emit(Operation::Bind {
                        binding,
                        value: field,
                        initialization: true,
                        span: name.clone(),
                    });
                }
            }
            Stmt::Alias { .. } => {}
            Stmt::Throw { keyword, .. } =>
                self.emit(Operation::Reject {
                    span: keyword.clone(),
                    message: "exceptions have no supported synchronous ownership lowering".into(),
                }),
            Stmt::TryCatch { exception, .. } =>
                self.emit(Operation::Reject {
                    span: exception.clone(),
                    message: "exceptions have no supported synchronous ownership lowering".into(),
                }),
            | Stmt::Function { name, .. }
            | Stmt::ExternFunction { name, .. }
            | Stmt::Struct { name, .. }
            | Stmt::Enum { name, .. }
            | Stmt::Trait { name, .. } =>
                self.emit(Operation::Reject {
                    span: name.clone(),
                    message: "named declarations in executable scopes require a module-level declaration; use a closure for local code".into(),
                }),
            | Stmt::Impl { keyword, .. }
            | Stmt::Using { keyword, .. }
            | Stmt::Export { keyword, .. } =>
                self.emit(Operation::Reject {
                    span: keyword.clone(),
                    message: "declaration must be resolved at module scope before ownership analysis".into(),
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_prelude_obeys_ownership_contracts() {
        let source = include_str!("stdlib.fih");
        let mut scanner = crate::scanner::Scanner::new(&source, "prelude.fih");
        scanner.scan_tokens();
        let mut parser = crate::parser::Parser::new(scanner.tokens);
        let mut ast = parser.parse();
        assert!(parser.errors.is_empty(), "{:?}", parser.errors);
        crate::ast_visit::inject_prelude_intrinsics(&mut ast);
        let mut loader = crate::module_loader::ModuleLoader::new();
        ast = loader.resolve_root(vec![], std::path::Path::new("."), ast);
        let mut checker = crate::type_checker::TypeChecker::new();
        checker.module_symbols = loader.symbols;
        checker.external_names = loader.external_names;
        checker.check(&mut ast);
        assert!(checker.errors.is_empty(), "{:?}", checker.errors);
        ast.extend(checker.instantiations.clone());
        let plan = Plan::build(
            &ast,
            &(Context {
                pattern_bindings: &checker.pattern_bindings,
                pattern_views: &checker.pattern_views,
                parametric_bindings: &checker.parametric_bindings,
                types: &checker.user_types,
                calls: &checker.resolved_calls,
                fields: &checker.property_indices,
                captures: &checker.capture_plans,
                defaults: &checker.default_calls,
                specialized: &checker.specialized_methods,
                traits: &checker.traits,
            })
        );
        let errors = plan.errors
            .iter()
            .map(|e| format!("{}:{} {}", e.span.line, e.span.lexeme, e.message))
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "{}", errors.join("\n"));
        let module = crate::ir_builder::IrBuilder
            ::new(
                checker.capture_plans,
                checker.property_indices,
                checker.resolved_calls,
                checker.specialized_methods,
                checker.default_calls,
                checker.traits,
                checker.trait_vtable_layout,
                checker.user_types
            )
            .with_ownership(plan)
            .build(&ast);
        if let Err(error) = crate::ir_verify::verify(&module) {
            let name = error.split(':').next().unwrap();
            panic!(
                "{error}\n{}",
                module.functions
                    .iter()
                    .find(|f| f.name == name)
                    .unwrap()
            );
        }
    }
    #[test]
    fn private_take_keeps_the_source_generic_element_noncopy() {
        let source =
            "fn duplicate<T>(owner:&mut T,ptr:^T)->(T,T){let value=__fishy_raw_take(owner,ptr,0);return(value,value);}fn main(){let mut n:u8=7;duplicate(n,0 as ^u8);}";
        let mut scanner = crate::scanner::Scanner::new(source, "trusted-storage-test.fih");
        scanner.scan_tokens();
        let mut parser = crate::parser::Parser::new(scanner.tokens);
        let mut ast = parser.parse();
        assert!(parser.errors.is_empty());
        crate::ast_visit::inject_prelude_intrinsics(&mut ast);
        let mut checker = crate::type_checker::TypeChecker::new();
        checker.check(&mut ast);
        assert!(checker.errors.is_empty(), "{:?}", checker.errors);
        ast.extend(checker.instantiations.clone());
        let plan = Plan::checked_program(&ast, &checker);
        assert!(
            plan.errors.iter().any(|e| e.message == "value 'value' was moved"),
            "{:?}",
            plan.errors
        );
    }
    fn checked(source: &str) -> Plan {
        let mut scanner = crate::scanner::Scanner::new(source, "ownership-test.fih");
        scanner.scan_tokens();
        let mut parser = crate::parser::Parser::new(scanner.tokens);
        let mut ast = parser.parse();
        assert!(parser.errors.is_empty(), "{:?}", parser.errors);
        let mut checker = crate::type_checker::TypeChecker::new();
        checker.check(&mut ast);
        assert!(checker.errors.is_empty(), "{:?}", checker.errors);
        ast.extend(checker.instantiations.clone());
        Plan::build(
            &ast,
            &(Context {
                pattern_bindings: &checker.pattern_bindings,
                pattern_views: &checker.pattern_views,
                parametric_bindings: &checker.parametric_bindings,
                types: &checker.user_types,
                calls: &checker.resolved_calls,
                fields: &checker.property_indices,
                captures: &checker.capture_plans,
                defaults: &checker.default_calls,
                specialized: &checker.specialized_methods,
                traits: &checker.traits,
            })
        )
    }
    fn lowered(source: &str) -> crate::ir::ModuleIr {
        let mut scanner = crate::scanner::Scanner::new(source, "ownership-ir.fih");
        scanner.scan_tokens();
        let mut parser = crate::parser::Parser::new(scanner.tokens);
        let mut ast = parser.parse();
        assert!(parser.errors.is_empty());
        let mut checker = crate::type_checker::TypeChecker::new();
        checker.check(&mut ast);
        assert!(checker.errors.is_empty(), "{:?}", checker.errors);
        ast.extend(checker.instantiations.clone());
        let plan = Plan::build(
            &ast,
            &(Context {
                pattern_bindings: &checker.pattern_bindings,
                pattern_views: &checker.pattern_views,
                parametric_bindings: &checker.parametric_bindings,
                types: &checker.user_types,
                calls: &checker.resolved_calls,
                fields: &checker.property_indices,
                captures: &checker.capture_plans,
                defaults: &checker.default_calls,
                specialized: &checker.specialized_methods,
                traits: &checker.traits,
            })
        );
        assert!(plan.errors.is_empty(), "{:?}", plan.errors);
        let module = crate::ir_builder::IrBuilder
            ::new(
                checker.capture_plans,
                checker.property_indices,
                checker.resolved_calls,
                checker.specialized_methods,
                checker.default_calls,
                checker.traits,
                checker.trait_vtable_layout,
                checker.user_types
            )
            .with_ownership(plan)
            .build(&ast);
        crate::ir_verify::verify(&module).unwrap();
        module
    }
    #[test]
    fn whole_moves_and_owned_returns_transfer_without_a_retain() {
        let module = lowered(
            "struct P{pub n:u8,}fn id(p:P)->P{return p;}fn main(){let a=P{n:1};let b=id(a);}"
        );
        for name in ["main", "id"] {
            let function = module.functions
                .iter()
                .find(|f| f.name == name)
                .unwrap();
            assert!(
                !function.blocks
                    .iter()
                    .flat_map(|b| &b.instructions)
                    .any(|i| matches!(i, crate::ir::Instruction::Retain { .. })),
                "{function}"
            );
        }
    }
    #[test]
    fn consumed_argument_still_has_cleanup_during_later_argument_early_return() {
        let module = lowered(
            "extern fn panic(^u8)->void;struct P{pub n:u8,}fn sink(a:P,b:P){}fn main(){sink(P{n:1},match true{true->{return;}false->P{n:2};});}"
        );
        let function = module.functions
            .iter()
            .find(|f| f.name == "main")
            .unwrap();
        assert!(
            function.blocks.iter().any(|b| b.name.starts_with("arc.release.#owner.")),
            "{function}"
        );
    }
    #[test]
    fn typed_cfg_detects_moves_and_conditional_initialization() {
        let plan = checked("struct P{pub n:u8,}fn main(){let a=P{n:1};let b=a;let c=a;}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("was moved")),
            "{:?}",
            plan.errors
        );
        let plan = checked("fn main(){let mut x:i64;if true{x=7;}let y=x;}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("may be uninitialized")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn returning_arm_does_not_poison_other_arm() {
        let plan = checked(
            "struct P{pub n:u8,}fn take(x:P){}fn run(c:bool){let p=P{n:1};if c{take(p);return;}let q=p;}"
        );
        assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    }
    #[test]
    fn mutable_binding_reinitializes_without_resurrecting_moved_value() {
        let plan = checked(
            "struct P{pub n:u8,}fn run(){let mut p=P{n:1};let q=p;p=P{n:2};let r=p;}"
        );
        assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    }
    #[test]
    fn scalar_specialization_does_not_grant_copy_to_unconstrained_parameter() {
        for source in [
            "fn duplicate<T>(x:T)->(T,T){return (x,x);}fn main(){duplicate(7);}",
            "fn duplicate<T>(x:T)->(T,T){let y=x;return (y,y);}fn main(){duplicate(7);}",
            "fn duplicate<T>(x:T)->(T,T){let pair=(x,7);let [a,b]=pair;return(a,a);}fn main(){duplicate(7);}",
            "fn duplicate<T>()->(T,T){let a:T=7;return(a,a);}fn main(){let p:(i64,i64)=duplicate();}",
            "fn duplicate<T>(a:T,b:T=a)->T{return b;}fn main(){duplicate(7);}",
            "fn duplicate<T>(x:T)->(T,T){let f=fn(y:T)->(T,T){return(y,y);};return f(x);}fn main(){duplicate(7);}",
        ] {
            let plan = checked(source);
            assert!(
                plan.errors.iter().any(|e| e.message.contains("was moved")),
                "{:?}",
                plan.errors
            );
        }
    }
    #[test]
    fn typed_expression_ids_survive_clone_and_fresh_specializations_remap() {
        let mut e = Expr::Typed {
            id: ExprId::fresh(),
            ty: Type::I64,
            expr: Box::new(Expr::Literal(Literal::Integer(1))),
        };
        assert_eq!(e.identity(), e.clone().identity());
        let old = e.identity();
        let mut statement = Stmt::Expression(e.clone());
        crate::ast_visit::fresh_instance(&mut statement);
        if let Stmt::Expression(fresh) = statement {
            assert_ne!(old, fresh.identity());
            e = fresh;
        }
        assert!(e.identity().is_some());
    }
    #[test]
    fn input_reference_summaries_and_local_escape_are_distinct() {
        let plan = checked(
            "struct P{pub n:u8,}fn id(p:&P)->&P{return p;}fn choose(a:&P,b:&P,c:bool)->&P{return c?a:b;}"
        );
        assert!(plan.errors.is_empty(), "{:?}", plan.errors);
        let choose = plan.functions
            .iter()
            .find(|f| f.name == "choose")
            .unwrap()
            .analyze();
        assert_eq!(choose.returned.all_origins().len(), 2);
        let plan = checked("struct P{pub n:u8,}fn bad()->&P{let p=P{n:1};return &p;}");
        assert!(
            plan.errors
                .iter()
                .any(|e| e.message.contains("returned reference points to local storage")),
            "{:?}",
            plan.errors
        );
        let plan = checked("fn bad()->&i64{let n:i64=1;return &n;}");
        assert!(
            plan.errors
                .iter()
                .any(|e| e.message.contains("returned reference points to local storage")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn call_auto_borrow_retains_no_owner_and_expires_at_the_call() {
        let module = lowered(
            "extern fn panic(^u8)->void;struct P{pub n:u8,}fn read(p:&P)->u8{return p.n;}fn main(){let p=P{n:1};read(p);read(p);let q=p;}"
        );
        for name in ["main", "read"] {
            let f = module.functions
                .iter()
                .find(|f| f.name == name)
                .unwrap();
            assert!(
                !f.blocks
                    .iter()
                    .flat_map(|b| &b.instructions)
                    .any(|i| matches!(i, crate::ir::Instruction::Retain { .. })),
                "{f}"
            );
        }
        native(
            "struct P{pub n:u8,}fn read(p:&P)->u8{return p.n;}fn main()->i32{printf(\"%ld\\n\",read(P{n:7}));return 0;}",
            b"7\n"
        );
        let plan = checked(
            "struct P{pub n:u8,}fn id(p:&P)->&P{return p;}fn run(){let r=id(P{n:7});}"
        );
        assert!(
            plan.errors.iter().any(|e| e.message.contains("escapes its owner")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn stored_reference_and_returned_call_reference_block_move() {
        for source in [
            "struct P{pub n:u8,}fn run(){let p=P{n:1};let r:&P=&p;let q=p;}",
            "struct P{pub n:u8,}fn id(p:&P)->&P{return p;}fn run(){let p=P{n:1};let r=id(p);let q=p;}",
        ] {
            let plan = checked(source);
            assert!(
                plan.errors.iter().any(|e| e.message.contains("while it is borrowed")),
                "{:?}",
                plan.errors
            );
        }
    }
    #[test]
    fn callable_identity_preserves_captured_borrow_origins() {
        let plan = checked(
            "struct P{pub n:u8,}fn id(f:fn()->u8)->fn()->u8{return f;}fn bad()->fn()->u8{let p=P{n:7};let r=&p;let f=fn()->u8{return r.n;};return id(f);}"
        );
        assert!(
            plan.errors
                .iter()
                .any(|e| e.message.contains("returned reference points to local storage")),
            "{:?}",
            plan.errors
        );
        let plan = checked(
            "trait Read{fn read(self)->u8;}struct H{pub r:&u8,}impl Read for H{pub fn read(self)->u8{return ^self.r;}}fn bad()->Read{let n:u8=7;return H{r:&n} as Read;}"
        );
        assert!(
            plan.errors
                .iter()
                .any(|e| e.message.contains("returned reference points to local storage")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn exclusive_borrow_requires_mutability_and_prohibits_aliases() {
        let plan = checked("struct P{pub n:u8,}fn run(){let p=P{n:1};let r:&mut P=&p;}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("immutable binding")),
            "{:?}",
            plan.errors
        );
        let plan = checked(
            "struct P{pub n:u8,}fn run(){let mut p=P{n:1};let r:&mut P=&p;let s:&mut P=&p;}"
        );
        assert!(
            plan.errors.iter().any(|e| e.message.contains("another borrow is active")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn reference_storage_effects_are_checked_at_the_callers_storage_lifetime() {
        let plan = checked(
            "fn put(dst:&mut &i64,src:&i64){^dst=src;}fn run(){let a:i64=1;let mut r:&i64=&a;{let b:i64=2;put(r,&b);}let x=^r;}"
        );
        assert!(
            plan.errors.iter().any(|e| e.message.contains("escapes its owner")),
            "{:?}",
            plan.errors
        );
        let plan = checked("fn bad(dst:&mut &i64){let a:i64=1;^dst=&a;}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("escapes its owner")),
            "{:?}",
            plan.errors
        );
        let plan = checked(
            "fn put(dst:&mut &i64,src:&i64){^dst=src;}fn run(){let a:i64=1;let b:i64=2;let mut r:&i64=&a;put(r,&b);let x=^r;}"
        );
        assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    }
    #[test]
    fn exclusive_projection_reborrow_keeps_ancestor_permissions() {
        let plan = checked(
            "struct P{pub a:[i16;2],}fn run(){let mut p=P{a:[2,3]};let a:&mut [i16;2]=&p.a;(^a)[1]=7;++((^a)[0]);{let ptr:&mut i16=&((^a)[1]);^ptr=8;}}"
        );
        assert!(
            plan.errors.is_empty(),
            "{:?}\n{}",
            plan.errors,
            plan.functions
                .iter()
                .find(|f| f.name == "run")
                .unwrap()
                .entry_trace()
        );
    }
    #[test]
    fn formal_exclusive_capability_cannot_escape_with_an_overlapping_child() {
        let plan = checked(
            "struct P{pub n:u8,}fn bad(r:&mut P)->(&mut P,&mut u8){let child:&mut u8=&r.n;let other=r;return(other,child);}"
        );
        assert!(!plan.errors.is_empty(), "overlapping exclusive capabilities accepted");
    }
    #[test]
    fn by_value_generic_exclusive_capability_moves_instead_of_reborrowing() {
        let plan = checked(
            "struct P{pub n:u8,}fn id<T>(x:T)->T{return x;}fn run(){let mut p=P{n:7};let r:&mut P=&p;let next=id(r);r.n=8;}"
        );
        assert!(
            plan.errors.iter().any(|e| e.message.contains("was moved")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn raw_storage_cannot_fabricate_trusted_safe_references() {
        for source in [
            "fn run(p:^i64){let r:&i64=&(^p);}",
            "fn run(p:^i64){let r:&i64=&p[0];}",
            "fn run(p:^&i64){let r:&i64=^p;}",
        ] {
            let plan = checked(source);
            assert!(!plan.errors.is_empty(), "raw provenance accepted: {source}");
        }
        native(
            "fn main()->i32{let p=malloc(8);let address=p as u64;let r:&^u8=&p;free(p);printf(\"%ld\\n\",(^r as u64)==address);return 0;}",
            b"1\n"
        );
    }
    #[test]
    fn global_effect_summaries_protect_borrowed_storage_and_initialization() {
        let plan = checked(
            "let mut g:i64=1;fn change(){g=2;}fn run(){let r=&g;change();let n=^r;}"
        );
        assert!(
            plan.errors.iter().any(|e| e.message.contains("global 'g'")),
            "{:?}",
            plan.errors
        );
        let plan = checked("let a:i64=read();let b:i64=2;fn read()->i64{return b;}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("not initialized")),
            "{:?}",
            plan.errors
        );
    }
    #[test]
    fn native_uniform_references_and_move_transfers_have_zero_live_allocations() {
        native(
            "struct P{pub n:u8,}fn read(p:&P)->u8{return p.n;}fn bump(p:&mut P){p.n=p.n+1;}fn id(p:P)->P{return p;}fn main()->i32{let mut p=P{n:7};read(p);bump(p);printf(\"%ld\\n\",read(p));let q=id(p);return 0;}",
            b"8\n"
        );
    }
    #[test]
    fn native_dynamic_receiver_borrows_the_concrete_handle_cell() {
        native(
            "trait Read{fn read(self)->u8;}struct P{pub n:u8,}impl Read for P{pub fn read(self)->u8{return self.n;}}fn main()->i32{let t=P{n:9} as Read;printf(\"%ld %ld\\n\",t.read(),t.read());return 0;}",
            b"9 9\n"
        );
        native(
            "trait Read{fn read(self)->u8;fn twice(self)->u8{return self.read()+self.read();}}struct P{pub n:u8,}impl Read for P{pub fn read(self)->u8{return self.n;}}fn main()->i32{let t=P{n:9} as Read;printf(\"%ld\\n\",t.twice());return 0;}",
            b"18\n"
        );
    }
    #[test]
    fn array_iteration_borrows_noncopy_elements_and_protects_the_owner() {
        let plan = checked("struct P{pub n:u8,}fn run(){let a=[P{n:1}];for p in a{let b=a;}}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("while it is borrowed")),
            "{:?}",
            plan.errors
        );
        native(
            "struct P{pub n:u8,}fn main()->i32{let a=[P{n:7},P{n:8}];for p in a{printf(\"%ld\\n\",p.n);}return 0;}",
            b"7\n8\n"
        );
    }
    #[test]
    fn native_borrowed_enum_views_and_consuming_payload_extraction() {
        native(
            "struct P{pub n:u8,}enum E{One(P),None,}impl E{pub fn unwrap(self:E)->P{return match self{One(p)->p;None->P{n:0};};}pub fn view(self)->&P{return match self{One(p)->p;None->{panic(\"empty\");}};}}fn main()->i32{let e=E.One(P{n:7});{let r=e.view();printf(\"%ld\\n\",r.n);}let p=e.unwrap();printf(\"%ld\\n\",p.n);return 0;}",
            b"7\n7\n"
        );
    }
    #[test]
    fn closure_owner_capture_moves_and_repeatable_invocation_borrows_environment() {
        let plan = checked(
            "struct P{pub n:u8,}fn run(){let p=P{n:7};let f=fn()->u8{return p.n;};let q=p;}"
        );
        assert!(
            plan.errors.iter().any(|e| e.message.contains("was moved")),
            "{:?}",
            plan.errors
        );
        let plan = checked("struct P{pub n:u8,}fn run(){let p=P{n:7};let f=fn()->P{return p;};}");
        assert!(
            plan.errors.iter().any(|e| e.message.contains("owned capture")),
            "{:?}",
            plan.errors
        );
        native(
            "struct P{pub n:u8,}fn main()->i32{let p=P{n:7};let f=fn()->u8{return p.n;};printf(\"%ld %ld\\n\",f(),f());return 0;}",
            b"7 7\n"
        );
    }
    fn native(body: &str, expected: &[u8]) {
        use std::{ fs, process::Command };
        let source = format!(
            "extern fn malloc(u64)->^u8;extern fn free(^u8)->void;extern fn panic(^u8)->void;extern fn printf(^u8,...)->i32;{body}"
        );
        let module = lowered(&source);
        let directory = std::env
            ::temp_dir()
            .join(format!("fishy-ownership-native-{}-{}", std::process::id(), Token::fresh_id()));
        fs::create_dir_all(&directory).unwrap();
        let context = inkwell::context::Context::create();
        let mut emitter = crate::backend::LlvmEmitter::new(&context, "ownership_native");
        emitter.compile(&module, &HashMap::new()).unwrap();
        emitter.build_executable(directory.join("program.o").to_str().unwrap()).unwrap();
        fs::write(
            directory.join("support.c"),
            "#include <stdlib.h>\nvoid panic(char *message){abort();}\n"
        ).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let output = Command::new("cc")
            .current_dir(&directory)
            .args(["program.o", "support.c"])
            .arg(root.join("tests/ownership/heap_guard.c"))
            .args([
                "-Wl,--wrap=malloc",
                "-Wl,--wrap=free",
                "-Wl,--wrap=realloc",
                "-lm",
                "-o",
                "program",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let output = Command::new(directory.join("program"))
            .env("FISHYC_EXPECT_LIVE", "0")
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, expected);
    }
}
