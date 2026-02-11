pub fn ideator_prompt(idea: &str, num_concepts: u32) -> (String, String) {
    let system = format!(
        "You are a creative director brainstorming visual concepts. Given a simple idea, \
generate {} distinctly different creative interpretations. Each should be a \
unique visual direction — vary the style, mood, setting, or perspective.\n\n\
Output as a numbered list. Each concept should be 2-3 sentences describing the \
visual scene. Be specific and vivid. Think like a cinematographer.",
        num_concepts
    );

    let user = format!("User's idea: {}", idea);
    (system, user)
}

pub fn composer_prompt(concept: &str) -> (String, String) {
    let system = "You are a visual scene designer. Take this concept and enrich it with specific \
visual details that would make it a stunning image.\n\n\
Add: specific materials and textures, lighting direction and quality, color \
palette (name specific colors), camera angle and lens characteristics, \
atmospheric effects, small details that add realism or charm.\n\n\
Do NOT write in prompt syntax. Write a rich paragraph of natural description."
        .to_string();

    let user = format!("Concept: {}", concept);
    (system, user)
}

pub struct CheckpointContext {
    pub checkpoint_name: String,
    pub base_model: String,
    pub strengths: String,
    pub weaknesses: String,
    pub cfg_range_low: String,
    pub cfg_range_high: String,
    pub preferred_sampler: String,
    pub checkpoint_notes: String,
    pub term_list: String,
}

impl Default for CheckpointContext {
    fn default() -> Self {
        Self {
            checkpoint_name: "unknown".to_string(),
            base_model: "SD 1.5".to_string(),
            strengths: "general purpose".to_string(),
            weaknesses: "text rendering, hands".to_string(),
            cfg_range_low: "6.0".to_string(),
            cfg_range_high: "9.0".to_string(),
            preferred_sampler: "dpmpp_2m".to_string(),
            checkpoint_notes: "No specific notes available.".to_string(),
            term_list: "No specific term data available.".to_string(),
        }
    }
}

pub fn judge_prompt(original_idea: &str, concepts: &[String]) -> (String, String) {
    let system = "You are an art director evaluating visual concepts for image generation with \
Stable Diffusion 1.5. Rank these concepts from best to worst.\n\n\
Evaluate each on:\n\
1. Visual clarity — can this be rendered as a single coherent image?\n\
2. SD-friendliness — does it avoid things SD1.5 struggles with (hands, text, \
   multiple specific characters, complex spatial relationships)?\n\
3. Composition — is there a clear focal point and visual hierarchy?\n\
4. Faithfulness — does it honor the user's original idea?\n\
5. Appeal — would this make someone go \"wow\"?\n\n\
Return a JSON array ranked best-to-worst:\n\
[{\"rank\": 1, \"concept_index\": <n>, \"score\": <0-100>, \"reasoning\": \"...\"}, ...]"
        .to_string();

    let numbered: Vec<String> = concepts
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}. {}", i, c))
        .collect();

    let user = format!(
        "Original idea: {}\n\nConcepts:\n{}",
        original_idea,
        numbered.join("\n")
    );

    (system, user)
}

pub fn prompt_engineer_prompt(description: &str, ctx: &CheckpointContext) -> (String, String) {
    let system = format!(
        "You are an expert Stable Diffusion prompt engineer. Convert this scene \
description into optimized positive and negative prompts.\n\n\
TARGET CHECKPOINT: {checkpoint_name}\n\
Base model: {base_model}\n\n\
CHECKPOINT BEHAVIORAL PROFILE:\n\
Strengths: {strengths}\n\
Weaknesses: {weaknesses}\n\
Preferred CFG: {cfg_range_low}–{cfg_range_high}\n\
Preferred sampler: {preferred_sampler}\n\
Notes: {checkpoint_notes}\n\n\
KNOWN EFFECTIVE TERMS FOR THIS CHECKPOINT:\n\
{term_list}\n\n\
Rules:\n\
- Use comma-separated tags, not sentences\n\
- Put the most important elements first\n\
- Use (parentheses:weight) for emphasis, range 0.5-1.5\n\
- Include quality boosters: masterpiece, best quality, highly detailed\n\
- Negative prompt should cover common SD artifacts\n\
- Keep total positive prompt under 75 tokens (CLIP limit for SD1.5)\n\
- Match the style to the scene (photorealistic → photo terms, illustration → art terms)\n\
- Prefer terms known to be effective on the target checkpoint\n\
- Avoid terms known to be weak or broken on the target checkpoint\n\n\
Respond in EXACTLY this JSON format:\n\
{{\"positive\": \"the positive prompt here\", \"negative\": \"the negative prompt here\"}}",
        checkpoint_name = ctx.checkpoint_name,
        base_model = ctx.base_model,
        strengths = ctx.strengths,
        weaknesses = ctx.weaknesses,
        cfg_range_low = ctx.cfg_range_low,
        cfg_range_high = ctx.cfg_range_high,
        preferred_sampler = ctx.preferred_sampler,
        checkpoint_notes = ctx.checkpoint_notes,
        term_list = ctx.term_list,
    );

    let user = format!("Scene description:\n{}", description);
    (system, user)
}

