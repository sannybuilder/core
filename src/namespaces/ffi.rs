use crate::common_ffi::*;
use crate::namespaces::*;
use namespaces::Namespaces;

type PtrNs = *const Namespaces;
type PtrNsMut = *mut Namespaces;

#[no_mangle]
pub extern "C" fn classes_new() -> PtrNs {
    ptr_new(Namespaces::new())
}

#[no_mangle]
pub unsafe extern "C" fn classes_load_file(c: &mut Namespaces, file_name: PChar) -> bool {
    if let Ok(_) = c.load_file(pchar_to_str(file_name)) {
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn classes_free(ptr: PtrNsMut) {
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
        assert_eq!(f.opcodes.len(), 865);
    }
}
