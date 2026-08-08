#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleOperation {
    Quit,
    Start,
    Restart,
    Skip,
    Unknown,
}

impl ConsoleOperation {
    pub fn from_input(input: &str) -> Self {
        match input.trim() {
            "!выход" | "!quit" => ConsoleOperation::Quit,
            "!старт" | "!start" => ConsoleOperation::Start,
            "!рестарт" | "!restart" => ConsoleOperation::Restart,
            "!пропуск" | "!skip" => ConsoleOperation::Skip,
            _ => ConsoleOperation::Unknown,
        }
    }
}
