use std::any::Any;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::ops::Deref;
use std::process::exit;
use std::rc::{Rc, Weak};
use crate::token::{Token, TokenType, Literal};
use crate::scanner::Scanner;
use crate::parser::Parser;
use crate::ast::{Statement, ExpressionStatement, BinaryExpression, LiteralExpression, ParenthesizedExpression, UnaryExpression, IdentifierExpression, VariableDeclarationStatement, AstVisitor, Accept, Callable, CallExpression, BlockStatement, ObjectLiteralExpression, AssignmentExpression};
use crate::ast_printer::ASTPrettyPrinter;

pub struct Interpreter {
    had_error: bool,
    //https://tc39.es/ecma262/#sec-execution-contexts
    execution_contexts: Vec<ExecutionContext>,
}

// https://tc39.es/ecma262/#sec-execution-contexts
struct ExecutionContext {
    lexical_environment_record: Rc<RefCell<EnvironmentRecord>>,
    variable_environment_record:  Rc<RefCell<EnvironmentRecord>>
}

// https://tc39.es/ecma262/#sec-ecmascript-language-types-symbol-type
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
struct JSSymbol {
    description: String,
}

impl JSSymbol {
    pub fn new(description: String) -> JSSymbol {
        JSSymbol { description: description }
    }
}

// https://tc39.es/ecma262/#property-key
#[derive(Debug)]
#[derive(Eq, Hash, PartialEq)]
enum PropertyKey {
    String(String),
    Symbol(JSSymbol),
}

#[derive(Debug)]
struct DataProperty {
    value: Rc<RefCell<JSValue>>,
    writable: bool,
    enumerable: bool,
    configurable: bool,
}
#[derive(Debug)]
struct AccessorProperty {
    get: Option<fn(key: PropertyKey, receiver: &JSValue) -> JSObject>,
    set: Option<fn(key: PropertyKey, value: JSValue, receiver: &JSValue) -> JSObject>
}

#[derive(Debug)]
enum PropertyType {
    DataProperty(DataProperty),
    AccessorProperty(AccessorProperty),
}


// https://tc39.es/ecma262/#sec-object-type
#[derive(Debug)]
struct JSObject {
    // https://tc39.es/ecma262/#table-object-property-attributes
    values: HashMap<PropertyKey, Rc<PropertyType>>,
    pub prototype: Option<Rc<JSObject>>,
    pub extensible: bool,
}
// https://tc39.es/ecma262/#sec-property-descriptor-specification-type
#[derive(Debug)]
struct PropertyDescriptor {
   property: Option<PropertyType>
}

#[derive(Debug)]
enum PropertyDescriptorType {
    PropertyDescriptor(PropertyDescriptor),
    Undefined(JSValue)
}
impl JSObject {
    pub fn new() -> JSObject {
        JSObject { values: HashMap::new(), prototype: None, extensible: false }
    }

/*    fn value(&self, key: PropertyKey) -> Option<&DataProperty> {
        match key {
            PropertyKey::String(key) => {
                return self.values.get(&PropertyKey::String(key));
            },
            PropertyKey::Symbol(key) => {
                return self.values.get(&PropertyKey::Symbol(key));
            }
        }
    }*/

    // https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
    // TODO: Return a normal completion instead of a raw JSValue
    pub fn get(&self, key: &PropertyKey, receiver: &Rc<RefCell<JSObject>>) -> Rc<RefCell<JSValue>> {
        return self.ordinary_get(key, receiver);
    }

    // https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver
    // TODO: reciever should be of type Ecmascript Language Value (JSValue)
    pub fn set(&mut self, key: Rc<PropertyKey>, value: Rc<RefCell<JSValue>>, receiver: &Rc<RefCell<JSObject>>) -> CompletionRecord {
        // 1. Return ? OrdinarySet(O, P, V, Receiver).
        return self.ordinary_set(key, value, receiver);
    }

    // https://tc39.es/ecma262/#sec-ordinaryset
    fn ordinary_set(&mut self, property_key: Rc<PropertyKey>, value: Rc<RefCell<JSValue>>, receiver: &Rc<RefCell<JSObject>>) -> CompletionRecord {
        // 1. Let ownDesc be ? O.[[GetOwnProperty]](P).
        let own_descriptor = self.get_own_property(&*property_key);

        // 2. Return ? OrdinarySetWithOwnDescriptor(O, P, V, Receiver, ownDesc).
        match &*own_descriptor.value {
            ReferenceRecordOrJsValue::PropertyDescriptor(property_descriptor) => {
                return self.ordinary_set_with_own_descriptor(property_key, value, receiver, property_descriptor)
            },
            _ => {
                unimplemented!();
            }
        }
    }

    fn handle_data_property_set(
        &mut self,
        property_key: &PropertyKey,
        value: Rc<RefCell<JSValue>>,
        property_descriptor: PropertyDescriptor,
    ) -> CompletionRecord {
        match &property_descriptor.property {
            // 2. If IsDataDescriptor(ownDesc) is true, then
            Some(PropertyType::DataProperty(data_property)) => {
                // a. If ownDesc.[[Writable]] is false, return false.
                if !data_property.writable {
                    return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                }

                // b. If Receiver is not an Object, return false.
                // TODO: receiver param is always an JSObject, but should be of type Ecmascript Language Value (JSValue)

                // c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
                let existing_descriptor = self.get_own_property(&*property_key);

                match &*existing_descriptor.value {
                    ReferenceRecordOrJsValue::PropertyDescriptor(property_descriptor_type) => {
                        match property_descriptor_type {
                            // d. If existingDescriptor is not undefined, then
                            PropertyDescriptorType::PropertyDescriptor(property_descriptor) => {
                                match &property_descriptor.property {
                                    Some(PropertyType::AccessorProperty(_)) => {
                                        // i. If IsAccessorDescriptor(existingDescriptor) is true, return false.
                                        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                                    },
                                    Some(PropertyType::DataProperty(data_property)) => {
                                        // ii. If existingDescriptor.[[Writable]] is false, return false.
                                        if !data_property.writable {
                                            return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                                        }
                                        //               iii. Let valueDesc be the PropertyDescriptor { [[Value]]: V }.
                                        // TODO Need to find a way to make the fields writable,enumerable etc configurable and not just initialize them anyway
                                        let value_desc = PropertyDescriptor { property: Some(PropertyType::DataProperty(DataProperty { value: Rc::clone(&value), writable: true, enumerable: data_property.enumerable, configurable: data_property.configurable })) };
                                        //               iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).

                                        // TODO: We need to call DefineOwnProperty on the receiver as otherwise it will set the field on the parent object,
                                        // currently we have issues with the borrow checker so using self for now.
                                        // This will be required when we fully support the prototype chain
                                        return self.define_own_property(&*property_key, value_desc);
                                    },

                                    _ => {
                                        unimplemented!();
                                    }
                                }
                            },
                            // e. Else,
                            PropertyDescriptorType::Undefined(_) => {
                                // i. Assert: Receiver does not currently have a property P.
                                // ii. Return ? CreateDataProperty(Receiver, P, V). TODO Implement CreateDataProperty
                                let value_desc = PropertyDescriptor { property: Some(PropertyType::DataProperty(DataProperty { value: Rc::clone(&value), writable: true, enumerable: data_property.enumerable, configurable: data_property.configurable })) };
                                // iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).

                                // TODO: We need to call DefineOwnProperty on the receiver as otherwise it will set the field on the parent object,
                                // currently we have issues with the borrow checker so using self for now.
                                // This will be required when we fully support the prototype chain
                                return self.define_own_property(&*property_key, value_desc);
                            }
                        }
                    },

                    _ => { unimplemented!() }
                }
            },
            _ => { unimplemented!() }
        }
    }

    // https://tc39.es/ecma262/#sec-ordinarysetwithowndescriptor
    fn ordinary_set_with_own_descriptor(&mut self, property_key: Rc<PropertyKey>, value: Rc<RefCell<JSValue>>, receiver: &Rc<RefCell<JSObject>>, own_descriptor: &PropertyDescriptorType) -> CompletionRecord {
        // 1. If ownDesc is undefined, then
        match own_descriptor {
            PropertyDescriptorType::Undefined(_) => {
                // a. Let parent be ? O.[[GetPrototypeOf]](). TODO
                // b. If parent is not null, then TODO
                //   i. Return ? parent.[[Set]](P, V, Receiver). TODO
                // c. Else,
                //   i. Set ownDesc to the PropertyDescriptor { [[Value]]: undefined, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
                let own_desc = PropertyDescriptorType::PropertyDescriptor(PropertyDescriptor { property: Some(PropertyType::DataProperty(DataProperty { value: Rc::new(RefCell::new(JSValue::Undefined)), writable: true, enumerable: true, configurable: true })) });
                return self.define_own_property(&*property_key, PropertyDescriptor { property: Some(PropertyType::DataProperty(DataProperty { value: Rc::clone(&value), writable: true, enumerable: true, configurable: true })) });

            },
            PropertyDescriptorType::PropertyDescriptor(property_descriptor) => {
                match &property_descriptor.property {
                    // 2. If IsDataDescriptor(ownDesc) is true, then
                    Some(PropertyType::DataProperty(data_property)) => {
                        // a. If ownDesc.[[Writable]] is false, return false.
                        if !data_property.writable {
                            return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                        }

                        // b. If Receiver is not an Object, return false.
                        // TODO: receiver param is always an JSObject, but should be of type Ecmascript Language Value (JSValue)

                        // c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
                        let existing_descriptor = self.get_own_property(&*property_key);

                        match &*existing_descriptor.value {
                            ReferenceRecordOrJsValue::PropertyDescriptor(property_descriptor_type) => {
                                match property_descriptor_type {
                                    // d. If existingDescriptor is not undefined, then
                                    PropertyDescriptorType::PropertyDescriptor(property_descriptor) => {
                                        match &property_descriptor.property {
                                            Some(PropertyType::AccessorProperty(_)) => {
                                                // i. If IsAccessorDescriptor(existingDescriptor) is true, return false.
                                                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                                            },
                                            Some(PropertyType::DataProperty(data_property)) => {
                                                // ii. If existingDescriptor.[[Writable]] is false, return false.
                                                if !data_property.writable {
                                                    return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                                                }
                                                //               iii. Let valueDesc be the PropertyDescriptor { [[Value]]: V }.
                                                // TODO Need to find a way to make the fields writable,enumerable etc configurable and not just initialize them anyway
                                                let value_desc = PropertyDescriptor { property: Some(PropertyType::DataProperty(DataProperty { value: Rc::clone(&value), writable: true, enumerable: data_property.enumerable, configurable: data_property.configurable })) };
                                                //               iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).

                                                // TODO: We need to call DefineOwnProperty on the receiver as otherwise it will set the field on the parent object,
                                                // currently we have issues with the borrow checker so using self for now.
                                                // This will be required when we fully support the prototype chain
                                                return self.define_own_property(&*property_key, value_desc);
                                            },

                                            _ => {
                                                unimplemented!();
                                            }
                                        }
                                    },
                                    // e. Else,
                                    PropertyDescriptorType::Undefined(_) => {
                                        // i. Assert: Receiver does not currently have a property P.
                                        // ii. Return ? CreateDataProperty(Receiver, P, V). TODO Implement CreateDataProperty
                                        let value_desc = PropertyDescriptor { property: Some(PropertyType::DataProperty(DataProperty { value: Rc::clone(&value), writable: true, enumerable: data_property.enumerable, configurable: data_property.configurable })) };

                                        // TODO: We need to call DefineOwnProperty on the receiver as otherwise it will set the field on the parent object,
                                        // currently we have issues with the borrow checker so using self for now.
                                        // This will be required when we fully support the prototype chain
                                        return self.define_own_property(&*property_key, value_desc);
                                    }
                                }
                            },

                            _ => { unimplemented!() }
                        }
                    },
                    _ => { unimplemented!() }
                }
            }
        }


        // 3. Assert: IsAccessorDescriptor(ownDesc) is true.
        // 4. Let setter be ownDesc.[[Set]].
        // 5. If setter is undefined, return false.
        // 6. Perform ? Call(setter, Receiver, « V »).
        // 7. Return true.
    }

