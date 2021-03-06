

/// Events received from the Main thread.
pub enum FromMainEvent {
    Shutdown,
}

pub enum ToMainEvent {
    AllOk,
}

pub struct FileLines {
    pub filename: String,
    pub lines: Vec<String>,
}

/// Events send by the FilesProcessor
pub enum FileEvent {
    Lines(FileLines)
}