use std::path::Path;
use std::fs;
use anyhow::Result;

pub fn create_hidden_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::fileapi::SetFileAttributesW;
        use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;
        
        let wide: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        unsafe {
            SetFileAttributesW(wide.as_ptr(), FILE_ATTRIBUTE_HIDDEN);
        }
    }
    
    Ok(())
}

pub fn set_permissions_secure(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms)?;
    }
    
    #[cfg(not(unix))]
    let _ = path; // Suppress unused variable warning on non-Unix platforms
    
    Ok(())
}

pub fn set_permissions_readonly(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o400);
        fs::set_permissions(path, perms)?;
    }
    
    #[cfg(windows)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(path, perms)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_hidden_dir() {
        let dir = tempdir().unwrap();
        let hidden_path = dir.path().join(".hidden_test");
        
        create_hidden_dir(&hidden_path).unwrap();
        assert!(hidden_path.exists());
        assert!(hidden_path.is_dir());
    }
    
    #[test]
    fn test_set_permissions_secure() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("secure_test");
        fs::create_dir(&test_dir).unwrap();
        
        set_permissions_secure(&test_dir).unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&test_dir).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o700);
        }
    }
    
    #[test]
    fn test_set_permissions_readonly() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("readonly_test.txt");
        fs::write(&test_file, "test").unwrap();
        
        set_permissions_readonly(&test_file).unwrap();
        
        let perms = fs::metadata(&test_file).unwrap().permissions();
        assert!(perms.readonly());
    }
}
