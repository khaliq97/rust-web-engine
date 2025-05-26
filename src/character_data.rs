use crate::node::{DOMString, RefNode, WeakNode};

// https://dom.spec.whatwg.org/#characterdata
pub struct CharacterData { 
    pub data: DOMString,
    pub length: usize,
}
 
 impl CharacterData { 
 
     #[allow(dead_code)]
     pub fn new (data: DOMString) -> Self { 
         Self { data: data.to_owned(), length: data.len() }
     }
 
     #[allow(dead_code)]
     // https://dom.spec.whatwg.org/#dom-characterdata-substringdata
     pub fn substring_data(offset: u32, count: u32) -> DOMString { 
         todo!()
     }
 
     #[allow(dead_code)]
     // https://dom.spec.whatwg.org/#dom-characterdata-appenddata
     pub fn append_data(data: DOMString) { 
         todo!()
     }
 
     #[allow(dead_code)]
     // https://dom.spec.whatwg.org/#dom-characterdata-insertdata
     pub fn insert_data(offset: u32, data: DOMString) { 
         todo!()
     }
 
     #[allow(dead_code)]
     // https://dom.spec.whatwg.org/#dom-characterdata-replacedata
     pub fn replace_data(offset: u32, count: u32, data: DOMString) { 
         todo!()
     }
 
 }
 
 