use std::ffi::c_void;
use std::mem;

/// Generic<T> represents the interaction the Cpu can make with the system T. Each function returns
/// a bool which indicates whether the state has changed in a way that means the JIT'ed code should
/// return and allow the runtime to re-evaluate.
#[derive(Copy, Clone)]
pub struct Generic<T> {
    pub read: fn(&mut T, addr: u16) -> (bool, u8),
    pub write: fn(&mut T, addr: u16, val: u8) -> bool,
    pub interrupts: fn(&mut T, enabled: bool) -> bool,
}

/// TypeErased is the type erased version of Generic that will be passed to the assembly.
#[derive(Copy, Clone)]
pub struct TypeErased {
    pub read: extern "sysv64" fn(*mut c_void, addr: u16) -> (bool, u8),
    pub write: extern "sysv64" fn(*mut c_void, addr: u16, val: u8) -> bool,
    pub interrupts: extern "sysv64" fn(*mut c_void, enabled: bool) -> bool,
}

pub struct Wrapper<'a, T> {
    generic: &'a Generic<T>,
    parameter: &'a mut T,
}

impl<T> Generic<T> {
    pub(super) fn type_erased(&self) -> TypeErased {
        type W<'a, T> = Wrapper<'a, T>;

        TypeErased {
            read: read_wrapper::<W<T>>,
            write: write_wrapper::<W<T>>,
            interrupts: interrupts_wrapper::<W<T>>,
        }
    }
}

impl<'a, T> Wrapper<'a, T> {
    pub fn new(generic: &'a Generic<T>, parameter: &'a mut T) -> Self {
        Wrapper { generic, parameter }
    }

    unsafe fn from_raw(ptr: *mut c_void) -> &'a mut Self {
        let self_ptr: *mut Self = mem::transmute(ptr);
        &mut *self_ptr
    }
}

extern "sysv64" fn read_wrapper<'a, T: 'a>(param: *mut c_void, addr: u16) -> (bool, u8) {
    let wrapper = unsafe { Wrapper::<'a, T>::from_raw(param) };
    (wrapper.generic.read)(wrapper.parameter, addr)
}

extern "sysv64" fn write_wrapper<'a, T: 'a>(param: *mut c_void, addr: u16, val: u8) -> bool {
    let wrapper = unsafe { Wrapper::<'a, T>::from_raw(param) };
    (wrapper.generic.write)(wrapper.parameter, addr, val)
}

extern "sysv64" fn interrupts_wrapper<'a, T: 'a>(param: *mut c_void, enabled: bool) -> bool {
    let wrapper = unsafe { Wrapper::<'a, T>::from_raw(param) };
    (wrapper.generic.interrupts)(wrapper.parameter, enabled)
}
