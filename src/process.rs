use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location(pub String);

impl Location {
    pub fn new(s: &str) -> Self {
        Location(s.to_string())
    }
}

impl fmt::Display for Location {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(pub String);

impl Label {
    pub fn new(s: &str) -> Self {
        Label(s.to_string())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

pub type Guard<T> = fn(&T) -> bool;

pub type Action<T> = fn(T) -> T;

pub struct Trans<T> {
    pub label: Label,
    pub dst: Location,
    pub guard: Guard<T>,
    pub action: Action<T>,
}

pub struct ExecUnit<T> {
    pub src: Location,
    pub transs: Vec<Trans<T>>,
}

pub type Process<T> = Vec<ExecUnit<T>>;
