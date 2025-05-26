use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::character_data::CharacterData;
use crate::comment::Comment;

#[derive(Debug)]
pub enum NodeType {
    ELEMENT_NODE,
    ATTRIBUTE_NODE,
    TEXT_NODE,
    CDATA_SECTION_NODE,
    ENTITY_REFERENCE_NODE,
    ENTITY_NODE,
    PROCESSING_INSTRUCTION_NODE,
    COMMENT_NODE,
    DOCUMENT_NODE,
    DOCUMENT_TYPE_NODE,
    DOCUMENT_FRAGMENT_NODE,
    NOTATION_NODE,
}

// https://dom.spec.whatwg.org/#node
pub struct Node {
    pub data: NodeData,
    pub nodeType: NodeType,
    nodeName: DOMString,
    baseURI: USVString,
    isConnected: bool,
    pub ownerDocument: Option<WeakNode>,
    pub parentNode: Option<WeakNode>,
    pub childNodes: Children,
    firstChild: Weak<Option<Child>>,
    lastChild: Weak<Option<Child>>,
    previousSibling: Weak<Option<Child>>,
    nextSibling: Weak<Option<Child>>,
    nodeValue: Option<DOMString>,
    textContent: Option<DOMString>,
}

// https://dom.spec.whatwg.org/#interface-document
pub struct Document {}

impl Document {
    pub fn new() -> Self {
        Self {}
    }

}

// https://dom.spec.whatwg.org/#interface-document-type
pub struct DocumentType {
    pub name: DOMString,
    pub public_id: DOMString,
    pub system_id: DOMString,
}

impl DocumentType {
    pub fn new(name: DOMString, public_id: DOMString, system_id: DOMString) -> Self {
        Self { name, public_id, system_id }
    }
}

// https://dom.spec.whatwg.org/#domtokenlist
pub struct DOMTokenList {
}

// https://dom.spec.whatwg.org/#namednodemap
pub struct NamedNodeMap {

}
// https://dom.spec.whatwg.org/#interface-element
pub struct Element {
    namespace_URI: Option<DOMString>,
    prefix: Option<DOMString>,
    local_name: DOMString,
    tag_name: DOMString,
    id: DOMString,
    class_list: DOMString,
    slot: DOMString,
    classList: DOMTokenList,
    attributes: NamedNodeMap,
}



impl Element {
    pub fn new(local_name: DOMString) -> Self {
        Self {
            namespace_URI: None,
            prefix: None,
            local_name,
            tag_name: "".to_string(),
            id: "".to_string(),
            class_list: "".to_string(),
            slot: "".to_string(),
            classList: DOMTokenList {},
            attributes: NamedNodeMap {},
        }
    }
}

pub struct HTMLElement { 
    element: Element,
}

pub struct Text {
    pub character_data: CharacterData,
}

impl Text {
    pub fn new (data: Option<DOMString>) -> Self {
        match data {
            Some(data) => {
                Self { character_data: CharacterData::new(data) }
            },
            _ => { Self { character_data: CharacterData::new("".to_owned()) }}
        }

    }
}

pub type RefNode = Rc<RefCell<Node>>;
pub type WeakNode = Weak<RefCell<Node>>;
pub type Children = Vec<Child>;
pub type Child = RefNode;

impl Node { 
    pub fn new(data: NodeData, node_type: NodeType) -> Self {
        Self { nodeType: node_type, nodeName: "".to_string(), baseURI: "".to_string(), isConnected: false, ownerDocument: None, parentNode: None, childNodes: Vec::new(), firstChild: Default::default(), lastChild: Default::default(), previousSibling: Default::default(), nextSibling: Default::default(), nodeValue: Option::from("".to_string()), textContent: Option::from("".to_string()), data }
    }

    // https://dom.spec.whatwg.org/#concept-node-append
    // TODO: Not to spec
    pub fn append_child(&mut self, child_node: RefNode) {
        self.childNodes.push(child_node);
    }
}

pub fn create_ref_node(data: NodeData, node_type: NodeType) -> RefNode {
    return Rc::new(RefCell::new(Node::new(data, node_type)));
}

pub enum NodeData {
    Comment(Comment),
    Document(Document),
    DocumentType(DocumentType),
    Element(Element),
    CharacterData(CharacterData),
    Text(Text),
}

pub type DOMString = String;
pub type USVString = String;

