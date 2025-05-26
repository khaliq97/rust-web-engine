use crate::character_data::CharacterData;
use crate::node::DOMString;

pub struct Comment { 
    pub character_data: CharacterData
}

impl Comment {
     pub fn new (data: Option<DOMString>) -> Self {
        match data { 
            Some(data) => {
                Self { character_data: CharacterData::new(data) }
             },
             _ => { Self { character_data: CharacterData::new("".to_owned()) }}
        }
        
    }
}

