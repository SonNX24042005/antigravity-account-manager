use serde_json::{json, Value};
use crate::models::ChatCompletionRequest;

pub struct Mappers;

impl Mappers {
    pub fn openai_to_cloudcode(req: &ChatCompletionRequest) -> Value {
        let contents: Vec<Value> = req
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "user" => "user",
                    "assistant" => "model",
                    "system" => "user",
                    _ => "user",
                };

                let text = match &msg.content {
                    Some(Value::String(s)) => s.clone(),
                    Some(val) => val.to_string(),
                    None => "".to_string(),
                };

                json!({
                    "role": role,
                    "parts": [{"text": text}]
                })
            })
            .collect();

        json!({
            "model": req.model,
            "project": "antigravity-relay",
            "request": {
                "contents": contents,
                "generationConfig": {
                    "temperature": req.temperature.unwrap_or(0.7),
                    "topP": req.top_p.unwrap_or(0.95),
                    "maxOutputTokens": req.max_tokens.unwrap_or(8192)
                }
            }
        })
    }
}
