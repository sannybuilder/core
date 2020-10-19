use crate::common_ffi::*;
use crate::namespaces::namespaces::*;

#[no_mangle]
pub extern "C" fn classes_new() -> *mut Namespaces {
    ptr_new(Namespaces::new())
}

#[no_mangle]
pub unsafe extern "C" fn classes_load_file(ns: *mut Namespaces, file_name: PChar) -> bool {
    boolclosure! {{
        ns.as_mut()?.load_classes(pchar_to_str(file_name)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_load_enum_file(ns: *mut Namespaces, file_name: PChar) -> bool {
    boolclosure! {{
        ns.as_mut()?.load_enums(pchar_to_str(file_name)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_find_by_opcode(
    ns: *mut Namespaces,
    opcode: u16,
    out: *mut &Opcode,
) -> bool {
    boolclosure! {{
        let ns = ns.as_mut()?;
        let index = *ns.get_opcode_index_by_opcode(opcode)?;
        *out = ns.get_opcode_by_index(index)?;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_find_by_name(
    ns: *mut Namespaces,
    class_name: PChar,
    prop_name: PChar,
    out: *mut &Opcode,
) -> bool {
    boolclosure! {{
        let ns = ns.as_mut()?;
        let index = *ns.get_opcode_index_by_name(pchar_to_str(class_name)?, pchar_to_str(prop_name)?)?;
        *out = ns.get_opcode_by_index(index)?;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_find_by_prop(
    ns: *mut Namespaces,
    class_name: PChar,
    prop_name: PChar,
    prop_pos: u8,
    operation: PChar,
    out: *mut &Opcode,
) -> bool {
    boolclosure! {{
        let ns = ns.as_mut()?;
        let index = *ns.get_class_property_index_by_name(
            pchar_to_str(class_name)?,
            pchar_to_str(prop_name)?,
            prop_pos,
            pchar_to_str(operation)?
        )?;
        *out = ns.get_opcode_by_index(index)?;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_get_class_id_by_name(
    ns: *mut Namespaces,
    name: PChar,
    out: *mut i32,
) -> bool {
    boolclosure! {{
        *out = ns.as_mut()?.get_class_id_by_name(pchar_to_str(name)?)?;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_get_class_name_by_id(
    ns: *mut Namespaces,
    id: i32,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        *out = ns.as_mut()?.get_class_name_by_id(id)?.as_ptr();
        Some(())
    }}
}

// todo: support_non_i32_enum
#[no_mangle]
pub unsafe extern "C" fn classes_get_enum_name_by_value_i32(
    ns: *mut Namespaces,
    enum_name: PChar,
    value: i32,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        *out = ns.as_mut()?.get_enum_member_name_by_value(
            pchar_to_str(enum_name)?,
            &EnumMemberValue::Int(value),
        )?.as_ptr();
        Some(())
    }}
}

// todo: support_non_i32_enum
#[no_mangle]
pub unsafe extern "C" fn classes_get_enum_value_i32_by_name(
    ns: *mut Namespaces,
    enum_name: PChar,
    enum_member: PChar,
    out: *mut i32,
) -> bool {
    boolclosure! {{
        let value = ns.as_mut()?.get_enum_value_by_name(
            pchar_to_str(enum_name)?,
            pchar_to_str(enum_member)?,
        )?;
        match value{
            EnumMemberValue::Int(x) => { *out = *x; }
            _ => return None
        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_filter_enum_members_by_name(
    ns: *mut Namespaces,
    enum_name: PChar,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
) -> bool {
    boolclosure! {{
        let items = ns.as_mut()?.filter_enum_members_by_name(pchar_to_str(enum_name)?, pchar_to_str(needle)?)?;
        for item in items {
            dict.as_mut()?.add(item.0, item.1)
        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_filter_classes_by_name(
    ns: *mut Namespaces,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
) -> bool {
    boolclosure! {{
        let items = ns.as_mut()?.filter_classes_by_name(pchar_to_str(needle)?)?;
        for item in items {
            dict.as_mut()?.add(item.0, item.1)
        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_filter_enums_by_name(
    ns: *mut Namespaces,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
) -> bool {
    boolclosure! {{
        let items = ns.as_mut()?.filter_enums_by_name(pchar_to_str(needle)?)?;
        for item in items {
            dict.as_mut()?.add(item.0, item.1)
        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_filter_props_by_name(
    ns: *mut Namespaces,
    class_name: PChar,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_num_by_str::DictNumByStr,
) -> bool {
    boolclosure! {{
        let items = ns.as_mut()?.filter_class_props_by_name(pchar_to_str(class_name)?, pchar_to_str(needle)?)?;
        for item in items {
            dict.as_mut()?.add(item.0, item.1)
        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_has_prop(ns: *mut Namespaces, prop_name: PChar) -> bool {
    boolclosure! {{
         ns.as_mut()?.has_prop(pchar_to_str(prop_name)?).then_some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn classes_free(ns: *mut Namespaces) {
    ptr_free(ns);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test1() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_one_class.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 1);

        assert_eq!(f.op_count(), 1);
        let &i = f.get_opcode_index_by_opcode(1).unwrap();
        let op = &f.get_opcode_by_index(i).unwrap();

        let name = op.name.clone().into_string().unwrap();
        assert_eq!(name, "TEST.m");
        assert_eq!(op.id, 1);
        assert!(matches!(op.op_type, OpcodeType::Method));
        assert_eq!(op.help_code, 0);
        assert_eq!(op.hint, CString::new("\"p: Boolean\"").unwrap());

        let op = f.get_opcode_by_index(0).unwrap();
        let name = op.name.clone().into_string().unwrap();
        assert_eq!(name, "TEST.m");

        let op_index = f.get_opcode_index_by_opcode(1);
        assert_eq!(op_index, Some(&0));

        assert!(f.get_opcode_index_by_name("TEST", "M").is_some());
    }

    #[test]
    fn test_classes_load() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_empty.db");
        assert!(content.is_some());
    }

    #[test]
    fn test_only_classes() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_only_classes.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 2);
    }

    #[test]
    fn test_one_ignore_class() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_one_ignore_class.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 1);
    }

    #[test]
    fn test_prop() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 1);
        assert_eq!(f.op_count(), 3);
        let op_index = f.get_opcode_index_by_opcode(0x0226).unwrap();
        let op = f.get_opcode_by_index(*op_index).unwrap();
        let name = op.name.clone().into_string().unwrap();
        assert_eq!(name, "test.Health");

        let op_index = f
            .get_class_property_index_by_name("TEST", "HEALTH", 2, "=")
            .unwrap();
        let op = f.get_opcode_by_index(*op_index).unwrap();
        assert_eq!(op.id, 0x0226);
    }

    #[test]
    fn test_invalid() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_invalid.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 1);
        assert_eq!(f.op_count(), 0);
    }

    #[test]
    fn test_many() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_many.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 28);
        assert_eq!(f.op_count(), 971); //wrong
    }

    #[test]
    fn test_deprecated() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_deprecated.db");
        assert!(content.is_some());
        assert_eq!(f.map_enum.len(), 9);
        assert_eq!(f.op_count(), 1);
    }

    #[test]
    fn test_enum() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop_hint.db");
        assert!(content.is_some());
        assert_eq!(f.classes_count(), 1);
        assert_eq!(f.op_count(), 4);

        let op = f.get_opcode_by_index(0).unwrap();
        assert_eq!(
            op.hint,
            CString::new("\"p: Integer\" \"Value: Extended\"").unwrap()
        );

        let enum_val = f.get_enum_value_by_name("TeST.Method.1", "b").unwrap();
        assert!(match enum_val {
            EnumMemberValue::Int(10) => true,
            _ => false,
        });

        assert_eq!(
            f.get_enum_member_name_by_value("TeST.Method.1", &EnumMemberValue::Int(10)),
            Some(&CString::new("B").unwrap())
        );

        let op = f.get_opcode_by_index(1).unwrap();
        assert_eq!(op.hint, CString::new("\"Value: Type\"").unwrap());

        let op = f.get_opcode_by_index(2).unwrap();
        assert_eq!(op.hint, CString::new("\"Value: Unknown\"").unwrap());

        let op = f.get_opcode_by_index(3).unwrap();
        assert_eq!(op.hint, CString::new("\"_: boolean\"").unwrap());
    }

    #[test]
    fn test_find() {
        let mut f = Namespaces::new();
        f.load_classes("src/namespaces/test/classes_find.db");

        let index = *f.get_opcode_index_by_opcode(1).unwrap();
        let found = f.get_opcode_by_index(index).is_some();
        assert!(found);
        assert_eq!(index, 0);

        let index = *f.get_opcode_index_by_opcode(2).unwrap();
        let found = f.get_opcode_by_index(index).is_some();
        assert!(found);
        assert_eq!(index, 1);
    }

    #[test]
    fn test_case_ins_search() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop2.db");
        assert!(content.is_some());

        let enum_val = f.get_enum_member_name_by_value("TEST.HEALTH.0", &EnumMemberValue::Int(11));
        assert!(match enum_val {
            Some(x) => {
                let name = x.clone().into_string().unwrap();
                name == "PedType11"
            }
            _ => false,
        });

        let enum_val = f.get_enum_value_by_name("TEST.HEALTH.0", "PEDTYPE11");
        assert!(match enum_val {
            Some(&EnumMemberValue::Int(11)) => true,
            _ => false,
        })
    }

    #[test]
    fn test_filter_props() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop2.db");
        assert!(content.is_some());
        let items = f.filter_class_props_by_name("Test", "M");
        assert_eq!(items.unwrap().len(), 1);
    }

    #[test]
    fn test_enum_hole() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_enum_hole.db");
        assert!(content.is_some());
        let value = f.get_enum_value_by_name("Test.Method.0", "a");
        assert_eq!(value.unwrap(), &EnumMemberValue::Int(0));
        let value = f.get_enum_value_by_name("Test.Method.0", "b");
        assert_eq!(value.unwrap(), &EnumMemberValue::Int(2));

        let value = f.get_enum_member_name_by_value("Test.Method.0", &EnumMemberValue::Int(1));
        assert!(value.is_none());
    }

    #[test]
    fn test_load_enums() {
        let mut f = Namespaces::new();
        let content = f.load_enums("src/namespaces/test/enums_1.txt");
        assert!(content.is_some());

        let value = f.get_enum_value_by_name("PedType", "Medic");
        assert_eq!(value.unwrap(), &EnumMemberValue::Int(18));

        let value = f.get_enum_value_by_name("WeaponType", "Parachute");
        assert_eq!(value.unwrap(), &EnumMemberValue::Int(46));

        let value = f.get_enum_value_by_name("CarMission", "Cruise");
        assert_eq!(value.unwrap(), &EnumMemberValue::Int(1));
    }
}