    // https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
    fn define_own_property(&mut self, property_key: &PropertyKey, property_descriptor: PropertyDescriptor) -> CompletionRecord {
        return self.ordinary_define_own_property(property_key, property_descriptor);
    }

    // https://tc39.es/ecma262/#sec-ordinarydefineownproperty
    fn ordinary_define_own_property(&mut self, property_key: &PropertyKey, property_descriptor: PropertyDescriptor) -> CompletionRecord  {
        // Let current be ? O.[[GetOwnProperty]](P).
        let current = self.get_own_property(property_key);
        println!("CURRENT {:?}", current);
        // 2. Let extensible be ? IsExtensible(O). TODO

        // 3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).
        match current.value.deref() {
            ReferenceRecordOrJsValue::PropertyDescriptor(current_property_descriptor) => {
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(self.validate_and_apply_property_descriptor(property_key, true, property_descriptor, current_property_descriptor)))))))

            },
            _ => { unreachable!() }
        }
    }

    // https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
    fn validate_and_apply_property_descriptor(&mut self, property_key: &PropertyKey, extensible: bool, property_descriptor: PropertyDescriptor, current: &PropertyDescriptorType) -> bool {
        // Assert: P is a property key.
        // 2. If current is undefined, then
        match current {
            PropertyDescriptorType::Undefined(_) => {
                //        a. If extensible is false, return false.
                if !extensible {
                    return false;
                }
                //        b. If O is undefined, return true. TODO
                //        c. If IsAccessorDescriptor(Desc) is true, then
                match property_descriptor.property {
                    Some(PropertyType::AccessorProperty(data_property)) => {
                        //               i. Create an own accessor property named P of object O whose [[Get]], [[Set]], [[Enumerable]], and [[Configurable]] attributes
                        //                  are set to the value of the corresponding field in Desc if Desc has that field, or to the attribute's default value otherwise.
                        unimplemented!();
                    },
                    //        d. Else,
                    Some(PropertyType::DataProperty(data_prop)) => {
                        // i. Create an own data property named P of object O whose [[Value]], [[Writable]], [[Enumerable]], and [[Configurable]] attributes
                        // are set to the value of the corresponding field in Desc if Desc has that field, or to the attribute's default value otherwise.
                        let new_data_property = DataProperty { value: data_prop.value, writable: data_prop.writable, configurable: data_prop.configurable, enumerable: data_prop.enumerable };

                        match property_key {
                            PropertyKey::String(s) => {
                                self.values.insert(PropertyKey::String(s.clone()), Rc::new(PropertyType::DataProperty(new_data_property)));
                            },
                            _ => { unimplemented!() }
                        }
                        //        e. Return true.
                        return true;
                    },
                    None => { return false; }
                }




            },
            PropertyDescriptorType::PropertyDescriptor(property_descriptor) => {
                // 3. Assert: current is a fully populated Property Descriptor. TODO
                // 4. If Desc does not have any fields, return true. TODO
/*                match property_descriptor.property {
                    PropertyType::DataProperty(data_property) => {
                        // 5. If current.[[Configurable]] is false, then
                        if !data_property.configurable {
                            return create_normal_completion()
                        }
                    }
                }*/
                //
                //        a. If Desc has a [[Configurable]] field and Desc.[[Configurable]] is true, return false.
                //        b. If Desc has an [[Enumerable]] field and Desc.[[Enumerable]] is not current.[[Enumerable]], return false.
                //        c. If IsGenericDescriptor(Desc) is false and IsAccessorDescriptor(Desc) is not IsAccessorDescriptor(current), return false.
                //        d. If IsAccessorDescriptor(current) is true, then
                //               i. If Desc has a [[Get]] field and SameValue(Desc.[[Get]], current.[[Get]]) is false, return false.
                //               ii. If Desc has a [[Set]] field and SameValue(Desc.[[Set]], current.[[Set]]) is false, return false.
                //        e. Else if current.[[Writable]] is false, then
                //               i. If Desc has a [[Writable]] field and Desc.[[Writable]] is true, return false.
                //               ii. NOTE: SameValue returns true for NaN values which may be distinguishable by other means. Returning here ensures that any existing property of O remains unmodified.
                //               iii. If Desc has a [[Value]] field, return SameValue(Desc.[[Value]], current.[[Value]]).
                //
                // 6. If O is not undefined, then
                //
                //        a. If IsDataDescriptor(current) is true and IsAccessorDescriptor(Desc) is true, then
                //               i. If Desc has a [[Configurable]] field, let configurable be Desc.[[Configurable]]; else let configurable be current.[[Configurable]].
                //               ii. If Desc has a [[Enumerable]] field, let enumerable be Desc.[[Enumerable]]; else let enumerable be current.[[Enumerable]].
                //               iii. Replace the property named P of object O with an accessor property whose [[Configurable]] and [[Enumerable]] attributes
                //                    are set to configurable and enumerable, respectively, and whose [[Get]] and [[Set]] attributes are set to
                //                    the value of the corresponding field in Desc if Desc has that field, or to the attribute's default value otherwise.
                //        b. Else if IsAccessorDescriptor(current) is true and IsDataDescriptor(Desc) is true, then
                //               i. If Desc has a [[Configurable]] field, let configurable be Desc.[[Configurable]]; else let configurable be current.[[Configurable]].
                //               ii. If Desc has a [[Enumerable]] field, let enumerable be Desc.[[Enumerable]]; else let enumerable be current.[[Enumerable]].
                //               iii. Replace the property named P of object O with a data property whose [[Configurable]] and [[Enumerable]] attributes
                //                    are set to configurable and enumerable, respectively, and whose [[Value]] and [[Writable]] attributes are set to
                //                    the value of the corresponding field in Desc if Desc has that field, or to the attribute's default value otherwise.
                //        c. Else,
                //               i. For each field of Desc, set the corresponding attribute of the property named P of object O to the value of the field.
                //
                // 7. Return true.
                unimplemented!();
            }
        }


    }

    // https://tc39.es/ecma262/#sec-ordinaryget
    fn ordinary_get(&self, key: &PropertyKey, receiver: &Rc<RefCell<JSObject>>) -> Rc<RefCell<JSValue>> {
        // 1. Let desc be ? O.[[GetOwnProperty]](P).
        // https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
        let desc = self.ordinary_get_own_property(key);
        match desc {
            //     2. If desc is undefined, then
            PropertyDescriptorType::Undefined(_) => {
                // TODO: a. Let parent be ? O.[[GetPrototypeOf]](). We need to implement prototypes
                // let parent = &self;
                // TODO:    b. If parent is null, return undefined.

                //     c. Return ? parent.[[Get]](P, Receiver).
                return self.get(key, receiver);
            },
            PropertyDescriptorType::PropertyDescriptor(property_descriptor) => {
                match property_descriptor.property {
                    //     3. If IsDataDescriptor(desc) is true, return desc.[[Value]].
                    Some(PropertyType::DataProperty(data_property)) => {
                        return Rc::clone(&data_property.value);
                    },
                    //     4. Assert: IsAccessorDescriptor(desc) is true.
                    Some(PropertyType::AccessorProperty(accessor_property)) => {
                        //     5. Let getter be desc.[[Get]].
                        let getter = accessor_property.get;
                        //     6. If getter is undefined, return undefined.
                        if getter.is_none() {
                            return Rc::new(RefCell::new(JSValue::Undefined));
                        } else {
                            //     7. Return ? Call(getter, Receiver).
                            todo!();
                        }
                    },
                    None => unimplemented!()
                }
            }
        }
    }

    // https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    fn get_own_property(&self, key: &PropertyKey) -> CompletionRecord {
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::PropertyDescriptor(self.ordinary_get_own_property(key))));
    }

    // https://tc39.es/ecma262/#sec-ordinarygetownproperty
    fn ordinary_get_own_property(&self, key: &PropertyKey) -> PropertyDescriptorType {
        // 1. If O does not have an own property with key P, return undefined.
        if !self.values.contains_key(&key) {
            return PropertyDescriptorType::Undefined(JSValue::Undefined);
        }
        //     2. Let D be a newly created Property Descriptor with no fields.
        let mut property_descriptor: PropertyDescriptor = PropertyDescriptor { property: None };

        //     3. Let X be O's own property whose key is P.
        let property_data = self.values.get(&key).unwrap().clone();

        match &*property_data {
            //     4. If X is a data property, then
            PropertyType::DataProperty(ref data_prop ) => {
                // a. Set D.[[Value]] to the value of X's [[Value]] attribute.
                // b. Set D.[[Writable]] to the value of X's [[Writable]] attribute.
                property_descriptor.property = Some(PropertyType::DataProperty(DataProperty { value: Rc::clone(&data_prop.value), writable: data_prop.writable, enumerable: data_prop.enumerable, configurable: data_prop.configurable }));

                //     8. Return D.
                return PropertyDescriptorType::PropertyDescriptor(property_descriptor);
            },
            //     5. Else,
            // a. Assert: X is an accessor property.
            PropertyType::AccessorProperty(accessor_prop) => {
                //     b. Set D.[[Get]] to the value of X's [[Get]] attribute.
                //     c. Set D.[[Set]] to the value of X's [[Set]] attribute.
                //     8. Return D.
                unimplemented!();
            },
        }
        //     TODO: 6. Set D.[[Enumerable]] to the value of X's [[Enumerable]] attribute.
        //     TODO: 7. Set D.[[Configurable]] to the value of X's [[Configurable]] attribute.
    }

    // https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    fn has_property(&self, property_key: PropertyKey) -> CompletionRecord {
        return self.ordinary_has_property(property_key);
    }

    // https://tc39.es/ecma262/#sec-ordinaryhasproperty
    fn ordinary_has_property(&self, property_key: PropertyKey) -> CompletionRecord {
        // 1. Let hasOwn be ? O.[[GetOwnProperty]](P).
        let has_own = self.get_own_property(&property_key);
        // 2. If hasOwn is not undefined, return true.
        match has_own.value.deref() {
            ReferenceRecordOrJsValue::PropertyDescriptor(property_descriptor) => {
                match property_descriptor {
                    PropertyDescriptorType::PropertyDescriptor(_) => {
                        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(true))))));
                    },
                    _ => {
                        // 3. Let parent be ? O.[[GetPrototypeOf]]().
                        // 4. If parent is not null, then
                        //
                        //        a. Return ? parent.[[HasProperty]](P).
                        //
                    }
                }
            },
            _ => {}
        }
        // 5. Return false.
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
    }
}

