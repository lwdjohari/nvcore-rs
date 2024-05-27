pub fn indent_space(level: u32) -> String {
    std::iter::repeat("  ")
        .take(level as usize)
        .collect::<String>()
        .clone()
}