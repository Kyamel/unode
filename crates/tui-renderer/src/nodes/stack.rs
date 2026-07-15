use unode::core::ast::StackNode;

pub fn gap_lines(node: &StackNode) -> u16 {
    match node.gap {
        Some(unode::core::ast::Gap::None) | None => 0,
        Some(unode::core::ast::Gap::Xs) | Some(unode::core::ast::Gap::Sm) => 0,
        Some(unode::core::ast::Gap::Md) | Some(unode::core::ast::Gap::Lg) => 1,
    }
}