impl Callable for JSObject {
}

// https://tc39.es/ecma262/#sec-ecmascript-language-types-number-type
// TODO: Support BigInt https://tc39.es/ecma262/#sec-ecmascript-language-types-bigint-type
type Number = f64;

// https://tc39.es/ecma262/#sec-ecmascript-language-types
#[derive(Debug)]
enum JSValue {
    Undefined,
    Boolean(bool),
    String(String),
    Symbol(JSSymbol),
    Numeric(Number),
    Object(JSObject),
    Null
}


#[derive(Debug)]
enum EnvironmentRecordType {
    DeclarativeEnvironmentRecord(Rc<RefCell<DeclarativeEnvironmentRecord>>),
    FunctionEnvironmentRecord(FunctionEnvironmentRecord),
    ObjectEnvironmentRecord(Rc<RefCell<ObjectEnvironmentRecord>>),
    GlobalEnvironmentRecord(Rc<RefCell<GlobalEnvironmentRecord>>),
}

fn create_normal_completion(value: Rc<ReferenceRecordOrJsValue>) -> CompletionRecord {
    return CompletionRecord {
        type_: CompletionRecordType::Normal,
        value: Rc::clone(&value),
        target: None
    }
}

fn create_throw_completion(value: Rc<ReferenceRecordOrJsValue>) -> CompletionRecord {
    return CompletionRecord {
        type_: CompletionRecordType::Throw,
        value: Rc::clone(&value),
        target: None
    }
}

// https://tc39.es/ecma262/#sec-declarative-environment-records
impl DeclarativeEnvironmentRecord {

    // https://tc39.es/ecma262/#sec-declarative-environment-records-hasbinding-n
    fn has_binding(&self, binding_id: String) -> CompletionRecord {
        // If envRec has a binding for N, return true.
        if self.variable_bindings.contains_key(&binding_id) {
            return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(true))))));
        } else {
            // 2. Return false.
            return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));;
        }
    }
    // tc39.es/ecma262/#sec-declarative-environment-records-setmutablebinding-n-v-s
    pub fn set_mutable_binding(&mut self, binding_id: String, value: Rc<RefCell<JSValue>>, strict: bool) -> CompletionRecord {
        // 1. If envRec does not have a binding for N, then
        if !self.variable_bindings.contains_key(&binding_id) {
            // a. If S is true, throw a ReferenceError exception.
            if strict {
                // FIXME: The value here in throw and everywhere else we throw should be a Error object and and not just the base object
                return create_throw_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Object(JSObject { values: HashMap::new(), prototype: None, extensible: false }))))));
            } else {
                //     b. Perform ! envRec.CreateMutableBinding(N, true).
                self.create_mutable_binding(binding_id.clone(), strict);

                //     c. Perform ! envRec.InitializeBinding(N, V).
                self.initialize_binding(binding_id, value);

                //     d. Return unused.
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))));
            }
        }

        //     2. If the binding for N in envRec is a strict binding, set S to true. TODO: This occurs in CreateImmutableBinding which is not implemented yet
        // Check the initialization state before trying to insert
        let should_insert;
        let initialized;

        match self.variable_bindings.get(&binding_id) {
            Some(binding_ref) => {
                match binding_ref {
                    Binding::MutableBinding(mut_binding) => {
                        match &*mut_binding.borrow() {
                            //     3. If the binding for N in envRec has not yet been initialized, then
                            JSValue::Undefined => {
                                // a. Throw a ReferenceError exception.
                                // FIXME: The value here in throw and everywhere else we throw should be a Error object and and not just the base object
                                return create_throw_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Object(JSObject { values: HashMap::new(), prototype: None, extensible: false }))))));
                            },
                            _ => {
                                //     4. Else if the binding for N in envRec is a mutable binding, then
                                // a. Change its bound value to V.
                                should_insert = true;
                                initialized = true;
                            }
                        }
                    },
                    //     5. Else,
                    // a. Assert: This is an attempt to change the value of an immutable binding.
                   Binding::ImmutableBinding(_) => {
                       //     b. If S is true, throw a TypeError exception.
                       if strict {
                           // FIXME: The value here in throw and everywhere else we throw should be a Error object and and not just the base object
                           return create_throw_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Object(JSObject { values: HashMap::new(), prototype: None, extensible: false }))))));
                       }
                       should_insert = false;
                       initialized = false;
                   }
                }
            },
            None => {
                should_insert = false;
                initialized = false;
            }
        }

        // Now perform the insertion if needed
        if should_insert && initialized {
            let new_binding = Binding::MutableBinding(value);
            self.variable_bindings.insert(binding_id.to_string(), new_binding);
        }
        //     6. Return unused.
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))));
    }

    // https://tc39.es/ecma262/#sec-declarative-environment-records-setmutablebinding-n-v-s
    fn create_mutable_binding(&mut self, binding_id: String, marked_for_deletion: bool) -> CompletionRecord {
        // 1. Assert: envRec does not already have a binding for N.
        if !self.variable_bindings.contains_key(&binding_id) {
            // 2. Create a mutable binding in envRec for N and record that it is uninitialized (Setting value of mut binding to Undefined which means uninitialized)
            // TODO: If D is true, record that the newly created binding may be deleted by a subsequent DeleteBinding call.
            let new_mutable_binding: Binding = Binding::MutableBinding(Rc::new(RefCell::new(JSValue::Undefined)));
            self.variable_bindings.insert(binding_id, new_mutable_binding);
        }

        // 3. Return unused.
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))));
    }

    // https://tc39.es/ecma262/#sec-declarative-environment-records-initializebinding-n-v
    fn initialize_binding(&mut self, binding_id: String, value: Rc<RefCell<JSValue>>) -> CompletionRecord {
        match self.variable_bindings.get(&binding_id) {
            // 1. Assert: envRec must have an uninitialized binding for N.
            Some(binding) => {
                match &binding {
                    Binding::MutableBinding(mut_binding) => {
                        // 2. Set the bound value for N in envRec to V.
                        // 3. Record that the binding for N in envRec has been initialized. (Presence here determines if it is initialized)
                        let new_binding = Binding::MutableBinding(value);
                        self.variable_bindings.insert(binding_id.to_string(), new_binding);
                    }
                    Binding::ImmutableBinding(_) => {
                        // 2. Set the bound value for N in envRec to V.
                        // 3. Record that the binding for N in envRec has been initialized. (Presense here determine if it is initialized)
                        unreachable!("[js::initialize_binding] Trying to initialize an immutable binding after creation!")
                    }
                }

            }
            None => { unreachable!() }
        }
        // 4. Return unused.
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))));
    }

    // https://tc39.es/ecma262/#sec-declarative-environment-records-getbindingvalue-n-s
    fn get_binding_value(&self, binding_id: String, is_strict: bool) -> CompletionRecord {
        // 1. Assert: envRec has a binding for N.
        if self.variable_bindings.contains_key(&binding_id) {
            // 2. If the binding for N in envRec is an uninitialized binding, throw a ReferenceError exception.
            if self.variable_bindings.get(&binding_id).is_none() {
                // FIXME: value should of a ReferenceError JS object
                return CompletionRecord { type_: CompletionRecordType::Throw, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Object(JSObject { values: HashMap::new(), prototype: None, extensible: false }))))), target: None }
            }
        }

        // 3. Return the value currently bound to N in envRec.
        let binding = self.variable_bindings.get(&binding_id).unwrap();
        match binding {
            Binding::MutableBinding(js_value) => {
                return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::clone(js_value))), target: None }
            },
            // Binding::ImmutableBinding(js_value) => {
            //     return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::clone(js_value))), target: None }
            // }
            _ => { todo!("Implement returning immutable binding value") }
        }
    }
}

#[derive(Debug)]
struct EnvironmentRecord {
    environment_record_type: EnvironmentRecordType,
    outer_environment_record: Option<Rc<RefCell<EnvironmentRecord>>>,
}

