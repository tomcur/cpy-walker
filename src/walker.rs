use std::collections::{HashMap, VecDeque};

use crate::error::{Error, Result};
use crate::interpreter::{Pointer as Ptr, *};
use crate::memory::Memory;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataPointer(usize);

#[derive(Debug)]
pub enum DecodedData {
    Type(String),
    Object {
        object_type: DataPointer,
        object_type_name: String,
        attributes: HashMap<String, DataPointer>,
    },
    None,
    Bytes(Vec<u8>),
    String(String),
    Tuple(Vec<DataPointer>),
    List(Vec<DataPointer>),
    Dict(HashMap<DataPointer, DataPointer>),
    Error(Error),
}

struct Decoded {
    object_data: DecodedData,
    type_object_data: DecodedData,
    type_object_pointer: DataPointer,
}

fn step<P, O, M>(
    mem: &M,
    object: O,
    graph: &mut HashMap<DataPointer, DecodedData>,
    queue: &mut VecDeque<O>,
    memoized_types: &mut HashMap<usize, O::TypeObject>,
) -> Result<Decoded>
where
    P: Ptr,
    O: Object<Pointer = P> + TryDeref<Pointer = P> + Clone + std::fmt::Debug,
    M: Memory,
{
    // let object: O = pointer.try_deref(mem)?;

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
                                step::<P, O, M>(mem, key, graph, queue, memoized_types)?.object_data
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
        Type::Bytes => DecodedData::Bytes(typed.as_bytes().unwrap().read(mem)?),
        Type::String => DecodedData::String(typed.as_string().unwrap().read(mem)?),
        Type::Unicode => DecodedData::String(typed.as_unicode().unwrap().read(mem)?),
        Type::Tuple => {
            let tuple = typed.as_tuple().unwrap();

            let mut items = Vec::new();

            for item in tuple.items(mem)? {
                items.push(DataPointer(item.me().address()));
                queue.push_back(item);
            }

            DecodedData::Tuple(items)
        }
        Type::List => {
            let list = typed.as_list().unwrap();

            let mut items = Vec::new();

            for item in list.items(mem)? {
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
    };

    Ok(Decoded {
        object_data: decoded,
        type_object_data: type_object_data,
        type_object_pointer: DataPointer(type_ptr.address()),
    })
}

pub fn walk<P, O, M>(mem: &M, pointer: P) -> HashMap<DataPointer, DecodedData>
where
    P: Ptr + Copy,
    O: Object<Pointer = P> + TryDeref<Pointer = P> + Clone + std::fmt::Debug,
    M: Memory,
{
    let mut graph: HashMap<DataPointer, DecodedData> = HashMap::new();
    let mut queue: VecDeque<O> = VecDeque::new();
    let mut memoized_types: HashMap<usize, O::TypeObject> = HashMap::new();

    if let Ok(object) = pointer.try_deref(mem) {
        queue.push_back(object);
    }

    while let Some(object) = queue.pop_front() {
        let address = object.me().address();
        if graph.contains_key(&DataPointer(address)) {
            continue;
        }

        match step::<P, O, M>(mem, object, &mut graph, &mut queue, &mut memoized_types) {
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