pub fn reviewer_prompt(
    original_idea: &str,
    positive: &str,
    negative: &str,
) -> (String, String) {
    let system = "Compare this SD prompt against the user's original idea. Check for:\n\
1. Prompt drift — did we lose the core of what they asked for?\n\
2. Conflicting terms — anything contradictory?\n\
3. Token bloat — is the prompt over-stuffed?\n\
4. Missing elements — anything from the original idea that got dropped?\n\n\
If the prompts are good, respond: {\"approved\": true}\n\
If changes needed, respond: {\"approved\": false, \"issues\": [...], \
\"suggested_positive\": \"...\", \"suggested_negative\": \"...\"}"
        .to_string();

    let user = format!(
        "Original idea: {}\nPositive prompt: {}\nNegative prompt: {}",
        original_idea, positive, negative
    );

    (system, user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ideator_prompt_contains_count_and_idea() {
        let (system, user) = ideator_prompt("a cat on a throne", 5);
        assert!(system.contains("5 distinctly different"));
        assert!(user.contains("a cat on a throne"));
    }

    #[test]
    fn test_composer_prompt_contains_concept() {
        let (system, user) = composer_prompt("Gothic black cat on iron throne");
        assert!(system.contains("visual scene designer"));
        assert!(user.contains("Gothic black cat"));
    }

    #[test]
    fn test_judge_prompt_numbers_concepts() {
        let concepts = vec![
            "Concept A".to_string(),
            "Concept B".to_string(),
            "Concept C".to_string(),
        ];
        let (system, user) = judge_prompt("cat throne", &concepts);
        assert!(system.contains("art director"));
        assert!(user.contains("0. Concept A"));
        assert!(user.contains("1. Concept B"));
        assert!(user.contains("2. Concept C"));
        assert!(user.contains("cat throne"));
    }

    #[test]
    fn test_prompt_engineer_prompt_includes_checkpoint_context() {
        let ctx = CheckpointContext {
            checkpoint_name: "dreamshaper_8.safetensors".to_string(),
            base_model: "SD 1.5".to_string(),
            strengths: "photorealism, cinematic lighting".to_string(),
            weaknesses: "text rendering".to_string(),
            cfg_range_low: "6.0".to_string(),
            cfg_range_high: "9.0".to_string(),
            preferred_sampler: "dpmpp_2m".to_string(),
            checkpoint_notes: "Good all-around".to_string(),
            term_list: "cinematic lighting (strong): volumetric rays".to_string(),
        };
        let (system, user) = prompt_engineer_prompt("A cat on a throne", &ctx);
        assert!(system.contains("dreamshaper_8.safetensors"));
        assert!(system.contains("SD 1.5"));
        assert!(system.contains("photorealism"));
        assert!(system.contains("dpmpp_2m"));
        assert!(system.contains("cinematic lighting (strong)"));
        assert!(user.contains("A cat on a throne"));
    }

    #[test]
    fn test_reviewer_prompt_includes_all_inputs() {
        let (system, user) = reviewer_prompt(
            "cat on throne",
            "masterpiece, best quality, cat on throne",
            "lowres, bad anatomy",
        );
        assert!(system.contains("Prompt drift"));
        assert!(user.contains("cat on throne"));
        assert!(user.contains("masterpiece"));
        assert!(user.contains("lowres"));
    }

    #[test]
    fn test_checkpoint_context_default() {
        let ctx = CheckpointContext::default();
        assert_eq!(ctx.base_model, "SD 1.5");
        assert!(ctx.checkpoint_notes.contains("No specific notes"));
    }
}