impl ObjectEnvironmentRecord {

    // https://tc39.es/ecma262/#sec-object-environment-records-getbindingvalue-n-s
    fn get_binding_value(&self, binding_id: String, is_strict: bool) -> CompletionRecord {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        let bindingObject = &self.binding_object;

        //     2. Let value be ? HasProperty(bindingObject, N).
        //      https://tc39.es/ecma262/#sec-hasproperty
        let value = bindingObject.borrow().values.contains_key(&PropertyKey::String(binding_id.clone()));

        //     3. If value is false, then
        if !value {
            todo!()
            // a. If S is false, return undefined; otherwise throw a ReferenceError exception.
        } else {
            //     4. Return ? Get(bindingObject, N).
            // https://tc39.es/ecma262/#sec-get-o-p
            return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(bindingObject.borrow().get(&*Rc::new(PropertyKey::String(binding_id)), bindingObject))), target: None }
        }
    }

    // https://tc39.es/ecma262/#sec-object-environment-records-hasbinding-n
    fn has_binding(&self, binding_name: String) -> CompletionRecord {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        let bindingObject = &self.binding_object;
        // 2. Let foundBinding be ? HasProperty(bindingObject, N).
        // TODO: PropertyKey could also be a Symbol?
        let found_binding = ObjectEnvironmentRecord::has_property(bindingObject, PropertyKey::String(binding_name.clone()));
        // 3. If foundBinding is false, return false.
        match found_binding.value.deref() {
            ReferenceRecordOrJsValue::JSValue(js_value) => {
                match js_value.borrow().deref() {
                    JSValue::Boolean(bool_value) => {
                        if !bool_value {
                            return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                        } else {
                            // 4. If envRec.[[IsWithEnvironment]] is false, return true.
                            // 5. Let unscopables be ? Get(bindingObject, %Symbol.unscopables%).
                            // 6. If unscopables is an Object, then
                            //
                            //        a. Let blocked be ToBoolean(? Get(unscopables, N)).
                            //        b. If blocked is true, return false.
                            //
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        // 7. Return true.
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(true))))));
    }

    // https://tc39.es/ecma262/#sec-hasproperty
    fn has_property(object: &Rc<RefCell<JSObject>>, property_key: PropertyKey) -> CompletionRecord {
        return object.borrow().has_property(property_key);
    }
}

impl GlobalEnvironmentRecord {
    // https://tc39.es/ecma262/#sec-global-environment-records-getbindingvalue-n-s
    fn get_binding_value(&self, binding_id: String, is_strict: bool) -> CompletionRecord {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        let declarative_record = &self.declarative_environment_record;
        // 2. If ! DclRec.HasBinding(N) is true, then
        match declarative_record.borrow().has_binding(binding_id.clone()).value.deref() {
            ReferenceRecordOrJsValue::JSValue(js_value) => {
                match js_value.borrow().deref() {
                    JSValue::Boolean(bool_value) => {
                        if *bool_value {
                            //        a. Return ? DclRec.GetBindingValue(N, S).
                            return declarative_record.borrow().get_binding_value(binding_id, is_strict);
                        } else {
                            // 3. Let ObjRec be envRec.[[ObjectRecord]].
                            let object_record = &self.object_environment_record;
                            // 4. Return ? ObjRec.GetBindingValue(N, S).
                            return object_record.clone().unwrap().borrow().get_binding_value(binding_id, is_strict);
                        }
                    },
                    _ => { unreachable!() }
                }
            },
            _ => { unreachable!() }
        }

    }
}
impl EnvironmentRecord {
    pub fn new(type_: EnvironmentRecordType) -> EnvironmentRecord {
        EnvironmentRecord {
            environment_record_type: type_,
            outer_environment_record: None, // TODO: Fine for the global environment as it has no outer env but will need to be set
        }
    }

    // https://tc39.es/ecma262/#table-abstract-methods-of-environment-records
    fn has_binding(&self, binding_name: String) -> CompletionRecord {
        match &self.environment_record_type {
            // TODO: https://tc39.es/ecma262/#sec-global-environment-records-hasbinding-n
            EnvironmentRecordType::GlobalEnvironmentRecord(global_environment_record) => {
                // 1. Let DclRec be envRec.[[DeclarativeRecord]].
                let declarative_record = &global_environment_record.borrow().declarative_environment_record;
                // TODO: 2. If ! DclRec.HasBinding(N) is true, return true.

                // 3. Let ObjRec be envRec.[[ObjectRecord]].
                let object_record = &global_environment_record.borrow().object_environment_record;
                // 4. Return ? ObjRec.HasBinding(N).
                return object_record.clone().unwrap().borrow().has_binding(binding_name);
            },
            _ => { todo!("has_binding: Support other environment record types") }
        }
    }
}

type MutableBinding = Rc<RefCell<JSValue>>;
type ImmutableBinding = Rc<JSValue>;

#[derive(Debug)]
enum Binding {
    MutableBinding(MutableBinding),
    ImmutableBinding(ImmutableBinding)
}

#[derive(Debug)]
struct DeclarativeEnvironmentRecord {
    // TODO: Should not be of an option type
    function_environment_record: Option<FunctionEnvironmentRecord>,
    variable_bindings: HashMap<String, Binding>,
}

#[derive(Debug)]
enum ThisBindingStatus {
    Lexical,
    Uninitialized,
    Initialized,
}

#[derive(Debug)]
// https://tc39.es/ecma262/#table-additional-fields-of-function-environment-records
struct FunctionEnvironmentRecord {
    this_value: Box<JSValue>,
    this_binding_status: ThisBindingStatus,
    function_object: JSObject,
    new_target: Option<JSObject>,
}

#[derive(Debug)]
struct ObjectEnvironmentRecord {
    binding_object: Rc<RefCell<JSObject>>,
    is_with_environment: bool,
}

#[derive(Debug)]
// https://tc39.es/ecma262/#table-additional-fields-of-global-environment-records
struct GlobalEnvironmentRecord {
    object_environment_record: Option<Rc<RefCell<ObjectEnvironmentRecord>>>,
    global_this_value: Option<Box<JSObject>>,
    declarative_environment_record: RefCell<DeclarativeEnvironmentRecord>
}

macro_rules! completion {
    ($expr:expr) => {
        match $expr.type_ {
            CompletionRecordType::Normal => {
                create_normal_completion($expr.value)
            },
            CompletionRecordType::Throw => return $expr,
            _ => unimplemented!()
        }
    };
}


impl AstVisitor<CompletionRecord> for Interpreter {
    fn visit_expression_statement(&mut self, expression: &ExpressionStatement) -> CompletionRecord {
        return self.evaluate(expression);
    }

