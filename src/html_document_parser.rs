use std::cell::RefCell;
use std::process::abort;
use std::rc::Rc;
use web_engine::node::{Node};
use crate::node::{DOMString, Document, DocumentType, Element, NodeType, Text, WeakNode};
use crate::node::NodeData;
use crate::comment::Comment;
use crate::html_token::{HtmlToken, HtmlTokenType};
use crate::node;
use crate::node::create_ref_node;
use crate::node::RefNode;

enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoScript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

pub struct HTMLDocumentParser {
    insertion_mode: InsertionMode,
    document: RefNode,
    stack_of_open_elements: Vec<WeakNode>,
    head_element: Option<WeakNode>,
}

impl HTMLDocumentParser {
    pub fn new() -> HTMLDocumentParser {
        let document = create_document_node();
        let mut stack_of_open_elements: Vec<WeakNode> = Vec::new();
        stack_of_open_elements.push(Rc::downgrade(&document));
        
        return HTMLDocumentParser {
            insertion_mode: InsertionMode::Initial,
            document: create_document_node(),
            stack_of_open_elements,
            head_element: None,
        }
    }

    pub fn parse_html_token(&mut self, html_token: &HtmlToken) {
            // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
            match self.insertion_mode {
                InsertionMode::Initial => {
                    match html_token.token_type {
                        HtmlTokenType::Character => {
                            if (html_token.data == "\u{0009}" || html_token.data == "\u{000A}" || html_token.data == "\u{000C}" || html_token.data == "\u{000D}" || html_token.data == "\u{0020}") {
                                // Ignore the token.
                            }
                        },
                        HtmlTokenType::Comment => {
                            self.document.borrow_mut().append_child(create_comment_node(Some(html_token.data.to_owned()), &self.document, &self.document));
                        },
                        HtmlTokenType::DocType => {
                            if (html_token.name != "html"
                                || html_token.public_identifier.len() != 0
                                || (html_token.system_identifier.len() != 0 && html_token.system_identifier != "about:legacy-compat")) {
                                panic!("Parse Error: Invalid DOCTYPE");
                            } else {
                                self.document.borrow_mut().append_child(create_document_type_node(html_token.name.to_owned(), html_token.public_identifier.to_owned(), html_token.system_identifier.to_owned()));
                            }

                            // TODO: Support quirks mode for document

                            self.switch_to_insertion_mode(InsertionMode::BeforeHtml);
                        }
                        _ => {
                            // TODO: If the document is not an iframe srcdoc document, then this is a parse error; if the parser cannot change the mode flag is false, set the Document to quirks mode.
                            self.switch_to_insertion_mode(InsertionMode::BeforeHtml)
                        }
                    }
                },
                // https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
                InsertionMode::BeforeHtml => {
                    match html_token.token_type {
                        HtmlTokenType::DocType => {
                            panic!("Parse Error: Unexpected DOCTYPE");
                        },
                        HtmlTokenType::Comment => {
                            self.document.borrow_mut().append_child(create_comment_node(Some(html_token.data.to_owned()), &self.document, &self.document));
                        },
                        HtmlTokenType::Character => {
                            if (html_token.data == "\u{0009}" || html_token.data == "\u{000A}" || html_token.data == "\u{000C}" || html_token.data == "\u{000D}" || html_token.data == "\u{0020}") {
                                // Ignore the token.
                            }
                        },
                        HtmlTokenType::StartTag => {
                            if (html_token.tag_name == "html") {
                                let element_node = self.create_element_node_for_token(html_token.tag_name.to_owned());
                                let element_node_clone = Rc::clone(&element_node);

                                self.document.borrow_mut().append_child(element_node);
                                self.stack_of_open_elements.push(Rc::downgrade(&element_node_clone));

                                self.switch_to_insertion_mode(InsertionMode::BeforeHead);
                            }
                        },
                        HtmlTokenType::EndTag => {
                            match html_token.tag_name.as_str() {
                                "head" | "body" | "html" | "br" => {
                                    let element_node = self.create_element_node_for_token(html_token.tag_name.to_owned());
                                    let element_node_clone = Rc::clone(&element_node);

                                    self.document.borrow_mut().append_child(element_node);
                                    self.stack_of_open_elements.push(Rc::downgrade(&element_node_clone));

                                    self.switch_to_insertion_mode(InsertionMode::BeforeHead);
                                },
                                _ => {
                                    panic!("Parse Error: Unexpected end tag. Ignore the token.");
                                }
                            }
                        }
                        _ => { }
                    }
                },
                // https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
                InsertionMode::BeforeHead => {
                    match html_token.token_type {
                        HtmlTokenType::Character => {
                            if (html_token.data == "\u{0009}" || html_token.data == "\u{000A}" || html_token.data == "\u{000C}" || html_token.data == "\u{000D}" || html_token.data == "\u{0020}") {
                                // Ignore the token.
                            }
                        },
                        HtmlTokenType::Comment => {
                            let appropriate_place_for_inserting_a_node = self.appropriate_place_for_inserting_a_node(None).upgrade().unwrap();
                            appropriate_place_for_inserting_a_node.borrow_mut().append_child(create_comment_node(Some(html_token.data.to_owned()), &appropriate_place_for_inserting_a_node, &self.document));
                        },
                        HtmlTokenType::DocType => {
                            panic!("Parse Error: Unexpected DOCTYPE. Ignore the token.");
                        },
                        HtmlTokenType::StartTag => {
                            // Process the token using the rules for the "in body" insertion mode.
                            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
                            match html_token.tag_name.as_str() {
                                "html" => {
                                    println!("Parse Error: Unexpected html start tag.");

                                    todo!()
                                    /*
                                    TODO:
                                    If there is a template element on the stack of open elements, then ignore the token.

                                    Otherwise, for each attribute on the token,
                                    check to see if the attribute is already present on the top element of the stack of open elements.
                                    If it is not, add the attribute and its corresponding value to that element.
                                     */
                                },
                                "head" => {
                                    let head_element_node = self.create_element_node_for_token(html_token.tag_name.to_owned());
                                    self.head_element = Some(Rc::downgrade(&head_element_node));
                                    
                                    self.appropriate_place_for_inserting_a_node(None).upgrade().unwrap().borrow_mut().append_child(head_element_node);

                                    self.switch_to_insertion_mode(InsertionMode::InHead);
                                },
                                _ => {}

                            }
                        },
                        HtmlTokenType::EndTag => {
                            match html_token.tag_name.as_str() {
                                "head" | "body" | "html" | "br" => {
                                    todo!()
                                    // Anything else
                                    /*
                                        Insert an HTML element for a "head" start tag token with no attributes.

                                        Set the head element pointer to the newly created head element.

                                        Switch the insertion mode to "in head".

                                        Reprocess the current token.
                                     */
                                },
                                _ => {
                                    panic!("Parse Error: Unexpected end tag. Ignore the token.");
                                }
                            }
                        }
                        _ => {}
                    }


                },
                InsertionMode::InHead => {
                    match html_token.token_type {
                        HtmlTokenType::Character => {
                            if (html_token.data == "\u{0009}" || html_token.data == "\u{000A}" || html_token.data == "\u{000C}" || html_token.data == "\u{000D}" || html_token.data == "\u{0020}") {
                                // https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character

                                // 1. Let data be the characters passed to the algorithm, or, if no characters were explicitly specified, the character of the character token being processed
                                let character = &html_token.data;

                                // 2. Let the adjusted insertion location be the appropriate place for inserting a node.
                                let adjusted_insertion_location = &self.appropriate_place_for_inserting_a_node(None);

                                // 3. If the adjusted insertion location is in a Document node, then return.
                                match adjusted_insertion_location.upgrade().unwrap().borrow().nodeType {
                                    NodeType::DOCUMENT_NODE => {
                                        return;
                                    },
                                    _ => {}
                                }

                                match &mut self.stack_of_open_elements[self.stack_of_open_elements.len() - 2].upgrade().unwrap().borrow_mut().data {
                                    // 4. If there is a Text node immediately before the adjusted insertion location, then append data to that Text node's data.
                                    node::NodeData::Text(ref mut text) => {
                                        text.character_data.data.push_str(&character);
                                    }
                                    // Otherwise, create a new Text node whose data is data and whose node document is the same as that of the element in which the adjusted insertion location finds itself,
                                    // and insert the newly created node at the adjusted insertion location.
                                    _ => {
                                        let text_node = self.create_text_node(character.clone());
                                        self.stack_of_open_elements.push(Rc::downgrade(&text_node));
                                        adjusted_insertion_location.upgrade().unwrap().borrow_mut().append_child(text_node);
                                    }
                                }

                            }
                        },
                        _ => {}
                    }
                }
                _ => {}
            }

    }

