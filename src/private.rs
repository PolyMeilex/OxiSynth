pub trait HasHandle {
    type Handle;

    fn get_handle(&self) -> *const Self::Handle;

    fn get_mut_handle(&mut self) -> *mut Self::Handle;
}