    // https://tc39.es/ecma262/#sec-evaluatestringornumericbinaryexpression
    fn visit_binary(&mut self, expression: &BinaryExpression) -> CompletionRecord {
        // 1. Let lRef be ? Evaluation of leftOperand.
        let left_expression = completion!(self.evaluate(&*expression.left));

        // 2. Let lVal be ? GetValue(lRef).
        let left_value =  completion!(Interpreter::get_value(left_expression.value.clone()));

        // 3. Let rRef be ? Evaluation of rightOperand.
        let right_expression = completion!(self.evaluate(&*expression.right));

        // 4. Let rVal be ? GetValue(rRef).
        let right_value = completion!(Interpreter::get_value(right_expression.value.clone()));

        match (&*left_value.value, &*right_value.value) {
            (ReferenceRecordOrJsValue::JSValue(l_value), ReferenceRecordOrJsValue::JSValue(r_value)) => {
                let value = Interpreter::apply_string_or_numeric_binary_operator(l_value.clone(), r_value.clone(), &expression.operator.token_type);
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(value)));
            }
            _ => { unreachable!() }
        }

    }

    // https://tc39.es/ecma262/#sec-literals-runtime-semantics-evaluation
    fn visit_literal(&mut self, expression: &LiteralExpression) -> CompletionRecord {
        match &expression.value {
            Literal::String(value) => {
                let js_value = JSValue::String(value.to_string());
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(js_value)))));
            }
            Literal::Numeric(value) => {
                let js_value = JSValue::Numeric(*value as Number);
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(js_value)))));
            }
            Literal::Boolean(value) => {
                let js_value = JSValue::Boolean(*value);
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(js_value)))));
            }
            Literal::Null() => {
                let js_value = JSValue::Null;
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(js_value)))));
            }

        }
    }

    fn visit_parenthesized(&mut self, expression: &ParenthesizedExpression) -> CompletionRecord {
        return self.evaluate(&*expression.expression);
    }

    // https://tc39.es/ecma262/#prod-UnaryExpression
    fn visit_unary(&mut self, expression: &UnaryExpression) -> CompletionRecord {
        // 1. Let expr be ? Evaluation of UnaryExpression.
        let right = self.evaluate(&expression.right);

        match expression.operator.token_type {
            // httÓps://tc39.es/ecma262/#sec-unary-plus-operator-runtime-semantics-evaluation
            TokenType::PLUS => {
                // 2. Return ? ToNumber(? GetValue(expr)).
                match right.value.deref() {
                    ReferenceRecordOrJsValue::JSValue(value) => {
                        return Interpreter::to_number(value.clone());
                    },
                    _ => { unreachable!("TODO: We should handle passing in a JSValue from a ReferenceRecord as well") }
                }
            },
            // https://tc39.es/ecma262/#sec-unary-minus-operator-runtime-semantics-evaluation
            TokenType::MINUS => {
                // 2. Let oldValue be ? ToNumeric(? GetValue(expr)).
                let right_value_js = Interpreter::get_value(right.value);
                let old_value: Rc<RefCell<JSValue>> = match right_value_js.value.deref() {
                    ReferenceRecordOrJsValue::JSValue(value) => {
                        Interpreter::to_numeric(value.clone())
                    },
                    _ => { unreachable!("TODO: We should handle passing in a JSValue from a ReferenceRecord as well") }
                };

                // 3. If oldValue is a Number, then
                let borrowed_value = old_value.borrow();
                match borrowed_value.deref() {
                    JSValue::Numeric(value) => {
                        //a. TODO: Return Number::unaryMinus(oldValue).
                        // https://tc39.es/ecma262/#sec-numeric-types-number-unaryMinus
                        // Currently we just return the negative value and don't check for NaN.
                        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Numeric(-value))))));
                    },
                    // 4. Else
                    _ => {
                        // a. Assert: oldValue is a BigInt.
                        // b. Return BigInt::unaryMinus(oldValue).
                        todo!()
                    }
                }
            },
            // https://tc39.es/ecma262/#sec-bitwise-not-operator-runtime-semantics-evaluation
            TokenType::BITWISE_NOT => {
                todo!();
            },
            // https://tc39.es/ecma262/#sec-logical-not-operator-runtime-semantics-evaluation
            TokenType::BANG => {
                // 2. Let oldValue be ToBoolean(? GetValue(expr)).
                let old_value: Rc<RefCell<JSValue>> = match right.value.deref() {
                    ReferenceRecordOrJsValue::JSValue(value) => {
                        Interpreter::to_boolean(value.clone())
                    },
                    _ => { unreachable!("TODO: We should handle passing in a JSValue from a ReferenceRecord as well") }
                };

                match old_value.borrow().deref() {
                    // 3. If oldValue is true, return false.
                    JSValue::Boolean(true) => {
                        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(false))))));
                    },
                    // 4. Return true.
                    _ => {
                        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Boolean(true))))));
                    }
                };
            }
            _ => { todo!() }
        }

        // https://tc39.es/ecma262/#sec-unary-plus-operator-runtime-semantics-evaluation

        // https://tc39.es/ecma262/#sec-unary-minus-operator-runtime-semantics-evaluation

        // TODO: https://tc39.es/ecma262/#sec-bitwise-not-operator

        // https://tc39.es/ecma262/#sec-logical-not-operator


    }

    //https://tc39.es/ecma262/#sec-variable-statement-runtime-semantics-evaluation
    fn visit_variable_declaration(&mut self, expression: &VariableDeclarationStatement) -> CompletionRecord {
        //1. Let bindingId be the StringValue of BindingIdentifier. TODO: Not to spec
        let binding_id = expression.binding_identifier.lexeme.clone();

        // 2. Let lhs be ? ResolveBinding(bindingId).
        let left_hand_side = self.resolve_binding(binding_id, None);

        // 3. TODO: If IsAnonymousFunctionDefinition(Initializer) is true, then

        // 4. Else
        // a. Let rhs be ? Evaluation of Initializer.
        let right_hand_side = match &expression.initializer {
             Some(initializer) => self.evaluate(
                 &ExpressionStatement::AssignmentExpression(Box::new(AssignmentExpression { expression: Rc::clone(&initializer.expression), left_hand_side_expression: initializer.left_hand_side_expression.clone() }))
             ),
             None => {
                 // Not sure if returning undefined is correct here but if the variable has no iniliazer then just set to undefined
                 return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))), target: None };
             },
         };

        // b. Let value be ? GetValue(rhs).
        let value = Interpreter::get_value(right_hand_side.value);

        // 5. Perform ? PutValue(lhs, value).
        match &*value.value {
            ReferenceRecordOrJsValue::JSValue(value) => {
                self.put_value(left_hand_side.value, value.clone());
            },
            _ => { }
        }

        //print the global obj

        println!("\nGlobal Object {:?}\n", self.global_object());

        // Return empty.
        return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))), target: None };
    }

    // https://tc39.es/ecma262/#sec-identifiers-runtime-semantics-evaluation
    fn visit_identifier_expression(&mut self, expression: &IdentifierExpression) -> CompletionRecord {
        return self.resolve_binding(expression.binding_identifier.lexeme.clone(), None);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression) -> CompletionRecord {
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))))
    }

    // https://tc39.es/ecma262/#sec-block-runtime-semantics-evaluation
    fn visit_block_statement(&mut self, expression: &BlockStatement) -> CompletionRecord {
       // TODO: Ensure the correct environment record is used and scoped to the block
        let mut value: CompletionRecord = CompletionRecord {
            type_: CompletionRecordType::Normal,
            value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))),
            target: None,
        };

        for statement in expression.statements.iter() {
            value = self.execute(statement);
        }

        // The value of a StatementList is the value of the last value-producing item in the StatementList.
        return value; // TODO: Remove
    }

    // https://tc39.es/ecma262/#sec-object-initializer-runtime-semantics-evaluation
    fn visit_object_literal_expression(&mut self, object_literal_expression: &ObjectLiteralExpression) -> CompletionRecord {
        return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))));
    }

    // https://tc39.es/ecma262/#sec-assignment-operators-runtime-semantics-evaluation
    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression) -> CompletionRecord {

        // 1. If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral, then
        match &*expression.expression {
            ExpressionStatement::ObjectLiteralExpression(_) => { unimplemented!() },
            _ => {
                // a. Let lRef be ? Evaluation of LeftHandSideExpression.
                let left_reference = self.evaluate(&*expression.left_hand_side_expression);
                println!("Left Hand Side Expression: {:?}\n", self.evaluate(&*expression.left_hand_side_expression));

                //        b. If IsAnonymousFunctionDefinition(AssignmentExpression) is true and IsIdentifierRef of LeftHandSideExpression is true, then TODO
                //               i. Let lhs be the StringValue of LeftHandSideExpression.
                //               ii. Let rVal be ? NamedEvaluation of AssignmentExpression with argument lhs.
                //        c. Else,
                //               i. Let rRef be ? Evaluation of AssignmentExpression.
                let right_reference = self.evaluate(&*expression.expression);
                //               ii. Let rVal be ? GetValue(rRef).
                let right_value = Interpreter::get_value(right_reference.value);
                //        d. Perform ? PutValue(lRef, rVal).
                match right_value.value.deref() {
                    ReferenceRecordOrJsValue::JSValue(value) => {
                        self.put_value(left_reference.value, value.clone());
                        //        e. Return rVal.
                        return right_value;
                    },
                    _ => { unreachable!() }
                }
            }
        }

        // 2. Let assignmentPattern be the AssignmentPattern that is covered by LeftHandSideExpression.
        // 3. Let rRef be ? Evaluation of AssignmentExpression.
        // 4. Let rVal be ? GetValue(rRef).
        // 5. Perform ? DestructuringAssignmentEvaluation of assignmentPattern with argument rVal.
        // 6. Return rVal.
    }
}

#[derive(Debug)]
enum BaseValue {
    JSValue(Box<JSValue>),
    EnvironmentRecord(Rc<RefCell<EnvironmentRecord>>),
    Unresolvable
}

// https://tc39.es/ecma262/#sec-reference-record-specification-type
#[derive(Debug)]
struct ReferenceRecord {
    // https://tc39.es/ecma262/#table-reference-record-fields
    base: Rc<BaseValue>,
    referenced_name: JSValue, // TODO: Or a Private Name
    strict: bool,
    this_value: Option<Box<JSValue>>,
}

#[derive(Debug)]
enum ReferenceRecordOrJsValue {
    ReferenceRecord(ReferenceRecord),
    JSValue(Rc<RefCell<JSValue>>),
    PropertyDescriptor(PropertyDescriptorType),
}

#[derive(Debug)]
enum CompletionRecordType {
    Normal,
    Break,
    Continue,
    Return,
    Throw
}

// https://tc39.es/ecma262/#sec-completion-record-specification-type
#[derive(Debug)]
struct CompletionRecord {
    // https://tc39.es/ecma262/#table-completion-record-fields
    type_: CompletionRecordType,
    value: Rc<ReferenceRecordOrJsValue>,
    target: Option<String>
}

