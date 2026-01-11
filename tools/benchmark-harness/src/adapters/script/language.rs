//! Script language enum and metadata.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    Python,
    Ruby,
    Php,
    Node,
    Wasm,
    Java,
    CSharp,
    Go,
    Elixir,
}

impl ScriptLanguage {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Node => "node",
            Self::Wasm => "wasm",
            Self::Java => "java",
            Self::CSharp => "csharp",
            Self::Go => "go",
            Self::Elixir => "elixir",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::Python,
            Self::Ruby,
            Self::Php,
            Self::Node,
            Self::Wasm,
            Self::Java,
            Self::CSharp,
            Self::Go,
            Self::Elixir,
        ]
    }
}
