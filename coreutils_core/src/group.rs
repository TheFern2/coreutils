//! A module do deal more easily with UNIX groups.

use std::{ffi::CString, io, ptr};

use libc::{getegid, getgrgid, getgrgid_r, getgrnam, getgroups, gid_t};

use bstr::{BStr, BString};

/// Group ID type.
pub type Gid = gid_t;

/// Enum that holds possible errors while creating `Group` type.
#[derive(Debug)]
pub enum GroupError {
    /// Happens when `getgrgid_r` or `getgrnam_r` fails.
    ///
    /// It holds the error code of these function return.
    GetGroupFailed(i32),
    /// Happens when the pointer to the `group.gr_name` is NULL.
    NameCheckFailed,
    /// Happens when the pointer to the `group.gr_passwd` is NULL.
    PasswdCheckFailed,
    /// Happens when the pointer to the `group.gr_mem` is NULL.
    MemCheckFailed,
    /// Happens when the pointer of `group` primitive is NULL.
    ///
    /// This can happen even if `getgrgid_r` or `getgrnam_r` return 0.
    GroupNotFound,
}

/// This struct holds information about a group of UNIX/UNIX-like systems.
///
/// Contains `sys/types.h` `group` struct attributes as Rust more common types.
// It also contains a pointer to the libc::group type for more complex manipulations.
#[derive(Clone, Debug)]
pub struct Group {
    /// Group name.
    name: BString,
    /// Group ID.
    id: Gid,
    /// Group encrypted password
    passwd: BString,
    /// Group list of members
    mem: BString,
    // gr: *mut group
}

impl Group {
    /// Creates a new `Group` getting the user group as default.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `GroupError` wrapped in a `Err`.
    pub fn new() -> Result<Self, GroupError> {
        let mut gr = unsafe { std::mem::zeroed() };
        let mut gr_ptr = ptr::null_mut();
        let mut buff = [0; 16384]; // Got this from manual page about `getgrgid_r`.

        let res: i32;
        unsafe {
            res = getgrgid_r(getegid(), &mut gr, &mut buff[0], buff.len(), &mut gr_ptr);
        }

        if res != 0 {
            return Err(GroupError::GetGroupFailed(res));
        }

        if gr_ptr.is_null() {
            return Err(GroupError::GroupNotFound);
        }

        let name = if !gr.gr_name.is_null() {
            let name_cstr = unsafe { CString::from_raw(gr.gr_name) };
            BString::from_slice(name_cstr.as_bytes())
        } else {
            return Err(GroupError::NameCheckFailed);
        };

        let id = gr.gr_gid;

        let passwd = if !gr.gr_passwd.is_null() {
            let passwd_cstr = unsafe { CString::from_raw(gr.gr_passwd) };
            BString::from_slice(passwd_cstr.as_bytes())
        } else {
            return Err(GroupError::PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let aux_ptr = unsafe { *gr.gr_mem };
        let mem = if !gr.gr_mem.is_null() && !aux_ptr.is_null() {
            let mem_cstr = unsafe { CString::from_raw(aux_ptr) };
            BString::from_slice(mem_cstr.as_bytes())
        } else {
            return Err(GroupError::MemCheckFailed);
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem
            // gr: &mut gr,
        })
    }

    /// Creates a `Group` using a `id` to get all attributes.
    pub fn from_gid(id: Gid) -> Result<Self, GroupError> {
        let gr = unsafe { getgrgid(id) };
        let name_ptr = unsafe { (*gr).gr_name };
        let pw_ptr = unsafe { (*gr).gr_passwd };
        let mem_ptr = unsafe { (*gr).gr_mem };

        if gr.is_null() {
            return Err(GroupError::GroupNotFound);
        }

        let name = if !name_ptr.is_null() {
            let name_cstr = unsafe { CString::from_raw(name_ptr) };
            BString::from_slice(name_cstr.as_bytes())
        } else {
            return Err(GroupError::NameCheckFailed);
        };

        let passwd = if !pw_ptr.is_null() {
            let passwd_cstr = unsafe { CString::from_raw(pw_ptr) };
            BString::from_slice(passwd_cstr.as_bytes())
        } else {
            return Err(GroupError::PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let aux_ptr = unsafe { *mem_ptr };
        let mem = if !mem_ptr.is_null() && !aux_ptr.is_null() {
            let mem_cstr = unsafe { CString::from_raw(*mem_ptr) };
            BString::from_slice(mem_cstr.as_bytes())
        } else {
            return Err(GroupError::MemCheckFailed);
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem,
            // gr,
        })
    }

    /// Creates a `Group` using a `name` to get all attributes.
    pub fn from_name(name: impl AsRef<[u8]>) -> Result<Self, GroupError> {
        let gr_name = BString::from_slice(name);
        let gr = unsafe { getgrnam((*gr_name).as_ptr() as *const i8) };
        let pw_ptr = unsafe { (*gr).gr_passwd };
        let mem_ptr = unsafe { (*gr).gr_mem };

        if gr.is_null() {
            return Err(GroupError::GroupNotFound);
        }

        let id = unsafe { (*gr).gr_gid };

        let passwd = if !pw_ptr.is_null() {
            let passwd_cstr = unsafe { CString::from_raw(pw_ptr) };
            BString::from_slice(passwd_cstr.as_bytes())
        } else {
            return Err(GroupError::PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let aux_ptr = unsafe { *mem_ptr };
        let mem = if !mem_ptr.is_null() && !aux_ptr.is_null() {
            let mem_cstr = unsafe { CString::from_raw(*mem_ptr) };
            BString::from_slice(mem_cstr.as_bytes())
        } else {
            return Err(GroupError::MemCheckFailed);
        };

        Ok(Group {
            name: gr_name,
            id,
            passwd,
            mem,
            // gr,
        })
    }

    /// Get the `Group` name.
    pub fn name(&self) -> &BStr {
        &self.name
    }

    /// Get the `Group` id.
    pub fn id(&self) -> Gid {
        self.id
    }

    /// Get the `Group` encrypted password.
    pub fn passwd(&self) -> &BStr {
        &self.passwd
    }

    /// Get the `Group` list of members.
    pub fn mem(&self) -> &BStr {
        &self.mem
    }

    // /// Get a raw pointer to the group.
    // pub fn raw_ptr(&self) -> *const group {
    //     self.gr
    // }
    //
    // // Get a mutable raw pointer to the group.
    // // Use with caution.
    // pub unsafe fn raw_ptr_mut(&mut self) -> *mut group {
    //     self.gr
    // }
}

/// Get all `Groups` in the system.
// Based of uutils get_groups
pub fn get_groups() -> io::Result<Vec<Group>> {
    // First we check if we indeed have groups.
    // "If gidsetsize is 0 (fist parameter), getgroups() returns the number of supplementary group
    // IDs associated with the calling process without modifying the array pointed to by grouplist."
    let num_groups = unsafe { getgroups(0, ptr::null_mut()) };
    if num_groups == -1 {
        return Err(io::Error::last_os_error());
    }

    let mut groups_ids = Vec::with_capacity(num_groups as usize);
    let num_groups = unsafe { getgroups(num_groups, groups_ids.as_mut_ptr()) };
    if num_groups == -1 {
        return Err(io::Error::last_os_error());
    } else {
        unsafe {
            groups_ids.set_len(num_groups as usize);
        }
    }

    let groups = {
        let mut gs = Vec::with_capacity(num_groups as usize);
        for g_id in groups_ids {
            if let Ok(gr) = Group::from_gid(g_id) {
                gs.push(gr);
            }
        }
        gs
    };

    Ok(groups)
}
