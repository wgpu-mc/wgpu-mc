use std::fmt::{Display, Formatter};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ResourcePath(pub String);

impl ResourcePath {
    pub fn append(&self, a: &str) -> Self {
        Self(format!("{}{}", self.0, a))
    }

    pub fn prepend(&self, a: &str) -> Self {
        let mut split = self.0.split(':');

        Self(format!("{}:{}{}", split.next().unwrap(), a, split.next().unwrap()))
    }
}

impl Display for ResourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ResourcePath {

    fn from(string: &str) -> Self {
        // Parse the rest of the namespace
        let split = string.split(':').collect::<Vec<&str>>();

        match (split.first(), split.get(1)) {
            (Some(path), None) => Self(format!("minecraft:{}", path)),
            (Some(namespace), Some(path)) => Self(format!("{}:{}", namespace, path)),
            _ => Self("".into())
        }
    }

}

impl From<&String> for ResourcePath {

    fn from(string: &String) -> Self {
        // Parse the rest of the namespace
        let split = string.split(':').collect::<Vec<&str>>();

        match (split.first(), split.get(1)) {
            (Some(path), None) => Self(format!("minecraft:{}", path)),
            (Some(namespace), Some(path)) => Self(format!("{}:{}", namespace, path)),
            _ => Self("".into())
        }
    }

}

impl From<String> for ResourcePath {

    fn from(string: String) -> Self {
        // Parse the rest of the namespace
        let split = string.split(':').collect::<Vec<&str>>();

        match (split.first(), split.get(1)) {
            (Some(path), None) => Self(format!("minecraft:{}", path)),
            (Some(_namespace), Some(_path)) => Self(string),
            _ => Self("".into())
        }
    }

}

impl From<(&str, &str)> for ResourcePath {
    fn from(strings: (&str, &str)) -> Self {
        Self(format!("{}:{}", strings.0, strings.1))
    }
}
pub trait ResourceProvider: Send + Sync {
    fn get_bytes(&self, id: &ResourcePath) -> Option<Vec<u8>>;

    fn get_string(&self, id: &ResourcePath) -> Option<String> {
        String::from_utf8(self.get_bytes(id)?).ok()
    }
}