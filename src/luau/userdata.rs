use std::sync::atomic::{AtomicU8, Ordering};

use super::*;

pub fn new_tag() -> u8 {
    static COUNT: AtomicU8 = AtomicU8::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed)
}

pub trait Userdata {
    fn tag() -> u8;
    fn name() -> &'static str;
    fn register(main: &Main);
}

#[macro_export]
macro_rules! userdata {
    ($udtype:ty, $($method:ident),*) => {
		impl $crate::luau::Userdata for $udtype {
            fn tag() -> u8 {
                static TAG: ::std::sync::OnceLock<u8> = ::std::sync::OnceLock::new();
                *TAG.get_or_init($crate::luau::new_tag)
            }

            fn name() -> &'static str {
                stringify!($udtype)
            }

            fn register(main: &$crate::luau::Main) {
				extern "C-unwind" fn cleanup(_: *mut $crate::luau::ffi::lua_State, ud: *mut ::std::ffi::c_void) {
					unsafe { (ud as *mut $udtype).drop_in_place() };
				}

                let tag = Self::tag();
                let stack = main.stack();

				stack.push_table();

				stack.push_string(Self::name());
				stack.table_set_raw_field(-2, c"__type");

				stack.push_table();
				$(
					let methodname = const {
						let bytes = concat!(stringify!($method), "\0").as_bytes();
						match ::std::ffi::CStr::from_bytes_with_nul(bytes) {
							Ok(cstr) => cstr,
							Err(_) => unreachable!(),
						}
					};

					stack.push_function(methodname, Self::$method);
					stack.table_set_raw_field(-2, methodname);
				),*
				stack.table_set_raw_field(-2, c"__index");

				unsafe {
					$crate::luau::ffi::lua_setuserdatadtor(stack.as_ptr(), tag as _, cleanup);
					$crate::luau::ffi::lua_setuserdatametatable(stack.as_ptr(), tag as _);
				}
            }
        }
    };
}
