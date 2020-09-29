use crate::common_ffi::*;
use crate::namespaces::*;
use namespaces::{EnumMemberValue, Namespaces, Opcode};

#[no_mangle]
pub extern "C" fn classes_new() -> *mut Namespaces {
    ptr_new(Namespaces::new())
}

#[no_mangle]
pub unsafe extern "C" fn classes_load_file(c: *mut Namespaces, file_name: PChar) -> bool {
    if let Some(p) = c.as_mut() {
        if let Ok(_) = p.load_classes(pchar_to_str(file_name)) {
            return true;
        }
    };
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn classes_find_by_opcode(
    c: *mut Namespaces,
    opcode: u16,
    index: *mut i32,
) -> bool {
    if let Some(p) = c.as_mut() {
        if let Some(&i) = p.get_opcode_index_by_opcode(opcode) {
            *index = i as i32;
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn classes_get_opcode(
    c: *mut Namespaces,
    index: i32,
    out: *mut &Opcode,
) -> bool {
    if let Some(ptr) = c.as_mut() {
        if let Some(op) = ptr.opcodes.get(index as usize) {
            *out = op;
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn classes_get_enum_name_by_value_i32(
    c: *mut Namespaces,
    enum_name: PChar,
    value: i32,
    out: *mut PChar,
) -> bool {
    if let Some(ptr) = c.as_mut() {
        if let Some(name) = ptr.get_anonymous_enum_name_by_member_value(
            pchar_to_str(enum_name),
            &EnumMemberValue::Int(value),
        ) {
            *out = name.as_ptr();
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn classes_free(ptr: *mut Namespaces) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use namespaces::{EnumMemberValue, OpcodeType};
    use std::ffi::CString;

    #[test]
    fn test1() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_one_class.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);

        assert_eq!(f.opcodes.len(), 1);
        let &i = f.map_op_by_id.get(&1).unwrap();
        let op = &f.opcodes.get(i).unwrap();

        let name = op.name.clone().into_string().unwrap();
        assert_eq!(name, "TEST.m");
        assert_eq!(op.id, 1);
        assert!(matches!(op.op_type, OpcodeType::Method));
        assert_eq!(op.help_code, 0);
        // assert_eq!(String::from(op.params.get(0).unwrap()), "p: Boolean");

        let op = f.get_opcode_by_index(0).unwrap();
        let name = op.name.clone().into_string().unwrap();
        assert_eq!(name, "TEST.m");

        let op_index = f.get_opcode_index_by_opcode(1);
        assert_eq!(op_index, Some(&0));

        assert!(f.get_class_method_index_by_name("TEST", "M").is_some());
    }

    #[test]
    fn test_classes_load() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_empty.db");
        assert!(content.is_ok(), content.unwrap_err());
    }

    #[test]
    fn test_only_classes() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_only_classes.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 2);
    }

    #[test]
    fn test_one_ignore_class() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_one_ignore_class.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
    }

    #[test]
    fn test_prop() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
        assert_eq!(f.opcodes.len(), 3);
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
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
        assert_eq!(f.opcodes.len(), 0);
    }

    #[test]
    fn test_many() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_many.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 28);
        assert_eq!(f.opcodes.len(), 944); //wrong
    }

    #[test]
    fn test_enum() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop_hint.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
        assert_eq!(f.opcodes.len(), 4);

        let op = f.opcodes.get(0).unwrap();
        // assert_eq!(
        //     op.params
        //         .iter()
        //         .map(|x| String::from(x))
        //         .collect::<Vec<_>>()
        //         .join("; "),
        //     "p: Integer; Value: Extended"
        // );

        assert!(!f.get_opcode_param_at(0, 0).unwrap().is_enum);
        assert!(f.get_opcode_param_at(0, 1).unwrap().is_enum);

        // get value by full name
        let enum_val = f.get_enum_value_by_name("TeST.Method.1", "b").unwrap();

        assert!(match enum_val {
            EnumMemberValue::Int(10) => true,
            _ => false,
        });

        // get value by name for anonym enum
        let enum_val = f
            .get_anonymous_enum_value_by_member_name(0, 1, "B")
            .unwrap();
        assert!(match enum_val {
            EnumMemberValue::Int(10) => true,
            _ => false,
        });

        assert_eq!(
            f.get_anonymous_enum_name_by_member_value("TeST.Method.1", &EnumMemberValue::Int(10)),
            Some(&CString::new("B").unwrap())
        );

        // let op = f.opcodes.get(1).unwrap();
        // assert_eq!(String::from(op.params.get(0).unwrap()), "Value: Type");

        // let op = f.opcodes.get(2).unwrap();
        // assert_eq!(String::from(op.params.get(0).unwrap()), "Value: Unknown");

        // let op = f.opcodes.get(3).unwrap();
        // assert_eq!(String::from(op.params.get(0).unwrap()), "_: boolean");
    }

    #[test]
    fn test_find() {
        unsafe {
            let f = classes_new();
            let loaded = classes_load_file(f, str_to_pchar("src/namespaces/test/classes_find.db"));
            assert!(loaded);

            let mut index = -1;
            let found = classes_find_by_opcode(f, 1, &mut index);

            assert!(found);
            assert_eq!(index, 0);

            let found = classes_find_by_opcode(f, 2, &mut index);
            assert!(found);
            assert_eq!(index, 1);
            classes_free(f);
        }
    }

    #[test]
    fn test_case_ins_search() {
        let mut f = Namespaces::new();
        let content = f.load_classes("src/namespaces/test/classes_prop2.db");
        assert!(content.is_ok(), content.unwrap_err());

        let enum_val =
            f.get_anonymous_enum_name_by_member_value("TEST.HEALTH.0", &EnumMemberValue::Int(11));
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
        });

        let enum_val = f.get_anonymous_enum_value_by_member_name(0, 0, "PEDTYPE11");
        assert!(match enum_val {
            Some(&EnumMemberValue::Int(11)) => true,
            _ => false,
        });
    }
}
