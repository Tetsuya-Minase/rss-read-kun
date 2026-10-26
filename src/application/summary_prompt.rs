pub const SUMMARY_PROMPT: &str = include_str!("../../prompts/summary_prompt.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t32_summary_prompt_content() {
        assert!(!SUMMARY_PROMPT.is_empty());
        assert!(SUMMARY_PROMPT.contains("🤖 AI・LLM"));
    }

    #[test]
    fn test_t35_prompt_contains_all_categories() {
        use crate::infrastructure::gemini::schema::CATEGORY_NAMES;
        for category in CATEGORY_NAMES {
            assert!(SUMMARY_PROMPT.contains(category), "Prompt does not contain category: {}", category);
        }
    }
}
