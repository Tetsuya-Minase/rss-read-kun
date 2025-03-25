use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Part {
    pub text: String
}

#[derive(Serialize, Debug)]
pub struct Content {
    pub parts: Vec<Part>
}

#[derive(Serialize, Debug)]
pub struct GeminiRequest {
    pub contents: Vec<Content>
}