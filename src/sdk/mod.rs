use lazy_static::lazy_static;
use libloading::Library;
pub mod messages;

lazy_static! {
    pub(crate) static ref GET_WORKBOOK_HANDLE: usize = unsafe {
        let lib = Library::new(std::env::current_exe().unwrap()).unwrap();
        match lib.get::<unsafe extern "C" fn() -> usize>(b"get_wb_handle") {
            Ok(f) => f(),
            Err(e) => {
                log::error!("{}", e);
                0
            }
        }
    };
}
