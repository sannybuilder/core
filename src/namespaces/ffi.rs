use crate::common_ffi::*;
use crate::namespaces::*;
use namespaces::Namespaces;

#[no_mangle]
pub extern "C" fn classes_new() -> *mut Namespaces {
    ptr_new(Namespaces::new())
}

#[no_mangle]
pub unsafe extern "C" fn classes_load_file(c: *mut Namespaces, file_name: PChar) -> bool {
    if let Some(p) = c.as_mut() {
        if let Ok(_) = p.load_file(pchar_to_str(file_name)) {
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
        if let Some(&i) = p.find_by_opcode(opcode) {
            *index = i as i32;
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
    use namespaces::OpcodeType;

    #[test]
    fn test1() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_one_class.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);

        assert_eq!(f.opcodes.len(), 1);
        let &i = f.map_op_by_id.get(&1).unwrap();
        let op = &f.opcodes.get(i).unwrap();

        assert_eq!(op.name, "TEST.m");
        assert_eq!(op.id, 1);
        assert!(matches!(op.r#type, OpcodeType::Method));
        assert_eq!(op.help_code, 0);
        assert_eq!(op.hint, "p: Boolean");
    }

    #[test]
    fn test_classes_load() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_empty.db");
        assert!(content.is_ok(), content.unwrap_err());
    }

    #[test]
    fn test_only_classes() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_only_classes.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 2);
    }

    #[test]
    fn test_one_ignore_class() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_one_ignore_class.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
    }

    #[test]
    fn test_prop() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_prop.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
    }

    #[test]
    fn test_invalid() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_invalid.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
        assert_eq!(f.opcodes.len(), 0);
    }

    #[test]
    fn test_many() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_many.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 28);
        assert_eq!(f.opcodes.len(), 847); //wrong
    }

    #[test]
    fn test_hint_enum() {
        let mut f = Namespaces::new();
        let content = f.load_file("src/namespaces/test/classes_prop_hint.db");
        assert!(content.is_ok(), content.unwrap_err());
        assert_eq!(f.names.len(), 1);
        assert_eq!(f.opcodes.len(), 2);

        let op = f.opcodes.get(0).unwrap();

        assert_eq!(op.hint, "Value: ^A^B");

        let op = f.opcodes.get(1).unwrap();
        assert_eq!(op.hint, "Value: Type");
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
}
