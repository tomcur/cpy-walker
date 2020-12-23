use num_bigint::BigInt;
use std::collections::{HashMap, VecDeque};

use crate::error::{Error, Result};
use crate::interpreter::*;
use crate::memory::Memory;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataPointer(pub usize);

#[derive(Debug)]
pub enum DecodedData {
    Type(String),
    Object {
        object_type: DataPointer,
        object_type_name: String,
        attributes: HashMap<String, DataPointer>,
    },
    None,
    Class {
        class_name: String,
        bases: Option<DataPointer>,
    },
    Instance {
        instance_class: DataPointer,
        instance_class_name: String,
        attributes: HashMap<String, DataPointer>,
    },
    Bytes(Vec<u8>),
    String(String),
    Tuple(Vec<DataPointer>),
    List(Vec<DataPointer>),
    Dict(HashMap<DataPointer, DataPointer>),
    Bool(bool),
    Int(BigInt),
    Float(f64),
    Error(Error),
}

struct Decoded {
    object_data: DecodedData,
    type_object_data: DecodedData,
    type_object_pointer: DataPointer,
}

fn step<I, M>(
    mem: &M,
    object: I::Object,
    graph: &mut HashMap<DataPointer, DecodedData>,
    queue: &mut VecDeque<I::Object>,
    memoized_types: &mut HashMap<usize, I::TypeObject>,
) -> Result<Decoded>
where
    I: Interpreter,
    M: Memory,
{
    let type_ptr = object.ob_type_pointer();
    let type_object = if let Some(type_object) = memoized_types.get(&type_ptr.address()) {
        type_object
    } else {
        let type_object = object.ob_type(mem)?;
        memoized_types.insert(type_ptr.address(), type_object);
        memoized_types.get(&type_ptr.address()).unwrap()
    };
    let type_name = type_object.name().to_string();
    let type_object_data = DecodedData::Type(type_object.name().to_string());

    let typed = type_object.downcast(mem, object)?;

    let decoded = match typed.object_type() {
        Type::Type => DecodedData::Type(typed.as_type().unwrap().name().to_string()),
        Type::Object => {
            let (_type_object, object) = typed.as_object().unwrap();
            let attr_dict = object.attributes(mem)?;

            DecodedData::Object {
                object_type: DataPointer(type_ptr.address()),
                object_type_name: type_name,
                attributes: match attr_dict {
                    Some(dict) => {
                        let mut attributes = HashMap::new();
                        for (_hash, key, value) in
                            dict.entries(mem)?.into_iter().map(|entry| entry.take())
                        {
                            // If the input data is is bad, this might recurse forever.
                            if let DecodedData::String(string) =
                                step::<I, M>(mem, key, graph, queue, memoized_types)?.object_data
                            {
                                attributes.insert(string, DataPointer(value.me().address()));
                                queue.push_back(value);
                            }
                        }
                        attributes
                    }
                    None => HashMap::new(),
                },
            }
        }
        Type::None => DecodedData::None,
        Type::Class => {
            let class = typed.as_class().unwrap();

            DecodedData::Class {
                class_name: class.name().to_owned(),
                bases: match class.bases(mem)? {
                    Some(base_class) => {
                        queue.push_back(base_class.to_object());
                        Some(DataPointer(base_class.to_object().me().address()))
                    }
                    None => None,
                },
            }
        }
        Type::Instance => {
            let instance = typed.as_instance().unwrap();
            let class = instance.class(mem)?;
            let attr_dict = instance.attributes(mem)?;

            DecodedData::Instance {
                instance_class: DataPointer(class.to_object().me().address()),
                instance_class_name: class.name().to_owned(),
                attributes: {
                    let mut attributes = HashMap::new();
                    for (_hash, key, value) in attr_dict
                        .entries(mem)?
                        .into_iter()
                        .map(|entry| entry.take())
                    {
                        // If the input data is is bad, this might recurse forever.
                        if let DecodedData::String(string) =
                            step::<I, M>(mem, key, graph, queue, memoized_types)?.object_data
                        {
                            attributes.insert(string, DataPointer(value.me().address()));
                            queue.push_back(value);
                        }
                    }
                    attributes
                },
            }
        }

        Type::Bytes => DecodedData::Bytes(typed.as_bytes().unwrap().read(mem)?),
        Type::String => DecodedData::String(typed.as_string().unwrap().read(mem)?),
        Type::Unicode => DecodedData::String(typed.as_unicode().unwrap().read(mem)?),
        Type::Tuple => {
            let tuple = typed.as_tuple().unwrap();

            let mut items = Vec::new();

            for item in tuple
                .items(mem)
                .take_while(Result::is_ok)
                .map(Result::unwrap)
            {
                items.push(DataPointer(item.me().address()));
                queue.push_back(item);
            }

            DecodedData::Tuple(items)
        }
        Type::List => {
            let list = typed.as_list().unwrap();

            let mut items = Vec::new();

            for item in list
                .items(mem)
                .take_while(Result::is_ok)
                .map(Result::unwrap)
            {
                items.push(DataPointer(item.me().address()));
                queue.push_back(item);
            }

            DecodedData::List(items)
        }
        Type::Dict => {
            let dict = typed.as_dict().unwrap();

            let mut entries = HashMap::new();

            for (_hash, key, value) in dict.entries(mem)?.into_iter().map(|entry| entry.take()) {
                entries.insert(
                    DataPointer(key.me().address()),
                    DataPointer(value.me().address()),
                );
                queue.push_back(key);
                queue.push_back(value);
            }

            DecodedData::Dict(entries)
        }
        Type::Bool => DecodedData::Bool(typed.as_bool().unwrap().value()),
        Type::Int => DecodedData::Int(typed.as_int().unwrap().read(mem)?),
        Type::Float => DecodedData::Float(typed.as_float().unwrap().value()),
    };

    Ok(Decoded {
        object_data: decoded,
        type_object_data,
        type_object_pointer: DataPointer(type_ptr.address()),
    })
}

pub fn walk<I, M>(mem: &M, pointer: Pointer) -> HashMap<DataPointer, DecodedData>
where
    I: Interpreter,
    M: Memory,
{
    let mut graph: HashMap<DataPointer, DecodedData> = HashMap::new();
    let mut queue: VecDeque<I::Object> = VecDeque::new();
    let mut memoized_types: HashMap<usize, I::TypeObject> = HashMap::new();

    if let Ok(object) = pointer.try_deref_me(mem) {
        queue.push_back(object);
    }

    while let Some(object) = queue.pop_front() {
        let address = object.me().address();
        if graph.contains_key(&DataPointer(address)) {
            continue;
        }

        match step::<I, M>(mem, object, &mut graph, &mut queue, &mut memoized_types) {
            Ok(Decoded {
                object_data,
                type_object_data,
                type_object_pointer,
            }) => {
                graph.insert(type_object_pointer, type_object_data);
                graph.insert(DataPointer(address), object_data);
            }
            Err(error) => {
                graph.insert(DataPointer(address), DecodedData::Error(error));
            }
        };
    }

    graph
}
