use serde::{Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct EmbedField {
    pub name: String,
    pub value: String
}
#[derive(Serialize, Debug, Clone)]
pub struct Embed {
    pub title: String,
    pub url: String,
    pub fields: Vec<EmbedField>
}
#[derive(Serialize, Debug)]
pub struct EmbedData {
    pub embeds: Vec<Embed>
}
