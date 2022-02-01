use crate::{ffi, util, BtResult, BtResultExt};
use ordered_float::OrderedFloat;
use std::collections::BTreeSet;
use std::{fmt, ptr, slice};

/// Fields are containers of trace data: they are found in events and packets
pub struct Field {
    field: *const ffi::bt_field,
    class: *const ffi::bt_field_class,
}

impl Field {
    pub(crate) fn from_raw(field: *const ffi::bt_field) -> Option<Self> {
        if field.is_null() {
            None
        } else {
            let class = unsafe { ffi::bt_field_borrow_class_const(field) };
            Field { field, class }.into()
        }
    }

    pub fn get_type(&self) -> FieldType {
        let typ = unsafe { ffi::bt_field_class_get_type(self.class) };
        FieldType::from_raw(typ)
    }

    pub fn to_owned(self) -> BtResult<Option<OwnedField>> {
        use FieldType::*;
        Ok(match self.get_type() {
            Bool => {
                let v = unsafe { ffi::bt_field_bool_get_value(self.field) };
                Some(OwnedField::Scalar(None, ScalarField::Bool(v != 0)))
            }
            UnsignedInteger => {
                let v = unsafe { ffi::bt_field_integer_unsigned_get_value(self.field) };
                Some(OwnedField::Scalar(None, ScalarField::UnsignedInteger(v)))
            }
            SignedInteger => {
                let v = unsafe { ffi::bt_field_integer_signed_get_value(self.field) };
                Some(OwnedField::Scalar(None, ScalarField::SignedInteger(v)))
            }
            SinglePrecisionReal => {
                let v = unsafe { ffi::bt_field_real_single_precision_get_value(self.field) };
                Some(OwnedField::Scalar(
                    None,
                    ScalarField::SinglePrecisionReal(v.into()),
                ))
            }
            DoublePrecisionReal => {
                let v = unsafe { ffi::bt_field_real_double_precision_get_value(self.field) };
                Some(OwnedField::Scalar(
                    None,
                    ScalarField::DoublePrecisionReal(v.into()),
                ))
            }
            String => {
                let raw = unsafe { ffi::bt_field_string_get_value(self.field) };
                if let Some(v) = util::opt_owned_cstr(raw)? {
                    Some(OwnedField::Scalar(None, ScalarField::String(v)))
                } else {
                    log::trace!("Skipping empty field string");
                    None
                }
            }
            UnsignedEnumeration => {
                let v = unsafe { ffi::bt_field_integer_unsigned_get_value(self.field) };
                let l = unsafe {
                    let mut labels = ptr::null();
                    let mut count = 0;
                    ffi::bt_field_enumeration_unsigned_get_mapping_labels(
                        self.field,
                        &mut labels,
                        &mut count,
                    )
                    .capi_result()?;
                    let labels_slice = if count == 0 || labels.is_null() {
                        &[]
                    } else {
                        slice::from_raw_parts(labels, count as _)
                    };
                    let mut labels_storage = BTreeSet::new();
                    for cstr in labels_slice.iter() {
                        if let Some(label_string) = util::opt_owned_cstr(*cstr)? {
                            labels_storage.insert(label_string);
                        }
                    }
                    labels_storage
                };
                Some(OwnedField::Scalar(
                    None,
                    ScalarField::UnsignedEnumeration(v, l),
                ))
            }
            SignedEnumeration => {
                let v = unsafe { ffi::bt_field_integer_signed_get_value(self.field) };
                let l = unsafe {
                    let mut labels = ptr::null();
                    let mut count = 0;
                    ffi::bt_field_enumeration_signed_get_mapping_labels(
                        self.field,
                        &mut labels,
                        &mut count,
                    )
                    .capi_result()?;
                    let labels_slice = if count == 0 || labels.is_null() {
                        &[]
                    } else {
                        slice::from_raw_parts(labels, count as _)
                    };
                    let mut labels_storage = BTreeSet::new();
                    for cstr in labels_slice.iter() {
                        if let Some(label_string) = util::opt_owned_cstr(*cstr)? {
                            labels_storage.insert(label_string);
                        }
                    }
                    labels_storage
                };
                Some(OwnedField::Scalar(
                    None,
                    ScalarField::SignedEnumeration(v, l),
                ))
            }
            Structure => {
                let num_members =
                    unsafe { ffi::bt_field_class_structure_get_member_count(self.class) };
                if num_members > 0 {
                    let mut members = Vec::new();
                    for midx in 0..num_members {
                        let mclass = unsafe {
                            ffi::bt_field_class_structure_borrow_member_by_index_const(
                                self.class, midx,
                            )
                        };
                        let mfield = unsafe {
                            ffi::bt_field_structure_borrow_member_field_by_index_const(
                                self.field, midx,
                            )
                        };
                        let mtype =
                            FieldType::from_raw(unsafe { ffi::bt_field_get_class_type(mfield) });
                        if !mtype.is_supported() {
                            log::trace!(
                                "Skipping unsupported structure member field type {:?}",
                                mtype
                            );
                        } else {
                            let mname_cstr =
                                unsafe { ffi::bt_field_class_structure_member_get_name(mclass) };

                            if let Some(mut f) = Field::from_raw(mfield)
                                .map(|f| f.to_owned())
                                .transpose()?
                                .flatten()
                            {
                                if let OwnedField::Scalar(n, _v) = &mut f {
                                    *n = util::opt_owned_cstr(mname_cstr)?;
                                }

                                members.push(f);
                            }
                        }
                    }

                    // We may have discarded unsupported fields
                    if members.is_empty() {
                        None
                    } else {
                        Some(OwnedField::Structure(members))
                    }
                } else {
                    None
                }
            }
            Unsupported(typ) => {
                log::trace!("Skipping unsupported field type {}", typ);
                None
            }
        })
    }
}

