// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! "Manter conectado": the refresh token and the issued client id, kept in
//! the operating system credential vault; start with [`SessionVault`].
//!
//! Nothing else about the account is stored: no password, e-mail, access
//! token or conversation. The real vault is the Windows Credential Manager
//! (or the macOS Keychain); elsewhere the option is unavailable.
use std::sync::Mutex;

/// Vault service name; one item per installation.
pub const SERVICE: &str = "CAD-CRAFT";
const ACCOUNT: &str = "chatgpt";

/// What "Manter conectado" keeps between launches.
#[derive(Clone, PartialEq, Eq)]
pub struct Saved {
    pub client_id: String,
    pub refresh_token: String,
}
impl std::fmt::Debug for Saved {
    // Never print secrets.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Saved").finish_non_exhaustive()
    }
}
impl Saved {
    fn to_json(&self) -> String {
        serde_json::json!({ "client_id": self.client_id, "refresh_token": self.refresh_token })
            .to_string()
    }
    fn from_json(text: &str) -> Option<Self> {
        let json: serde_json::Value = serde_json::from_str(text).ok()?;
        Some(Self {
            client_id: json["client_id"].as_str()?.to_owned(),
            refresh_token: json["refresh_token"].as_str()?.to_owned(),
        })
    }
}

/// Storage for the kept session. Tests use [`MemoryVault`].
pub trait SessionVault: Send + Sync {
    /// The vault exists on this system.
    fn available(&self) -> bool;
    fn load(&self) -> Result<Option<Saved>, String>;
    fn save(&self, saved: &Saved) -> Result<(), String>;
    /// Removes the item; succeeds when there is none.
    fn clear(&self) -> Result<(), String>;
}

/// The operating system vault (Windows Credential Manager, macOS Keychain).
#[derive(Default)]
pub struct OsVault;

#[cfg(any(windows, target_os = "macos"))]
impl OsVault {
    fn entry() -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| format!("cofre de credenciais: {e}"))
    }
}

#[cfg(any(windows, target_os = "macos"))]
impl SessionVault for OsVault {
    fn available(&self) -> bool {
        Self::entry().is_ok()
    }
    fn load(&self) -> Result<Option<Saved>, String> {
        match Self::entry()?.get_password() {
            Ok(text) => Ok(Saved::from_json(&text)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("cofre de credenciais: {e}")),
        }
    }
    fn save(&self, saved: &Saved) -> Result<(), String> {
        Self::entry()?
            .set_password(&saved.to_json())
            .map_err(|e| format!("cofre de credenciais: {e}"))
    }
    fn clear(&self) -> Result<(), String> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("cofre de credenciais: {e}")),
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
impl SessionVault for OsVault {
    fn available(&self) -> bool {
        false
    }
    fn load(&self) -> Result<Option<Saved>, String> {
        Ok(None)
    }
    fn save(&self, _: &Saved) -> Result<(), String> {
        Err("cofre de credenciais indisponível neste sistema".into())
    }
    fn clear(&self) -> Result<(), String> {
        Ok(())
    }
}

/// In-memory vault for tests.
pub struct MemoryVault {
    pub saved: Mutex<Option<Saved>>,
    pub available: bool,
}
impl Default for MemoryVault {
    fn default() -> Self {
        Self {
            saved: Mutex::new(None),
            available: true,
        }
    }
}
impl MemoryVault {
    pub fn current(&self) -> Option<Saved> {
        self.saved.lock().ok().and_then(|s| s.clone())
    }
}
impl SessionVault for MemoryVault {
    fn available(&self) -> bool {
        self.available
    }
    fn load(&self) -> Result<Option<Saved>, String> {
        Ok(self.current())
    }
    fn save(&self, saved: &Saved) -> Result<(), String> {
        *self.saved.lock().map_err(|_| "cofre indisponível")? = Some(saved.clone());
        Ok(())
    }
    fn clear(&self) -> Result<(), String> {
        *self.saved.lock().map_err(|_| "cofre indisponível")? = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_round_trips_as_json_and_hides_secrets() {
        let s = Saved {
            client_id: "oaiapp_1".into(),
            refresh_token: "segredo".into(),
        };
        assert_eq!(Saved::from_json(&s.to_json()), Some(s.clone()));
        assert!(!format!("{s:?}").contains("segredo"));
        assert_eq!(Saved::from_json("{}"), None);
    }
}
