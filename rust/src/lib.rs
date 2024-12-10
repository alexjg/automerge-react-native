uniffi::setup_scaffolding!();

use std::{
    borrow::Borrow,
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use automerge::{self as am, transaction::Transactable, ObjType, ReadDoc, ScalarValue};
use uniffi;

type ActorId = String;
type ChangeHash = String;
type Hash = String;
type ObjId = String;

#[derive(uniffi::Object)]
struct FullValue(am::Value<'static>, am::ObjId);

#[uniffi::export]
impl FullValue {
    fn get_type(&self) -> String {
        String::from(match &self.0 {
            am::Value::Scalar(s) => match s.borrow() {
                ScalarValue::Str(_) => "str",
                ScalarValue::Int(_) => "int",
                ScalarValue::Uint(_) => "uint",
                ScalarValue::F64(_) => "f64",
                ScalarValue::Boolean(_) => "boolean",
                ScalarValue::Timestamp(_) => "timestamp",
                ScalarValue::Counter(_) => "counter",
                ScalarValue::Bytes(_) => "bytes",
                ScalarValue::Null => "null",
                ScalarValue::Unknown {
                    type_code: _,
                    bytes: _,
                } => "unknown",
            },
            am::Value::Object(ObjType::List) => "list",
            am::Value::Object(ObjType::Map) => "map",
            am::Value::Object(ObjType::Table) => "table",
            am::Value::Object(ObjType::Text) => "text",
        })
    }

    fn get_value_string(&self) -> String {
        String::from(match &self.0 {
            am::Value::Scalar(s) => match s.borrow() {
                ScalarValue::Str(s) => s.to_string(),
                // TODO: Add proper error handling
                _ => panic!("Expected a string"),
            },
            // TODO: Add proper error handling
            _ => panic!("Expected a string"),
        })
    }
}

#[derive(uniffi::Record)]
struct ValueRecord {
    value_type: String,
    value_str: Option<String>,
    // TODO: Add fields for every scalar
}

impl<'a> From<&(am::Value<'a>, am::ObjId)> for ValueRecord {
    fn from(value: &(am::Value<'a>, am::ObjId)) -> Self {
        let full_value = FullValue(value.0.to_owned(), value.1.to_owned());
        let value_type = full_value.get_type();
        ValueRecord {
            value_type,
            value_str: value.0.to_str().map(|s| String::from(s)),
        }
    }
}

// TODO: Remove this temporary hack
type MaterializedObjectOfStrings = HashMap<String, String>;

#[derive(uniffi::Enum)]
pub enum Prop {
    Map(String),
    Seq(u32),
}

impl From<Prop> for am::Prop {
    fn from(value: Prop) -> Self {
        match value {
            Prop::Map(s) => am::Prop::Map(s),
            Prop::Seq(i) => am::Prop::Seq(i.try_into().unwrap()),
        }
    }
}

impl From<&am::Prop> for Prop {
    fn from(value: &am::Prop) -> Self {
        match value {
            am::Prop::Map(s) => Prop::Map(s.to_string()),
            am::Prop::Seq(i) => Prop::Seq(i.to_owned().try_into().unwrap()),
        }
    }
}

// Uniffi lowerable equivalent of am::Patch
// See https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge/src/patches/patch.rs#L11-L24
#[derive(uniffi::Record)]
struct Patch {
    /// The object this patch modifies
    obj: ObjId,
    /// The path to the property in the parent object where this object lives
    path: Vec<PathSegment>,
    /// The change this patch represents
    action: PatchAction,
}

#[derive(uniffi::Record)]
struct PathSegment {
    obj: ObjId,
    prop: Prop,
}

// Uniffi lowerable equivalent of am::PatchAction
// See https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge/src/patches/patch.rs#L26-L83
// TODO: Implement the rest of the enum
#[derive(uniffi::Enum)]
enum PatchAction {
    /// A key was created or updated in a map
    PutMap {
        key: String,
        /// The value that was inserted and the object ID of the object that was inserted. Note
        /// that the Object ID is only meaningful for `Value::Obj` values
        value: ValueRecord,
        /// Whether there is a conflict at this key. If there is a conflict this patch represents
        /// the "winning" value of the conflict. The conflicting values can be obtained with
        /// [`crate::ReadDoc::get_all`]
        conflict: bool,
    },
}

impl From<&am::Patch> for Patch {
    fn from(value: &am::Patch) -> Self {
        Patch {
            obj: value.obj.to_string(),
            path: value
                .path
                .iter()
                .map(|(obj, prop)| PathSegment {
                    obj: obj.to_string(),
                    prop: Prop::from(prop),
                })
                .collect(),
            action: match &value.action {
                am::PatchAction::PutMap {
                    key,
                    value,
                    conflict,
                } => PatchAction::PutMap {
                    key: key.to_owned(),
                    value: ValueRecord::from(value),
                    conflict: conflict.to_owned(),
                },
                _ => todo!("Not yet implemented"),
            },
        }
    }
}

/// @see https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/javascript/src/stable.ts#L162-L176
#[derive(uniffi::Record)]
pub struct InitOptions {
    /// The actor ID to use for this document, a random one will be generated if `null` is passed
    actor: Option<ActorId>,
    freeze: Option<bool>,
    /// A callback which will be called with the initial patch once the document has finished loading
    // TODO: Add this in https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/javascript/src/types.ts#L66-L69
    // patchCallback?: PatchCallback<T>
    /// @hidden
    enable_text_v2: Option<bool>,
    /// @hidden
    unchecked: Option<bool>,
    /// Allow loading a document with missing changes
    allow_missing_changes: Option<bool>,
    /// @hidden
    convert_raw_strings_to_text: Option<bool>,
}

#[uniffi::export]
pub fn create(options: Option<InitOptions>) -> Result<Automerge, Error> {
    Ok(Automerge {
        // TODO: Use load_with_options
        // @see https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/lib.rs#L1254-L1260
        doc: Arc::new(Mutex::new(am::AutoCommit::new())),
        freeze: AtomicBool::new(false),
    })
}

#[derive(uniffi::Object)]
pub struct Automerge {
    doc: Arc<Mutex<am::AutoCommit>>,
    freeze: AtomicBool,
}

#[uniffi::export]
impl Automerge {
    fn get_heads(&self) -> Vec<ChangeHash> {
        self.doc
            .lock()
            .unwrap()
            .get_heads()
            .iter()
            .map(|&head| head.to_string())
            .collect()
    }

    fn isolate(&self, heads: Vec<ChangeHash>) {
        let heads_parsed = get_heads(Some(heads));
        self.doc.lock().unwrap().isolate(&heads_parsed);
    }

    fn pending_ops(&self) -> u32 {
        self.doc.lock().unwrap().pending_ops().try_into().unwrap()
    }

    fn integrate(&self) {
        self.doc.lock().unwrap().integrate()
    }

    fn rollback(&self) -> u32 {
        self.doc.lock().unwrap().rollback().try_into().unwrap()
    }

    fn commit(&self, message: Option<String>, time: Option<f64>) -> Option<Hash> {
        let mut commit_opts = am::transaction::CommitOptions::default();
        if let Some(message) = message {
            commit_opts.set_message(message);
        }
        if let Some(time) = time {
            commit_opts.set_time(time as i64);
        }
        self.doc
            .lock()
            .unwrap()
            .commit_with(commit_opts)
            .map(|h| h.to_string())
    }

    // https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/lib.rs#L193-L204
    fn keys(&self, obj: ObjId, heads: Option<Vec<ChangeHash>>) -> Vec<String> {
        let obj_parsed = am::ObjId::try_from(hex::decode(obj).unwrap().as_slice()).unwrap();
        if let Some(heads) = heads {
            let heads_parsed: Vec<am::ChangeHash> = heads
                .iter()
                .map(|h| am::ChangeHash::from_str(h.as_str()).unwrap())
                .collect();
            self.doc
                .lock()
                .unwrap()
                .keys_at(obj_parsed, &heads_parsed)
                .map(|k| k.to_string())
                .collect()
        } else {
            self.doc
                .lock()
                .unwrap()
                .keys(obj_parsed)
                .map(|k| k.to_string())
                .collect()
        }
    }

    fn get_from_map_with_type(
        &self,
        obj: ObjId,
        prop: String,
        heads: Option<Vec<ChangeHash>>,
    ) -> Result<FullValue, Error> {
        self.get_with_type(obj, am::Prop::Map(prop), heads)
    }

    fn get_from_seq_with_type(
        &self,
        obj: ObjId,
        prop: u32,
        heads: Option<Vec<ChangeHash>>,
    ) -> Result<FullValue, Error> {
        self.get_with_type(obj, am::Prop::Seq(prop.try_into().unwrap()), heads)
    }

    fn enable_freeze(&self, enable: bool) -> bool {
        self.freeze.swap(enable, Ordering::Relaxed)
    }

    // https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/lib.rs#L986-L997
    fn materialize(
        &self,
        obj: Option<ObjId>,
        heads: Option<Vec<ChangeHash>>,
        // meta: JsValue,
    ) -> MaterializedObjectOfStrings {
        let (obj, _) = self.import(obj).unwrap_or((am::ROOT, am::ObjType::Map));
        self.doc.lock().unwrap().update_diff_cursor();
        // TODO: Implement the equivalent of an export cache https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/export_cache.rs
        // For now, we expect all materialize to be called only on root objects with string properties
        assert!(
            obj == am::ROOT,
            "expected materialize to be called with the ROOT obj"
        );
        let mut result = MaterializedObjectOfStrings::new();
        let doc = self.doc.lock().unwrap();
        let mut iter = if let Some(heads) = heads {
            let heads = get_heads(Some(heads));
            doc.map_range_at(obj, .., heads.as_slice())
        } else {
            doc.map_range(obj, ..)
        };
        // Copied from https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/export_cache.rs#L114-L125
        while let Some(map_item) = iter.next() {
            match map_item.value {
                am::Value::Object(_) => {
                    todo!("Not yet implemented")
                }
                am::Value::Scalar(s) => {
                    assert!(s.is_str(), "Expected doc to have only string values");
                    let value = s.to_str().unwrap();
                    result.insert(String::from(map_item.key), String::from(value));
                }
            }
        }
        result
    }

    fn put_string_to_map(&self, obj: ObjId, prop: String, value: String) {
        let (obj, _) = self.import(Some(obj)).unwrap();
        self.doc.lock().unwrap().put(&obj, prop, value).unwrap();
    }

    fn diff_incremental(&self) -> Vec<Patch> {
        let mut doc = self.doc.lock().unwrap();
        doc.diff_incremental()
            .iter()
            .map(|p: &automerge::Patch| Patch::from(p))
            .collect()
    }
}

pub(crate) fn get_heads(heads: Option<Vec<ChangeHash>>) -> Vec<am::ChangeHash> {
    if let Some(heads) = heads {
        heads
            .iter()
            .map(|head| am::ChangeHash::from_str(head.as_str()).unwrap())
            .collect()
    } else {
        vec![]
    }
}

impl Automerge {
    // TODO: Support more return types than string
    fn get_with_type(
        &self,
        obj: ObjId,
        prop: am::Prop,
        heads: Option<Vec<ChangeHash>>,
    ) -> Result<FullValue, Error> {
        let doc = self.doc.lock().unwrap();
        let obj_parsed = am::ObjId::try_from(hex::decode(obj).unwrap().as_slice()).unwrap();
        let result = if let Some(heads) = heads {
            let heads_parsed: Vec<am::ChangeHash> = heads
                .iter()
                .map(|h| am::ChangeHash::from_str(h.as_str()).unwrap())
                .collect();
            doc.get_at(obj_parsed, prop, &heads_parsed)
        } else {
            doc.get(obj_parsed, prop)
        };
        match result {
            Ok(Some((value, id))) => Ok(FullValue(value.to_owned(), id)),
            Ok(None) => Err(Error::AutomergeError(String::from("Not yet implemented"))),
            Err(err) => Err(Error::AutomergeError(err.to_string())),
        }
    }

    // Copied from https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/interop.rs#L1212-L1275

    pub(crate) fn import(&self, id: Option<String>) -> Result<(am::ObjId, am::ObjType), Error> {
        if let Some(id) = id {
            // valid formats are
            // 123@aabbcc
            // 123@aabccc/prop1/prop2/prop3
            // /prop1/prop2/prop3
            let mut components = id.split('/');
            let obj = components.next();
            let (id, obj_type) = if obj == Some("") {
                (am::ROOT, am::ObjType::Map)
            } else {
                self.doc
                    .lock()
                    .unwrap()
                    .import(obj.unwrap_or_default())
                    .map_err(|_| Error::ImportObj)?
                // .map_err(error::ImportObj::BadImport)?
            };
            self.import_path(id, obj_type, components)
                // .map_err(|e| error::ImportObj::InvalidPath(s.to_string(), e))
                .map_err(|_| Error::ImportObj)
        } else {
            Err(Error::ImportObj)
        }
    }

    pub(crate) fn import_path<'a, I: Iterator<Item = &'a str>>(
        &self,
        mut obj: am::ObjId,
        mut obj_type: am::ObjType,
        components: I,
    ) -> Result<(am::ObjId, am::ObjType), Error> {
        let doc = self.doc.lock().unwrap();
        for (_i, prop) in components.enumerate() {
            if prop.is_empty() {
                break;
            }
            let is_map = matches!(obj_type, am::ObjType::Map | am::ObjType::Table);
            let val = if is_map {
                doc.get(obj, prop).map_err(|_| Error::ImportObj)?
            } else {
                let idx = prop.parse().map_err(|_| Error::ImportObj)?;
                doc.get(obj, am::Prop::Seq(idx))
                    .map_err(|_| Error::ImportObj)?
            };
            match val {
                Some((am::Value::Object(am::ObjType::Map), id)) => {
                    obj_type = am::ObjType::Map;
                    obj = id;
                }
                Some((am::Value::Object(am::ObjType::Table), id)) => {
                    obj_type = am::ObjType::Table;
                    obj = id;
                }
                Some((am::Value::Object(am::ObjType::List), id)) => {
                    obj_type = am::ObjType::List;
                    obj = id;
                }
                Some((am::Value::Object(am::ObjType::Text), id)) => {
                    obj_type = am::ObjType::Text;
                    obj = id;
                }
                None => return Err(Error::ImportObj),
                _ => return Err(Error::ImportObj),
            };
        }
        Ok((obj, obj_type))
    }

    // Copied from https://github.com/automerge/automerge/blob/ec3b085bb6d420dba5519d2cb01fad5f34afeeda/rust/automerge-wasm/src/interop.rs#L1120-L1162
    // pub(crate) fn apply_patch(
    //     &self,
    //     root: Object,
    //     patch: &Patch,
    //     meta: &JsValue,
    //     cache: &mut ExportCache<'_>,
    // ) -> Result<Object, error::ApplyPatch> {
    //     let (_, root_cache) = self.unwrap_object(&root, cache, meta)?;
    //     let mut current = root_cache.clone();
    //     for (i, p) in patch.path.iter().enumerate() {
    //         let prop = prop_to_js(&p.1);
    //         let subval = js_get(&current.inner, &prop)?.0;
    //         if subval.is_string() && patch.path.len() - 1 == i {
    //             let s = subval.dyn_into::<JsString>().unwrap();
    //             let new_text = self.apply_patch_to_text(&s, patch)?;
    //             js_set(&current.inner, &prop, &new_text)?;
    //             return Ok(root_cache.outer);
    //         }
    //         if subval.is_object() {
    //             let subval = subval.dyn_into::<Object>().unwrap();
    //             let (cache_hit, cached_obj) = self.unwrap_object(&subval, cache, meta)?;
    //             if !cache_hit {
    //                 js_set(&current.inner, &prop, &cached_obj.outer)?;
    //             }
    //             current = cached_obj;
    //         } else {
    //             return Ok(root); // invalid patch
    //         }
    //     }
    //     if current.id != patch.obj {
    //         return Ok(root);
    //     }
    //     if current.inner.is_array() {
    //         let inner_array = current
    //             .inner
    //             .dyn_into::<Array>()
    //             .map_err(|_| error::ApplyPatch::NotArray)?;
    //         self.apply_patch_to_array(&inner_array, patch, meta, cache)?;
    //     } else {
    //         self.apply_patch_to_map(&current.inner, patch, meta, cache)?;
    //     }
    //     Ok(root_cache.outer)
    // }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum Error {
    #[error("Not yet implemented")]
    NotYetImplemented,
    #[error("Failed to import object")]
    ImportObj,
    #[error("Bad actor id")]
    BadActorId,
    #[error("Core automerge error: {0}")]
    AutomergeError(String),
}