// NOTE: we only support a subset of the available field types
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FieldType {
    Bool,
    UnsignedInteger,
    SignedInteger,
    SinglePrecisionReal,
    DoublePrecisionReal,
    String,
    UnsignedEnumeration,
    SignedEnumeration,
    Structure,
    Unsupported(ffi::bt_field_class_type::Type),
}

impl FieldType {
    pub(crate) fn from_raw(raw: ffi::bt_field_class_type::Type) -> Self {
        use ffi::bt_field_class_type::*;
        use FieldType::*;
        match raw {
            BT_FIELD_CLASS_TYPE_BOOL => Bool,
            BT_FIELD_CLASS_TYPE_UNSIGNED_INTEGER => UnsignedInteger,
            BT_FIELD_CLASS_TYPE_SIGNED_INTEGER => SignedInteger,
            BT_FIELD_CLASS_TYPE_SINGLE_PRECISION_REAL => SinglePrecisionReal,
            BT_FIELD_CLASS_TYPE_DOUBLE_PRECISION_REAL => DoublePrecisionReal,
            BT_FIELD_CLASS_TYPE_STRING => String,
            BT_FIELD_CLASS_TYPE_UNSIGNED_ENUMERATION => UnsignedEnumeration,
            BT_FIELD_CLASS_TYPE_SIGNED_ENUMERATION => SignedEnumeration,
            BT_FIELD_CLASS_TYPE_STRUCTURE => Structure,
            _ => Unsupported(raw),
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, FieldType::Unsupported(_))
    }
}

/// Owned version of a field and its class name (field name, field value)
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum OwnedField {
    Scalar(Option<String>, ScalarField),
    // TODO: in the future, call this Container, one of structure, array, option, variant
    Structure(Vec<OwnedField>),
}

impl fmt::Display for OwnedField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OwnedField::*;
        match self {
            Scalar(n, v) => write!(
                f,
                "{} = {}",
                n.as_ref().map(|s| s.as_str()).unwrap_or("<anonymous>"),
                v
            ),
            Structure(fields) => write!(
                f,
                "{}",
                fields
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

/// Owned version of a scalar field
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ScalarField {
    Bool(bool),
    UnsignedInteger(u64),
    SignedInteger(i64),
    SinglePrecisionReal(OrderedFloat<f32>),
    DoublePrecisionReal(OrderedFloat<f64>),
    String(String),
    UnsignedEnumeration(u64, BTreeSet<String>),
    SignedEnumeration(i64, BTreeSet<String>),
}

impl fmt::Display for ScalarField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScalarField::*;
        match self {
            Bool(v) => write!(f, "{}", v),
            UnsignedInteger(v) => write!(f, "{}", v),
            SignedInteger(v) => write!(f, "{}", v),
            SinglePrecisionReal(v) => write!(f, "{}", v),
            DoublePrecisionReal(v) => write!(f, "{}", v),
            String(v) => write!(f, "'{}'", v),
            UnsignedEnumeration(v, l) => write!(
                f,
                "([{}] : container = {})",
                l.iter()
                    .map(|label| format!("'{}'", label))
                    .collect::<Vec<std::string::String>>()
                    .join(", "),
                v
            ),
            SignedEnumeration(v, l) => write!(
                f,
                "([{}] : container = {})",
                l.iter()
                    .map(|label| format!("'{}'", label))
                    .collect::<Vec<std::string::String>>()
                    .join(", "),
                v
            ),
        }
    }
}
