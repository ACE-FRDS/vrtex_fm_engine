use std::path::PathBuf;

const OPENAI_KEY_FILE: &str = "openai-api-key.dpapi";

#[derive(Debug, Clone)]
pub struct CredentialStore {
    directory: PathBuf,
}

impl CredentialStore {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn save_openai_key(&self, api_key: &str) -> Result<(), String> {
        let key = api_key.trim();
        if key.is_empty() {
            return Err("OpenAI APIキーを入力してください".to_owned());
        }
        std::fs::create_dir_all(&self.directory)
            .map_err(|error| format!("資格情報保存先を作成できません: {error}"))?;
        let encrypted = protect_for_current_user(key.as_bytes())?;
        std::fs::write(self.openai_key_path(), encrypted)
            .map_err(|error| format!("OpenAI APIキーを保存できません: {error}"))
    }

    pub fn load_openai_key(&self) -> Result<Option<String>, String> {
        let path = self.openai_key_path();
        if !path.exists() {
            return Ok(None);
        }
        let encrypted = std::fs::read(path)
            .map_err(|error| format!("保存済みOpenAI APIキーを読み込めません: {error}"))?;
        let decrypted = unprotect_for_current_user(&encrypted)?;
        let key = String::from_utf8(decrypted)
            .map_err(|_| "保存済みOpenAI APIキーの形式が正しくありません".to_owned())?;
        let key = key.trim().to_owned();
        Ok((!key.is_empty()).then_some(key))
    }

    pub fn delete_openai_key(&self) -> Result<bool, String> {
        let path = self.openai_key_path();
        if !path.exists() {
            return Ok(false);
        }
        std::fs::remove_file(path)
            .map_err(|error| format!("保存済みOpenAI APIキーを削除できません: {error}"))?;
        Ok(true)
    }

    pub fn resolve_openai_key(&self) -> Result<Option<(String, &'static str)>, String> {
        if let Some(key) = self.load_openai_key()? {
            return Ok(Some((key, "Windows保護ストレージ")));
        }
        let environment_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        Ok(environment_key.map(|key| (key, "起動環境（OPENAI_API_KEY）")))
    }

    fn openai_key_path(&self) -> PathBuf {
        self.directory.join(OPENAI_KEY_FILE)
    }
}

#[cfg(windows)]
fn protect_for_current_user(value: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr;
    use std::slice;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: value.len() as u32,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    let success = unsafe {
        CryptProtectData(
            &input,
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if success == 0 {
        return Err(format!(
            "WindowsによるAPIキーの暗号化に失敗しました: {}",
            std::io::Error::last_os_error()
        ));
    }
    let protected =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe { LocalFree(output.pbData.cast()) };
    Ok(protected)
}

#[cfg(windows)]
fn unprotect_for_current_user(value: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr;
    use std::slice;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: value.len() as u32,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    let success = unsafe {
        CryptUnprotectData(
            &input,
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if success == 0 {
        return Err(format!(
            "WindowsによるAPIキーの復号に失敗しました: {}",
            std::io::Error::last_os_error()
        ));
    }
    let unprotected =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe { LocalFree(output.pbData.cast()) };
    Ok(unprotected)
}

#[cfg(not(windows))]
fn protect_for_current_user(_value: &[u8]) -> Result<Vec<u8>, String> {
    Err("Windows保護ストレージはWindowsでのみ利用できます".to_owned())
}

#[cfg(not(windows))]
fn unprotect_for_current_user(_value: &[u8]) -> Result<Vec<u8>, String> {
    Err("Windows保護ストレージはWindowsでのみ利用できます".to_owned())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn protects_and_restores_secret_for_current_user() {
        let secret = b"test-secret-that-must-not-be-plain-text";
        let encrypted = protect_for_current_user(secret).expect("protect");
        assert_ne!(encrypted, secret);
        assert_eq!(
            unprotect_for_current_user(&encrypted).expect("unprotect"),
            secret
        );
    }
}