#[derive(PartialEq)]
enum ObjectInternalSlot {
    Prototype,
    Extensible,
    PrivateElements
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { had_error: false,
            execution_contexts: vec![
                ExecutionContext {
                    lexical_environment_record: Rc::new(RefCell::new(EnvironmentRecord::new(EnvironmentRecordType::GlobalEnvironmentRecord(Rc::new(RefCell::new(GlobalEnvironmentRecord {
                        global_this_value: None, // Should not be none, temporary
                        object_environment_record: Option::from(Rc::new(RefCell::new(ObjectEnvironmentRecord { binding_object: Rc::new(RefCell::new(JSObject {
                            values: HashMap::new(),
                            prototype: None,
                            extensible: false,
                        })), is_with_environment: false }))), // Should not be none, temporary
                        declarative_environment_record: RefCell::new(DeclarativeEnvironmentRecord { variable_bindings: HashMap::new(), function_environment_record: None })
                    })))))),
                    variable_environment_record: Rc::new(RefCell::new(EnvironmentRecord {
                        outer_environment_record: None,
                        environment_record_type: EnvironmentRecordType::DeclarativeEnvironmentRecord(
                            Rc::new(RefCell::new(DeclarativeEnvironmentRecord { variable_bindings: HashMap::new(), function_environment_record: None }))
                        )
                    })),
                }
            ]
        }
    }
    // https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    fn ordinary_object_create(&mut self, proto: Option<JSObject>, mut additional_internal_slots: Vec<ObjectInternalSlot>) -> JSObject {
        // 1. Let internalSlotsList be « [[Prototype]], [[Extensible]] ».
        let mut internal_slots = vec![ObjectInternalSlot::Prototype, ObjectInternalSlot::Extensible];

        // 2. If additionalInternalSlotsList is present, set internalSlotsList to the list-concatenation of internalSlotsList and additionalInternalSlotsList.
        if additional_internal_slots.len() > 0 {
            internal_slots.append(&mut additional_internal_slots);
        }

        // 3. Let O be MakeBasicObject(internalSlotsList).
        let mut object = self.make_basic_object(internal_slots);

        // 4. Set O.[[Prototype]] to proto.
        object.prototype = Some(Rc::new(proto.unwrap()));

        // 5. Return O.
        return object;
    }

    // https://tc39.es/ecma262/#sec-set-o-p-v-throw
    pub fn set(object: &Rc<RefCell<JSObject>>, key: Rc<PropertyKey>, value: Rc<RefCell<JSValue>>, throw: bool) -> CompletionRecord {
        // 1. Let success be ? O.[[Set]](P, V, O).
                let success = object.borrow_mut().set(key, value, object);
                // 2. If success is false and Throw is true, throw a TypeError exception. TODO
                // 3. Return unused.
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))));
    }

    fn make_basic_object(&self, mut internal_slots: Vec<ObjectInternalSlot>) -> JSObject {
        // 1. Set internalSlotsList to the list-concatenation of internalSlotsList and « [[PrivateElements]] ».
        internal_slots.push(ObjectInternalSlot::PrivateElements);

        let mut object = JSObject::new(); // Initializing here so we have a value to always return

        // 2. Let obj be a newly created object with an internal slot for each name in internalSlotsList.
        if internal_slots.contains(&ObjectInternalSlot::Prototype) && internal_slots.contains(&ObjectInternalSlot::Extensible) && internal_slots.contains(&ObjectInternalSlot::PrivateElements) {
            // 3. NOTE: As described in Object Internal Methods and Internal Slots, the initial value of each such internal slot is undefined unless specified otherwise.
            object = JSObject::new();

            // 4. Set obj.[[PrivateElements]] to a new empty List. TODO: Implement PrivateElements

            // 5. TODO: Set obj's essential internal methods to the default ordinary object definitions specified in 10.1.

            // 6. TODO: Assert: If the caller will not be overriding both obj's [[GetPrototypeOf]] and [[SetPrototypeOf]] essential internal methods, then internalSlotsList contains [[Prototype]].
            // 7. TODO: Assert: If the caller will not be overriding all of obj's [[SetPrototypeOf]], [[IsExtensible]], and [[PreventExtensions]] essential internal methods, then internalSlotsList contains [[Extensible]].

            // 8. If internalSlotsList contains [[Extensible]], set obj.[[Extensible]] to true.
            if internal_slots.contains(&ObjectInternalSlot::Extensible) {
                object.extensible = true;
            }
        }

        // 9. Return obj.
        return object;
    }

    // https://tc39.es/ecma262/#running-execution-context
    fn running_execution_context(&self) -> &ExecutionContext {
        return &self.execution_contexts[self.execution_contexts.len() - 1];
    }

    fn global_object(&self) -> Rc<RefCell<JSObject>> {
        match &self.running_execution_context().lexical_environment_record.borrow().environment_record_type {
            EnvironmentRecordType::GlobalEnvironmentRecord(record) => {
                return record.borrow_mut().object_environment_record.clone().unwrap().borrow_mut().binding_object.clone();
            },
            _ => { unreachable!() }
        }
    }
    // https://tc39.es/ecma262/#sec-putvalue
    fn put_value(&mut self, binding_identifier: Rc<ReferenceRecordOrJsValue>, value: Rc<RefCell<JSValue>>) -> CompletionRecord {
        match &*binding_identifier {
            // 1. If V is not a Reference Record, throw a ReferenceError exception.
            ReferenceRecordOrJsValue::JSValue(_) => {
                // FIXME: The value here in throw and everywhere else we throw should be a Error object and and not just the base object
                return CompletionRecord { type_: CompletionRecordType::Throw, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Object(JSObject { values: HashMap::new(), prototype: None, extensible: false }))))), target: None }
            }
            ReferenceRecordOrJsValue::ReferenceRecord(reference_record) => {
                //     2. If IsUnresolvableReference(V) is true, throw a ReferenceError exception.
                match reference_record.base.as_ref() {
                    BaseValue::Unresolvable => {
                        // TODO: If V.[[Strict]] is true, throw a ReferenceError exception.
                        //     b. Let globalObj be GetGlobalObject().
                        let global_object = self.global_object();

                        //  c. Perform ? Set(globalObj, V.[[ReferencedName]], W, false).
                        // This is what the Set (O, P, V, Throw) Method is. FIXME: Implement Set as own method
                        // https://tc39.es/ecma262/#sec-set-o-p-v-throw
                        match &reference_record.referenced_name {
                            JSValue::String(referenced_name) => {
                                Interpreter::set(&global_object, Rc::new(PropertyKey::String(referenced_name.clone())), value, false);
                            }
                            JSValue::Symbol(symbol_value) => {
                                todo!("Support Symbols Properly")
                                // global_object.borrow_mut().values.insert(PropertyKey::Symbol(symbol_value), Rc::new(PropertyType::DataProperty(DataProperty { value: Rc::new(value), writable: false })));
                            },
                            _ => { unreachable!() }
                        }
                        //     d. Return unused.
                        return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Undefined)))), target: None }
                    },
                    _ => {
                        // TODO: 3. If IsPropertyReference(V) is true, then
                        if Interpreter::is_property_reference(&reference_record) {

                                                        todo!();
                            // 1. Let baseObj be ? ToObject(V.[[Base]]).
                            //     b. If IsPrivateReference(V) is true, then
                            //
                            // i. Return ? PrivateSet(baseObj, V.[[ReferencedName]], W).
                            //
                            //     c. If V.[[ReferencedName]] is not a property key, then
                            //
                            // i. Set V.[[ReferencedName]] to ? ToPropertyKey(V.[[ReferencedName]]).
                            //
                            //     d. Let succeeded be ? baseObj.[[Set]](V.[[ReferencedName]], W, GetThisValue(V)).
                            //     e. If succeeded is false and V.[[Strict]] is true, throw a TypeError exception.
                            //     f. Return unused.
                        } else {
                            //4. Else,

                            // a. Let base be V.[[Base]].
                            let base = &reference_record.base;

                            // b. Assert: base is an Environment Record.
                            match &**base {
                                BaseValue::EnvironmentRecord(env_record) => {
                                    match &env_record.borrow().environment_record_type {
                                        EnvironmentRecordType::DeclarativeEnvironmentRecord(dec_record) => {
                                            match &reference_record.referenced_name {
                                                JSValue::String(referenced_name) => {
                                                    //c. Return ? base.SetMutableBinding(V.[[ReferencedName]], W, V.[[Strict]]) (see 9.1).
                                                    return dec_record.borrow_mut().set_mutable_binding(referenced_name.to_string(), value, false);
                                                },
                                                _ => { unreachable!() }
                                            }
                                        }
                                        _ => { unreachable!() }
                                    }
                                },
                                _ => { unreachable!() }
                            }


                        }
                    }
                }
            },
            _ => { unreachable!() }
        }
    }

    // https://tc39.es/ecma262/#sec-getvalue
    fn get_value(reference_record_or_js_value: Rc<ReferenceRecordOrJsValue>) -> CompletionRecord {
        match &*reference_record_or_js_value {
            // If V is not a Reference Record, return V.
            ReferenceRecordOrJsValue::JSValue(value) => {
                return CompletionRecord { type_: CompletionRecordType::Normal, value: Rc::new(ReferenceRecordOrJsValue::JSValue(value.clone())), target: None };
            },
            ReferenceRecordOrJsValue::ReferenceRecord(reference_record) => {
                //     2. If IsUnresolvableReference(V) is true, throw a ReferenceError exception.
                match reference_record.base.as_ref() {
                    BaseValue::Unresolvable => {
                        // FIXME: The value here in throw and everywhere else we throw should be a Error object and and not just the base object
                        return CompletionRecord { type_: CompletionRecordType::Throw, value: Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Object(JSObject { values: HashMap::new(), prototype: None, extensible: false }))))), target: None }
                    },

                    // 4. Else,
                    BaseValue::EnvironmentRecord(env_record) => {
                        // a. Let base be V.[[Base]].
                        let base = env_record;

                        // b. Assert: base is an Environment Record.
                        match &base.borrow().environment_record_type {
                            EnvironmentRecordType::DeclarativeEnvironmentRecord(dec_record) => {
                                // V.[[ReferencedName]]
                                // The name of the binding. Always a String if [[Base]] value is an Environment Record. Otherwise, may be an ECMAScript language value other than a String or a Symbol until ToPropertyKey is performed.
                                match &reference_record.referenced_name {
                                    JSValue::String(value) => {
                                        // c. Return ? base.GetBindingValue(V.[[ReferencedName]], V.[[Strict]]) (see 9.1).
                                        let binding_value = dec_record.borrow().get_binding_value(value.to_string(), false);
                                        return CompletionRecord { type_: CompletionRecordType::Normal, value: binding_value.value,  target: None }
                                    },
                                    _ => { unreachable!() }
                                }

                            },
                            EnvironmentRecordType::ObjectEnvironmentRecord(obj_record) => {
                                // V.[[ReferencedName]]
                                // The name of the binding. Always a String if [[Base]] value is an Environment Record. Otherwise, may be an ECMAScript language value other than a String or a Symbol until ToPropertyKey is performed.
                                match &reference_record.referenced_name {
                                    JSValue::String(value) => {
                                        // c. Return ? base.GetBindingValue(V.[[ReferencedName]], V.[[Strict]]) (see 9.1).
                                        let binding_value = obj_record.borrow().get_binding_value(value.to_string(), false);
                                        return CompletionRecord { type_: CompletionRecordType::Normal, value: binding_value.value,  target: None }
                                    },
                                    _ => { unreachable!() }
                                }
                            }
                            EnvironmentRecordType::GlobalEnvironmentRecord(global_record) => {
                                // V.[[ReferencedName]]
                                // The name of the binding. Always a String if [[Base]] value is an Environment Record. Otherwise, may be an ECMAScript language value other than a String or a Symbol until ToPropertyKey is performed.
                                match &reference_record.referenced_name {
                                    JSValue::String(value) => {
                                        // c. Return ? base.GetBindingValue(V.[[ReferencedName]], V.[[Strict]]) (see 9.1).
                                        let binding_value = global_record.borrow().get_binding_value(value.to_string(), false);
                                        return CompletionRecord { type_: CompletionRecordType::Normal, value: binding_value.value,  target: None }
                                    },
                                    _ => { unreachable!() }
                                }
                            },
                            _ => { unimplemented!() }

                            // FIXME: We should handle Function and Module enviroment records
                            // FIXME: This is a pretty manual way to dispatch calls to the get_binding_value methods for each respective env record type
                            // Maybe we can use something a bit more dynamic?
                        }
                    },
                    _ => {
                        todo!();
                        //     3. If IsPropertyReference(V) is true, then
                        //  if Interpreter::is_property_reference(&reference_record) {

                        // a. Let baseObj be ? ToObject(V.[[Base]]).
                        //     b. If IsPrivateReference(V) is true, then
                        // i. Return ? PrivateGet(baseObj, V.[[ReferencedName]]).
                        //     c. If V.[[ReferencedName]] is not a property key, then
                        // i. Set V.[[ReferencedName]] to ? ToPropertyKey(V.[[ReerencedName]]).
                        //     d. Return ? baseObj.[[Get]](V.[[ReferencedName]], GetThisValue(V)).
                        // }
                    },
                }

            },
            _ => { unreachable!() }
        }
    }

    // https://tc39.es/ecma262/#sec-ispropertyreference
    fn is_property_reference(reference_record: &ReferenceRecord) -> bool {
        match *reference_record.base {
            // 1. If V.[[Base]] is unresolvable, return false.
            BaseValue::Unresolvable => {
                return false;
            },
            // 2. If V.[[Base]] is an Environment Record, return false; otherwise return true.
            BaseValue::EnvironmentRecord(_) => {
                return false;
            },
            _=> { return true; }
        }
    }

    // https://tc39.es/ecma262/#sec-resolvebinding
    //TODO: environment can also be 'undefined' type
    fn resolve_binding(&self, name: String, environment: Option<Rc<RefCell<EnvironmentRecord>>>) -> CompletionRecord {
            match environment {
                // 1. If env is not present or env is undefined, then
                None => {
                    // a. Set env to the running execution context's LexicalEnvironment.
                    let env = Rc::clone(&self.running_execution_context().lexical_environment_record);
                    // 2. Assert: env is an Environment Record.
                    // 3. TODO: Let strict be IsStrict(the syntactic production that is being evaluated).
                    return Interpreter::get_identifier_reference(name, &Option::from(env), false);
                }
            Some(env_record) => {
                // 3. TODO: Let strict be IsStrict(the syntactic production that is being evaluated).
                // 4. Return ? GetIdentifierReference(env, name, strict).
                return Interpreter::get_identifier_reference(name, &Option::from(env_record), false);
            },

        }
    }

    // https://tc39.es/ecma262/#sec-getidentifierreference
    fn get_identifier_reference(name: String, environment: &Option<Rc<RefCell<EnvironmentRecord>>>, strict: bool) -> CompletionRecord {
        match environment {
            // 1. If env is null, then
            None => {
                // a. Return the Reference Record { [[Base]]: unresolvable, [[ReferencedName]]: name, [[Strict]]: strict, [[ThisValue]]: empty }.
                return CompletionRecord {
                    type_: CompletionRecordType::Normal,
                    value: Rc::new(ReferenceRecordOrJsValue::ReferenceRecord(
                        ReferenceRecord {
                            base: Rc::new(BaseValue::Unresolvable),
                            referenced_name: JSValue::String(name),
                            strict: false, // TODO: Should be passed in
                            this_value: None,
                        }
                    )),
                    target: None,
                }
            }
            Some(env_record) => {
                // 2. Let exists be ? env.HasBinding(name).
                let exists = env_record.borrow().has_binding(name.clone());

                // 3. If exists is true, then
                match exists.value.deref() {
                    ReferenceRecordOrJsValue::JSValue(js_value) => {
                        match js_value.borrow().deref() {
                            JSValue::Boolean(bool_value) => {
                                if *bool_value {
                                    // 3. Return the Reference Record { [[Base]]: env, [[ReferencedName]]: name, [[Strict]]: strict, [[ThisValue]]: empty }.
                                    return CompletionRecord {
                                        type_: CompletionRecordType::Normal,
                                        value: Rc::new(ReferenceRecordOrJsValue::ReferenceRecord(
                                            ReferenceRecord {
                                                base: Rc::new(BaseValue::EnvironmentRecord(Rc::clone(env_record))),
                                                referenced_name: JSValue::String(name),
                                                strict: false,
                                                this_value: None,
                                            }
                                        )),
                                        target: None,
                                    }
                                } else {
                                    // 4. Else
                                    // a. Let outer be env.[[OuterEnv]].
                                    let outer =  &env_record.borrow().outer_environment_record;

                                    // b. Return ? GetIdentifierReference(outer, name, strict).
                                    return Interpreter::get_identifier_reference(name, outer, strict);
                                }
                            },
                            _=> { unreachable!() }
                        }
                    },
                    _=> { unreachable!() }
                }
            }
        }
    }

    pub fn run_file(&mut self, path: String) {
        let file = File::open(path).expect("File could not opened!");
        let mut reader = BufReader::new(file);
        let mut source = String::new();
        reader.read_to_string(&mut source).expect("File could not be read!");
        self.run(source, ExecutionMode::Script);

        if self.had_error {
            std::process::exit(65);
        }
    }

    pub fn run_prompt(&mut self) {
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).expect("Failed to read line");
            self.run(line, ExecutionMode::Shell);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: String, execution_mode: ExecutionMode) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().clone();

        for token in tokens.iter() {
            println!("{}", token.to_string());
        }

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        self.interpret(statements, execution_mode);
    }

    fn error(line: usize, message: String) {
        println!("Error on line {}: {}", line, message);
    }

    fn report(&mut self, line: i64, location: String, message: String) {
        println!("[line {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }

    fn execute(&mut self, statement: &Statement) -> CompletionRecord {
        statement.accept(self)
    }

    // https://tc39.es/ecma262/#sec-evaluation
    // https://tc39.es/ecma262/#sec-completion-record-specification-type
    fn evaluate(&mut self, expression_statement: &ExpressionStatement) -> CompletionRecord {
        expression_statement.accept(self)
    }

    fn interpret(&mut self, statements: Vec<Statement>, execution_mode: ExecutionMode)  {
        for statement in statements.iter() {
            let result = self.execute(statement);
            match result.type_ {
                CompletionRecordType::Normal => {
                    let mut pretty_printer = ASTPrettyPrinter;
                    let expression_ast = statement.accept(&mut pretty_printer);
                    println!("Parsed expression {}", expression_ast);
                    println!("{:?}", result);
                },
                CompletionRecordType::Throw => {
                    println!("Uncaught {:?}", result.value);
                    match execution_mode {
                        ExecutionMode::Script => {
                            exit(1);
                        }
                        ExecutionMode::Shell => {},
                    }
                },
                _ => { unimplemented!() }
            }
        }
    }

    // https://tc39.es/ecma262/#sec-tonumber
    // TODO: Return a normal completion or throw a completion
    fn to_number(value: Rc<RefCell<JSValue>>) -> CompletionRecord {
        match value.borrow().deref() {
            // 1. If argument is a Number, return argument.
            JSValue::Numeric(val) => {
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(value.clone())))
            },
            // 2. If argument is either a Symbol or a BigInt, throw a TypeError exception.
            JSValue::Symbol(value) => {
                todo!()
            },
            // 3. If argument is undefined, return NaN.
            // TODO: Support undefined as a global object
            JSValue::Undefined => {
                // TODO: Implement NaN as a global object and not a string
                // https://tc39.es/ecma262/#sec-value-properties-of-the-global-object-nan
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::String("NaN".to_string()))))))
            },
            // 4. If argument is either null or false, return +0𝔽.
            JSValue::Null | JSValue::Boolean(false) => {
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Numeric(0.0))))));
            },
            // 5. If argument is true, return 1𝔽.
            JSValue::Boolean(true) => {
                return create_normal_completion(Rc::new(ReferenceRecordOrJsValue::JSValue(Rc::new(RefCell::new(JSValue::Numeric(1.0))))));
            }
            //6. If argument is a String, return StringToNumber(argument).
            JSValue::String(value) => {
                todo!();
            }
            // 7. Assert: argument is an Object.
            JSValue::Object(value) => {
                // 8. Let primValue be ? ToPrimitive(argument, number).
                // 9. Assert: primValue is not an Object.
                // 10. Return ? ToNumber(primValue).
                todo!()
            }

        }
    }


    // https://tc39.es/ecma262/#sec-toprimitive
    fn to_primitive(value: Rc<RefCell<JSValue>>, preferred_type: Option<JSValue>) -> Rc<RefCell<JSValue>> {
        match &*value.borrow() {
            // 1. If input is an Object, then
            JSValue::Object(value) => {
                todo!();
            },
            _ => {
                return value.clone();
            }
        }
    }

    // https://tc39.es/ecma262/#sec-tonumeric
    fn to_numeric(value: Rc<RefCell<JSValue>>) -> Rc<RefCell<JSValue>> {
        // 1. Let primValue be ? ToPrimitive(value, number).
        let prim_value = Interpreter::to_primitive(value, None);

        //2. TODO: If primValue is a BigInt, return primValue.

        //3. Return ? ToNumber(primValue).
        match Interpreter::to_number(prim_value).value.deref() {
            ReferenceRecordOrJsValue::JSValue(val) => {
                match Interpreter::to_number(val.clone()).value.deref() {
                    ReferenceRecordOrJsValue::JSValue(val) => {
                        return val.clone();
                    },
                    _ => { unreachable!("Encountered a reference record") }
                }

            },
            _ => { unreachable!("Encountered a reference record") }
        }
    }

    // https://tc39.es/ecma262/#sec-toboolean
    fn to_boolean(value: Rc<RefCell<JSValue>>) -> Rc<RefCell<JSValue>> {
        match value.borrow().deref() {
            //1. If argument is a Boolean, return argument.
            JSValue::Boolean(value) => {
                return Rc::new(RefCell::new(JSValue::Boolean(*value)));
            },
            // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false. TODO: NaN and 0ℤ
            JSValue::Undefined | JSValue::Null | JSValue::Numeric(0.0) | JSValue::Numeric(-0.0) => {
                return Rc::new(RefCell::new(JSValue::Boolean(false)));
            },
            JSValue::String(ref s) if s.is_empty() => {
                return Rc::new(RefCell::new(JSValue::Boolean(false)));
            },
            // 3. If argument is an Object and argument has an [[IsHTMLDDA]] internal slot, return false.
            JSValue::Object(value) => {
                todo!();
            }
            // Handle other cases
            _ => {
                return Rc::new(RefCell::new(JSValue::Boolean(true)));
            }
        }
    }


    // https://tc39.es/ecma262/#sec-applystringornumericbinaryoperator
    fn apply_string_or_numeric_binary_operator(left: Rc<RefCell<JSValue>>, right: Rc<RefCell<JSValue>>, operator: &TokenType) -> Rc<RefCell<JSValue>> {
        // 1. If opText is +, then
        if operator == &TokenType::PLUS {
            // a. Let lPrim be ? ToPrimitive(lVal).
            let left_primitive = Interpreter::to_primitive(left, None);

            // b. Let rPrim be ? ToPrimitive(rVal).
            let right_primitive = Interpreter::to_primitive(right, None);

            let left_prim_ref = left_primitive.borrow();
            let left_prim = left_prim_ref.deref();
            match left_prim {
                // c. If lPrim is a String or rPrim is a String, then
                JSValue::String(ref value) => {
                    // i. Let lStr be ? ToString(lPrim).
                    let left_string = Interpreter::to_string(left_primitive.clone());

                    // ii. Let rStr be ? ToString(rPrim).
                    let right_string = Interpreter::to_string(right_primitive.clone());

                    match left_string {
                        JSValue::String(ref left_string) => {
                            match right_string {
                                JSValue::String(ref right_string) => {
                                    // iii. Return the string-concatenation of lStr and rStr.
                                    return Rc::new(RefCell::new(JSValue::String(format!("{}{}", left_string, right_string))));
                                },
                                _ => { panic!("Unexpected right JS value: {:?}", right_string) }
                            }
                        },
                        _ => { panic!("Unexpected left JS value: {:?}", right_string) }
                    }
                },
                _ => {
                    match right_primitive.borrow().deref() {
                        // c. If lPrim is a String or rPrim is a String, then
                        JSValue::String(ref value) => {
                            let left_string = Interpreter::to_string(left_primitive.clone());
                            let right_string = Interpreter::to_string(right_primitive.clone());

                            match left_string {
                                JSValue::String(ref left_string) => {
                                    match right_string {
                                        JSValue::String(ref right_string) => {
                                            return Rc::new(RefCell::new(JSValue::String(format!("{}{}", left_string, right_string))));
                                        },
                                        _ => { panic!("Unexpected right JS value: {:?}", right_string) }
                                    }
                                },
                                _ => { panic!("Unexpected left JS value: {:?}", right_string) }
                            }
                        },
                        _ => {
                            // We know the opText is still '+' so apply the addition operation.
                            // https://tc39.es/ecma262/#sec-numeric-types-number-add
                            // Implement to spec

                            // 2. NOTE: At this point, it must be a numeric operation.

                            //3. Let lNum be ? ToNumeric(lVal).
                            let left_numeric = Interpreter::to_numeric(left_primitive.clone());

                            //4. Let rNum be ? ToNumeric(rVal).
                            let right_numeric = Interpreter::to_numeric(right_primitive.clone());

                            // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                            if !Interpreter::same_type(&left_numeric.borrow(), &right_numeric.borrow()) {
                                todo!("Throw TypeError exception");
                            }

                            // TODO: 6. If lNum is a BigInt, then

                            //7. Else,
                            let left_num_ref = left_numeric.borrow();
                            let right_num_ref = right_numeric.borrow();
                            let left_num = left_num_ref.deref();
                            let right_num = right_num_ref.deref();
                            match (left_num, right_num) {
                                (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                                    return Rc::new(RefCell::new(JSValue::Numeric(left_value + right_value)));
                                },
                                _ => { panic!("Unexpected right JS value") }
                            }
                        }
                    }
                }
            }
        } else {
            // d. Set lVal to lPrim.
            // e. Set rVal to rPrim.
            let left_primitive = Interpreter::to_primitive(left, None);
            let right_primitive = Interpreter::to_primitive(right, None);

            match operator {
                // https://tc39.es/ecma262/#sec-numeric-types-number-multiply
                // TODO: Implement to spec
                TokenType::STAR => {
                    // 2. NOTE: At this point, it must be a numeric operation.

                    //3. Let lNum be ? ToNumeric(lVal).
                    let left_numeric = Interpreter::to_numeric(left_primitive);

                    //4. Let rNum be ? ToNumeric(rVal).
                    let right_numeric = Interpreter::to_numeric(right_primitive);

                    // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                    if !Interpreter::same_type(&left_numeric.borrow(), &right_numeric.borrow()) {
                        todo!("Throw TypeError exception");
                    }

                    // TODO: 6. If lNum is a BigInt, then

                    //7. Else,
                    let left_borrowed = left_numeric.borrow();
                    let right_borrowed = right_numeric.borrow();
                    let left_ref = left_borrowed.deref();
                    let right_ref = right_borrowed.deref();

                    match (left_ref, right_ref) {
                        (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                            return Rc::new(RefCell::new(JSValue::Numeric(left_value * right_value)));
                        },
                        _ => { panic!("Unexpected right JS value") }
                    }
                },
                // https://tc39.es/ecma262/#sec-numeric-types-number-divide
                // TODO: Implement to spec
                TokenType::SLASH => {
                    // 2. NOTE: At this point, it must be a numeric operation.

                    //3. Let lNum be ? ToNumeric(lVal).
                    let left_numeric = Interpreter::to_numeric(left_primitive);

                    //4. Let rNum be ? ToNumeric(rVal).
                    let right_numeric = Interpreter::to_numeric(right_primitive);

                    // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                    if !Interpreter::same_type(&left_numeric.borrow(), &right_numeric.borrow()) {
                        todo!("Throw TypeError exception");
                    }

                    // TODO: 6. If lNum is a BigInt, then

                    //7. Else,
                    let left_borrowed = left_numeric.borrow();
                    let right_borrowed = right_numeric.borrow();
                    let left_ref = left_borrowed.deref();
                    let right_ref = right_borrowed.deref();

                    match (left_ref, right_ref) {
                        (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                            return Rc::new(RefCell::new(JSValue::Numeric(left_value / right_value)));
                        },
                        _ => { panic!("Unexpected right JS value") }
                    }
                },
                // https://tc39.es/ecma262/#sec-numeric-types-number-subtract
                // Implement to spec
                TokenType::MINUS => {
                    // 2. NOTE: At this point, it must be a numeric operation.

                    //3. Let lNum be ? ToNumeric(lVal).
                    let left_numeric = Interpreter::to_numeric(left_primitive);

                    //4. Let rNum be ? ToNumeric(rVal).
                    let right_numeric = Interpreter::to_numeric(right_primitive);

                    // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                    if !Interpreter::same_type(&left_numeric.borrow(), &right_numeric.borrow()) {
                        todo!("Throw TypeError exception");
                    }

                    // TODO: 6. If lNum is a BigInt, then

                    //7. Else,
                    let left_borrowed = left_numeric.borrow();
                    let right_borrowed = right_numeric.borrow();
                    let left_ref = left_borrowed.deref();
                    let right_ref = right_borrowed.deref();

                    match (left_ref, right_ref) {
                        (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                            return Rc::new(RefCell::new(JSValue::Numeric(left_value - right_value)));
                        },
                        _ => { panic!("Unexpected right JSValue") }
                    }
                },
                _ => { panic!("Unexpected operator: {:?}", operator) }
            }
        }




    }

    // https://tc39.es/ecma262/#sec-tostring
    fn to_string(value: Rc<RefCell<JSValue>>) -> JSValue {
        match value.borrow().deref() {
            // 1. If argument is a String, return argument.
            JSValue::String(value) => {
                return JSValue::String(value.clone());
            },
            // 2. If argument is a Symbol, throw a TypeError exception.
            JSValue::Symbol(value) => {
                todo!("Throw a TypeError exception");
            },
            // 3. If argument is undefined, return "undefined".
            JSValue::Undefined => {
                return JSValue::String("undefined".to_string());
            }
            // 4. If argument is null, return "null".
            JSValue::Null => {
                return JSValue::String("null".to_string());
            },
            // 5. If argument is true, return "true".
            JSValue::Boolean(true) => {
                return JSValue::String("true".to_string());
            },
            // 6. If argument is false, return "false".
            JSValue::Boolean(false) => {
                return JSValue::String("false".to_string());
            },
            // 7. If argument is a Number, return Number::toString(argument, 10).
            JSValue::Numeric(value) => {
                return JSValue::String(Interpreter::number_to_string(value.clone()));
            },
            // 8. TODO: If argument is a BigInt, return BigInt::toString(argument, 10).

            // 9. Assert: argument is an Object.
            JSValue::Object(value) => {
                // 10. Let primValue be ? ToPrimitive(argument, string).
                // 11. Assert: primValue is not an Object.
                // 12. Return ? ToString(primValue).
                todo!();
            }
        }
    }

    // https://tc39.es/ecma262/#sec-numeric-types-number-tostring
    // TODO: Implement this to spec, for now we'll just use Rust's default implementation of to_string on numbers
    fn number_to_string(value: Number) -> String {
        return value.to_string();
    }

    // https://tc39.es/ecma262/#sec-sametype
    fn same_type(left: &JSValue, right: &JSValue) -> bool {

        match (left, right) {
            // 1. If x is undefined and y is undefined, return true.
            (JSValue::Undefined, JSValue::Undefined) => {
                return true;
            },
            // 2. If x is null and y is null, return true.
            (JSValue::Null, JSValue::Null) => {
                return true;
            },
            // 3. If x is a Boolean and y is a Boolean, return true.
            (JSValue::Boolean(_), JSValue::Boolean(_)) => {
                return true;
            },
            // 4. If x is a Number and y is a Number, return true.
            (JSValue::Numeric(_), JSValue::Numeric(_)) => {
                return true;
            },
            // 5. TODO:  If x is a BigInt and y is a BigInt, return true.

            // 6. If x is a Symbol and y is a Symbol, return true.
            (JSValue::Symbol(_), JSValue::Symbol(_)) => {
                return true;
            },
            // 7. If x is a String and y is a String, return true.
            (JSValue::String(_), JSValue::String(_)) => {
                return true;
            },
            // 8. If x is an Object and y is an Object, return true.
            (JSValue::Object(_), JSValue::Object(_)) => {
                return true;
            },
            // 9. Return false.
            _ => {
                return false;
            }
        }
    }

}

enum ExecutionMode {
    Shell,
    Script
}