    fn current_node(&self) -> WeakNode {
        return self.stack_of_open_elements[self.stack_of_open_elements.len() - 1].clone();
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node
    fn appropriate_place_for_inserting_a_node(&self, override_target: Option<&RefNode>) -> WeakNode {
        let mut target = self.current_node();

        // 1. If there was an override target specified, then let target be the override target.
        if override_target.is_some() {
            target = Rc::downgrade(override_target.unwrap());
        }

        // TODO: 2. Determine the adjusted insertion location using the first matching steps from the following list:

        // TODO: 3. If the adjusted insertion location is inside a template element, let it instead be inside the template element's template contents, after its last child (if any).

        return target;
    }

    // This can be used for non-foreign elements but I think the spec implies that the logic is shared for both foreign and non-foreign
    // https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    fn insert_a_foreign_element(&mut self, tag_name: String) -> WeakNode {
        // 1. Let the adjustedInsertionLocation be the appropriate place for inserting a node.
        let adjusted_insertion_location = &self.appropriate_place_for_inserting_a_node(None);

        // 2. Let element be the result of creating an element for the token given token, namespace, and the element in which the adjustedInsertionLocation finds itself.
        let element = self.create_element_node_for_token(tag_name);

        // TODO: 3. If onlyAddToElementStack is false, then run insert an element at the adjusted insertion location with element.

        // 4. Push element onto the stack of open elements so that it is the new current node.
        self.stack_of_open_elements.push(Rc::downgrade(&element));

        return Rc::downgrade(&element);

    }

