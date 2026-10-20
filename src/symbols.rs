//! Compiler-generated symbols occupy a reserved, unambiguous namespace.
use crate::ast::Type;

pub fn member(owner: &str, name: &str) -> String {
    format!(
        "__fishy_member_{}_{}_{}_{}",
        owner.len(),
        owner,
        name.len(),
        name
    )
}

pub fn vtable(owner: &str, name: &str) -> String {
    format!(
        "__fishy_vtable_{}_{}_{}_{}",
        owner.len(),
        owner,
        name.len(),
        name
    )
}

pub fn generic(base: &str, arguments: &[Type]) -> String {
    // Debug's enum/field delimiters preserve the complete type tree, including
    // nominal names, pointer levels, arity, and variadic signatures. Hex encoding
    // keeps the result usable as a native symbol without losing that identity.
    let encoding = format!("{arguments:?}");
    let suffix: String = encoding.bytes().map(|b| format!("{b:02x}")).collect();
    format!("__fishy_generic_{}_{}_{}", base.len(), base, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nominal_names_and_member_boundaries_cannot_alias() {
        assert_ne!(member("A_B", "C"), member("A", "B_C"));
        assert_ne!(
            generic("F", &[Type::Custom("ptr_i64".into())]),
            generic("F", &[Type::Pointer(Box::new(Type::I64))])
        );
        assert_ne!(
            generic("A_B", &[Type::Custom("C".into())]),
            generic("A", &[Type::Custom("B_C".into())])
        );
    }
}
