use std::ffi::c_void;

use super::*;

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum OptLevel {
    None,

    #[default]
    Some,

    Full,
}

impl TryFrom<u8> for OptLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OptLevel::None),
            1 => Ok(OptLevel::Some),
            2 => Ok(OptLevel::Full),
            _ => Err(()),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum DebugLevel {
    None,

    #[default]
    Some,

    Full,
}

impl TryFrom<u8> for DebugLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DebugLevel::None),
            1 => Ok(DebugLevel::Some),
            2 => Ok(DebugLevel::Full),
            _ => Err(()),
        }
    }
}

#[derive(Default, Clone)]
pub struct Compiler {
    opt_level: OptLevel,
    dbg_level: DebugLevel,
}

impl Compiler {
    pub fn with_opt_level(mut self, opt_level: OptLevel) -> Self {
        self.opt_level = opt_level;
        self
    }

    pub fn with_dbg_level(mut self, dbg_level: DebugLevel) -> Self {
        self.dbg_level = dbg_level;
        self
    }

    pub fn compile(&self, source: &[u8]) -> Bytecode {
        use std::ptr::null;

        let mut options = ffi::lua_CompileOptions {
            optimizationLevel: self.opt_level as _,
            debugLevel: self.dbg_level as _,
            typeInfoLevel: 0,
            coverageLevel: 0,
            vectorLib: null(),
            vectorCtor: null(),
            vectorType: null(),
            mutableGlobals: null(),
            userdataTypes: null(),
            librariesWithKnownMembers: null(),
            libraryMemberTypeCb: None,
            libraryMemberConstantCb: None,
            disabledBuiltins: null(),
        };

        let mut len = 0;
        let ptr = unsafe {
            ffi::luau_compile(source.as_ptr() as _, source.len(), &mut options, &mut len)
        } as _;

        Bytecode { ptr, len }
    }
}

pub struct Bytecode {
    ptr: *const u8,
    len: usize,
}

impl Bytecode {
    pub fn inner(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for Bytecode {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}
