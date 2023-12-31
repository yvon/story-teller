use crate::chat::Message;
use jsonschema::JSONSchema;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug)]
pub struct Body {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<Function>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

#[allow(dead_code)]
#[derive(Serialize, Debug)]
pub enum FunctionCall {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "name")]
    Name(&'static str),
}

#[derive(Serialize, Debug)]
pub struct Function {
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            model: String::from("gpt-3.5-turbo-0613"),
            messages: Vec::new(),
            functions: None,
            function_call: None,
        }
    }
}

impl Function {
    pub fn new(
        name: &'static str,
        description: Option<&'static str>,
        parameters: Option<Value>,
    ) -> Result<Self, &'static str> {
        if let Some(value) = &parameters {
            if JSONSchema::compile(value).is_err() {
                return Err("Invalid JSON schema");
            }
        }

        Ok(Self {
            name,
            description,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::tests::user_message;
    use serde_json::json;

    #[test]
    fn test_request_body_serialization() {
        let body = Body {
            model: String::from("gpt-3.5-turbo"),
            messages: vec![user_message()],
            functions: None,
            function_call: None,
        };

        let json = serde_json::to_string(&body).unwrap();

        assert_eq!(
            json,
            r#"{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello, world!"}]}"#
        );
    }

    #[test]
    fn test_function_serialization() {
        let function = Function {
            name: "foo",
            description: Some("Stuff"),
            parameters: None,
        };

        let json = serde_json::to_string(&function).unwrap();

        assert_eq!(json, r#"{"name":"foo","description":"Stuff"}"#);
    }

    #[test]
    fn test_valid_json_format() {
        let schema = json!({"maxLength": 5});
        let function = Function::new("foo", None, Some(schema)).unwrap();
        let json = serde_json::to_string(&function).unwrap();
        assert_eq!(json, r#"{"name":"foo","parameters":{"maxLength":5}}"#);
    }

    #[test]
    fn test_invalid_json_format() {
        let schema = json!({"type": "invalidType"});
        let function = Function::new("foo", None, Some(schema));
        assert!(function.is_err());
    }

    #[test]
    fn test_function_call_none() {
        let body = Body {
            model: String::from("gpt-3.5-turbo"),
            messages: vec![user_message()],
            functions: None,
            function_call: Some(FunctionCall::None),
        };

        let json = serde_json::to_string(&body).unwrap();

        assert_eq!(
            json,
            r#"{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello, world!"}],"function_call":"none"}"#
        );
    }

    #[test]
    fn test_function_call_with_name() {
        let body = Body {
            model: String::from("gpt-3.5-turbo"),
            messages: vec![user_message()],
            functions: None,
            function_call: Some(FunctionCall::Name("foo")),
        };

        let json = serde_json::to_string(&body).unwrap();

        assert_eq!(
            json,
            r#"{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello, world!"}],"function_call":{"name":"foo"}}"#
        );
    }
}
