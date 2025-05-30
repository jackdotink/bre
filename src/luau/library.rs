use super::*;

pub trait Library {
    fn push(stack: Stack);
}

#[macro_export]
macro_rules! library {
	($libtype:ty, $($func:ident),*) => {
		impl $crate::luau::Library for $libtype {
			fn push(stack: $crate::luau::Stack) {
				stack.push_table();

				$(
					let debugname = const {
						let bytes = concat!(stringify!($libtype), "::", stringify!($func), "\0").as_bytes();
						match ::std::ffi::CStr::from_bytes_with_nul(bytes) {
							Ok(cstr) => cstr,
							Err(_) => unreachable!(),
						}
					};

					let funcname = const {
						let bytes = concat!(stringify!($func), "\0").as_bytes();
						match ::std::ffi::CStr::from_bytes_with_nul(bytes) {
							Ok(cstr) => cstr,
							Err(_) => unreachable!(),
						}
					};

					stack.push_function(debugname, Self::$func);
					stack.table_set_raw_field(-2, funcname);
				)*


			}
		}
	};
}
