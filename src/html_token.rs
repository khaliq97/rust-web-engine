use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub enum HtmlTokenType { 
    DocType,
    StartTag,
    EndTag,
    Comment,
    Character,
    EndOfFile
}

#[derive(Clone)]
pub struct HtmlToken { 
    pub token_type: HtmlTokenType,

    pub name: String,
    pub public_identifier: String,
    pub system_identifier: String,
    pub force_quirks: bool,

    pub tag_name: String,
    pub self_closing: bool,
    pub attributes: HashMap<String, String>,

    pub data: String
}

impl HtmlToken { 
    fn attributes_to_string(&self) -> String { 
        let mut attributes_string = String::from("");
        for (name, value) in self.attributes.iter() { 
            let s = format!("  Name: {}\n    Value: {}\n", name, value);
            attributes_string.push_str(&s);
        }

        if self.attributes.len() > 0 { 
            return attributes_string
        } else {
            return "(None)".to_string();
        }
        
    }
}

impl fmt::Display for HtmlToken { 
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.token_type { 
            HtmlTokenType::DocType => { 
                write!(f, "[{} token\n  Name:{}\n  Public identifier:{}\n  System identifier:{}\n  Force quirks:{}]", 
                self.token_type.to_string(),
                self.name,
                self.public_identifier,
                self.system_identifier,
                self.force_quirks
                )
              
            }
            HtmlTokenType::StartTag => { 
                write!(f, "[{} token\n  Tag name:{}\n  Self closing:{}\n  Attributes:\n{}]", 
                self.token_type.to_string(),
                self.tag_name,
                self.self_closing,
                self.attributes_to_string()
                )
            }
            HtmlTokenType::EndTag => { 
                write!(f, "[{} token\n  Tag name:{}\n  Self closing:{}\n  Attributes:\n{}]", 
                self.token_type.to_string(),
                self.tag_name,
                self.self_closing,
                self.attributes_to_string()
                )
            }
            HtmlTokenType::Comment => { 
                write!(f, "[{} token\n  Data:'{}']", 
                self.token_type.to_string(),
                self.data
                )
            }
            HtmlTokenType::Character => { 
                write!(f, "[{} token\n  Data:'{}']", 
                self.token_type.to_string(),
                self.data
                )
            }
            HtmlTokenType::EndOfFile => { 
                write!(f, "[{} token]", 
                self.token_type.to_string()
                )
            }
        }
    }
}


impl fmt::Display for HtmlTokenType { 
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result { 
        match self { 
            HtmlTokenType::DocType => write!(f, "DocType"),
            HtmlTokenType::StartTag => write!(f, "StartTag"),
            HtmlTokenType::EndTag => write!(f, "EndTag"),
            HtmlTokenType::Comment => write!(f, "Comment"),
            HtmlTokenType::Character => write!(f, "Character"),
            HtmlTokenType::EndOfFile => write!(f, "EndOfFile"),
        }
    }
}