    fn switch_to_insertion_mode(&mut self, new_insertion_mode: InsertionMode) {
        self.insertion_mode = new_insertion_mode;
    }

    pub fn print_document(&self) {
        self.print_node(&self.document, 0);
    }

    fn print_node(&self, node: &RefNode, depth: usize) {
        let indent = "  ".repeat(depth);

        let node_ref = node.borrow();

        println!("{}- {:?}", indent, node_ref.nodeType);

        if let Some(parent_weak) = &node_ref.parentNode {
            if let Some(parent) = parent_weak.upgrade() {
                let parent_ref = parent.borrow();
                println!("{}    Parent Node Type: {:?}", indent, parent_ref.nodeType);
            }
        }
        
        if let Some(owner_weak) = &node_ref.ownerDocument {
            if let Some(owner) = owner_weak.upgrade() {
                let owner_ref = owner.borrow();
                println!("{}    Owner Document Node Type: {:?}", indent, owner_ref.nodeType);
            }
        }

        // Recursively print all child nodes
        for child in &node_ref.childNodes {
            self.print_node(child, depth + 1);
        }
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token
    pub fn create_element_node_for_token(&self, tag_name: DOMString) -> RefNode {
        // TODO: Only steps 3, 4 and 10 are done.

        // 3. Let document be intendedParent's node document.
        let document = Rc::downgrade(&self.document);

        // 4. Let localName be token's tag name.
        let localName = tag_name.clone();


        // 10. Let element be the result of creating an element given document, localName, namespace, null, is, willExecuteScript, and registry.
        let element_node = self.create_element(document, localName, None, None, None, false);
        return element_node;
    }

    // https://dom.spec.whatwg.org/#concept-create-element
    // TODO: Add 'registry' param for CustomElementRegistry object
    pub fn create_element(&self, document: WeakNode, local_name: DOMString, namespace: Option<String>, prefix: Option<String>, is: Option<String>, synchronous_custom_elements : bool) -> RefNode {
        // 1. Let result be null

        // TODO: 2. If registry is "default", then set registry to the result of looking up a custom element registry given document.

        // TODO: 3. Let definition be the result of looking up a custom element definition given registry, namespace, localName, and is.

        // TODO: 4. If definition is non-null, and definition’s name is not equal to its local name (i.e., definition represents a customized built-in element):

        // TODO: 5. Otherwise, if definition is non-null:

        // 6. Otherwise:

        // 1. Let interface be the element interface for localName and namespace.

        // Partial TODO: 2. Set result to the result of creating an element internal given document, interface, localName, namespace, prefix, "uncustomized", is, and registry.
        let element_node = create_ref_node(NodeData::Element(Element::new(local_name)), NodeType::ELEMENT_NODE);
        element_node.borrow_mut().ownerDocument = Some(document);
        element_node.borrow_mut().parentNode = Some(self.appropriate_place_for_inserting_a_node(None));

        // TODO: 3. If namespace is the HTML namespace, and either localName is a valid custom element name or is is non-null, then set result’s custom element state to "undefined".
        return element_node;
    }

    pub fn create_text_node(&self, data: DOMString) -> RefNode {
        let text_node =  create_ref_node(NodeData::Text(Text::new(Some(data))), NodeType::TEXT_NODE);

        let document = Rc::downgrade(&self.document);
        text_node.borrow_mut().ownerDocument = Some(document);
        text_node.borrow_mut().parentNode = Some(self.appropriate_place_for_inserting_a_node(None));

        return text_node;
    }

}

// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment
pub fn create_comment_node(data: Option<DOMString>, parent_node: &RefNode, owner_document: &RefNode) -> RefNode {
    let comment_node = create_ref_node(NodeData::Comment(Comment::new(data)), NodeType::COMMENT_NODE);
    comment_node.borrow_mut().ownerDocument = Some(Rc::downgrade(owner_document));
    comment_node.borrow_mut().parentNode = Some(Rc::downgrade(parent_node));

    return comment_node;
}

pub fn create_document_node() -> RefNode {
    return create_ref_node(NodeData::Document(Document::new()), NodeType::DOCUMENT_NODE)
}

pub fn create_document_type_node(name: DOMString, public_id: DOMString, system_id: DOMString) -> RefNode {
    return create_ref_node(NodeData::DocumentType(DocumentType::new(name, public_id, system_id)), NodeType::DOCUMENT_TYPE_NODE)
}

