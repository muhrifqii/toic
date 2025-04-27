use ic_llm::{chat, ChatMessage, Model, Role};

const EXPAND_SYSTEM_PROMPT: &str = r#"
You are an AI writing assistant. Expand the user's paragraph by adding two to three meaningful sentences that match the original tone and topic.
Respond with the expanded text only.
If you cannot comply, respond with the exact keyword ::FAILED::.
"#;

const STORY_DETAIL_SYSTEM_PROMPT: &str = r#"
You are an AI writing assistant. Provide a brief, promotional description or headline based on the story content that will attract readers.
Respond with the description only.
If you cannot comply, respond with the exact keyword ::FAILED::.
"#;

pub async fn expand_paragraph(text: String) -> Result<String, String> {
    let system = ChatMessage {
        role: Role::System,
        content: EXPAND_SYSTEM_PROMPT.to_string(),
    };
    let user = ChatMessage {
        role: Role::User,
        content: text,
    };
    let response = chat(Model::Llama3_1_8B, vec![system, user]).await;
    if response == "::FAILED::" {
        return Err("Failed to expand paragraph".to_string());
    }

    Ok(response.to_string())
}

pub async fn write_story_description(text: String) -> Result<String, String> {
    let system = ChatMessage {
        role: Role::System,
        content: STORY_DETAIL_SYSTEM_PROMPT.to_string(),
    };
    let user = ChatMessage {
        role: Role::User,
        content: text,
    };
    let response = chat(Model::Llama3_1_8B, vec![system, user]).await;
    if response == "::FAILED::" {
        return Err("Failed to write story description".to_string());
    }

    Ok(response.to_string())
}
