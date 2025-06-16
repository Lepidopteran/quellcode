pub enum ProgressMessageKind {
    Starting,
    #[default]
    Misc,
    Error,
    Warning,
    Success,
}

#[derive(Default, Debug, Clone)]
pub struct ProgressMessage {
    pub index: u32,
    pub package_name: String,
    pub message: String,
    pub kind: ProgressMessageKind,
}

impl ProgressMessage {
    pub fn new(
        index: u32,
        package_name: String,
        message: String,
        kind: ProgressMessageKind,
    ) -> Self {
        Self {
            index,
            package_name,
            message,
            kind,
        }
    }
}

