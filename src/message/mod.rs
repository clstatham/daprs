use std::fmt::{Debug, Display};

#[derive(Debug, Clone)]
pub enum Message {
    Bang,
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Message>),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Bang => write!(f, "bang"),
            Message::Int(i) => write!(f, "{}", i),
            Message::Float(x) => write!(f, "{}", x),
            Message::String(s) => write!(f, "{}", s),
            Message::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl Message {
    pub fn is_same_type(&self, other: &Message) -> bool {
        matches!(
            (self, other),
            (Message::Bang, Message::Bang)
                | (Message::Int(_), Message::Int(_))
                | (Message::Float(_), Message::Float(_))
                | (Message::String(_), Message::String(_))
                | (Message::List(_), Message::List(_))
        )
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Message::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Message::Float(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Message::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Message]> {
        match self {
            Message::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn is_bang(&self) -> bool {
        matches!(self, Message::Bang)
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Message::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Message::Float(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Message::String(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Message::List(_))
    }

    pub fn cast_to_int(&self) -> Option<i64> {
        match self {
            Message::Int(i) => Some(*i),
            Message::Float(x) => Some(x.round() as i64),
            Message::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn cast_to_float(&self) -> Option<f64> {
        match self {
            Message::Int(i) => Some(*i as f64),
            Message::Float(x) => Some(*x),
            Message::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn cast_to_string(&self) -> Option<String> {
        match self {
            Message::Bang => Some("bang".to_string()),
            Message::Int(i) => Some(i.to_string()),
            Message::Float(x) => Some(x.to_string()),
            Message::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn cast_to_list(&self) -> Option<Vec<Message>> {
        match self {
            Message::List(list) => Some(list.clone()),
            msg => Some(vec![msg.clone()]),
        }
    }
}
