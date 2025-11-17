use anyhow::Result;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};
use std::ptr;

// libnfc FFI kötések
#[repr(C)]
pub struct NfcDevice {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NfcContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NfcIso14443aInfo {
    pub abtAtqa: [u8; 2],
    pub btSak: u8,
    pub szUidLen: usize,
    pub abtUid: [u8; 10],
    pub szAtsLen: usize,
    pub abtAts: [u8; 254],
}

#[repr(C)]
pub struct NfcTarget {
    pub nm: u32, // Modulation type
    pub nti: NfcIso14443aInfo,
}

#[link(name = "nfc")]
extern "C" {
    fn nfc_init(context: *mut *mut NfcContext) -> c_int;
    fn nfc_exit(context: *mut NfcContext);
    fn nfc_open(context: *mut NfcContext, connstring: *const c_char) -> *mut NfcDevice;
    fn nfc_close(device: *mut NfcDevice);
    fn nfc_initiator_init(device: *mut NfcDevice) -> c_int;
    fn nfc_initiator_select_passive_target(
        device: *mut NfcDevice,
        nm: u32,
        init_data: *const u8,
        init_data_len: usize,
        target: *mut NfcTarget,
    ) -> c_int;
    fn nfc_initiator_transceive_bytes(
        device: *mut NfcDevice,
        tx: *const u8,
        tx_len: usize,
        rx: *mut u8,
        rx_len: usize,
        timeout: c_int,
    ) -> c_int;
    fn nfc_device_get_name(device: *const NfcDevice) -> *const c_char;
    fn nfc_strerror(device: *const NfcDevice) -> *const c_char;
}

pub struct LibnfcReader {
    context: *mut NfcContext,
    device: *mut NfcDevice,
}

impl LibnfcReader {
    pub fn new() -> Result<Self> {
        println!("  📡 LibnfcReader::new() CALLED");
        
        let mut context: *mut NfcContext = ptr::null_mut();
        let result = unsafe { nfc_init(&mut context) };
        
        if result < 0 || context.is_null() {
            anyhow::bail!("Nem sikerült inicializálni a libnfc kontextust");
        }
        
        println!("  ✅ libnfc kontextus inicializálva");
        Ok(LibnfcReader {
            context,
            device: ptr::null_mut(),
        })
    }

    pub fn connect(&mut self) -> Result<()> {
        println!("  📡 LibnfcReader::connect() CALLED");
        
        if !self.device.is_null() {
            return Ok(()); // Már csatlakoztatva van
        }
        
        // Próbáljuk meg az ACR122U-t
        let connstring = CString::new("acr122_usb:").unwrap();
        let device = unsafe { nfc_open(self.context, connstring.as_ptr()) };
        
        if device.is_null() {
            // Próbáljuk meg PC/SC-n keresztül
            let connstring_pcsc = CString::new("acr122_pcsc:").unwrap();
            let device_pcsc = unsafe { nfc_open(self.context, connstring_pcsc.as_ptr()) };
            
            if device_pcsc.is_null() {
                anyhow::bail!("Nem sikerült megnyitni az NFC olvasót. Ellenőrizd a libnfc konfigurációt!");
            }
            
            self.device = device_pcsc;
        } else {
            self.device = device;
        }
        
        let device_name = unsafe {
            let name_ptr = nfc_device_get_name(self.device);
            if name_ptr.is_null() {
                "Unknown"
            } else {
                CStr::from_ptr(name_ptr).to_str().unwrap_or("Unknown")
            }
        };
        
        println!("  ✅ Olvasó találva: {}", device_name);
        
        let result = unsafe { nfc_initiator_init(self.device) };
        if result < 0 {
            anyhow::bail!("Nem sikerült inicializálni az NFC iniciátort");
        }
        
        println!("  ✅ NFC iniciátor inicializálva");
        Ok(())
    }

    pub fn select_target(&mut self) -> Result<NfcTarget> {
        println!("  🔌 Címke csatlakoztatása...");
        
        let mut target = NfcTarget {
            nm: 0,
            nti: NfcIso14443aInfo {
                abtAtqa: [0; 2],
                btSak: 0,
                szUidLen: 0,
                abtUid: [0; 10],
                szAtsLen: 0,
                abtAts: [0; 254],
            },
        };
        
        // ISO14443A modulation
        let nm = 0x0100; // NMT_ISO14443A
        let result = unsafe {
            nfc_initiator_select_passive_target(
                self.device,
                nm,
                ptr::null(),
                0,
                &mut target,
            )
        };
        
        if result < 0 {
            anyhow::bail!("Nem sikerült csatlakozni az NFC címkéhez. Kérlek helyezd a címkét az olvasóra.");
        }
        
        println!("  ✅ Címke csatlakoztatva");
        Ok(target)
    }

    pub fn transmit(&mut self, tx: &[u8]) -> Result<Vec<u8>> {
        let mut rx = vec![0u8; 256];
        let result = unsafe {
            nfc_initiator_transceive_bytes(
                self.device,
                tx.as_ptr(),
                tx.len(),
                rx.as_mut_ptr(),
                rx.len(),
                500, // timeout ms
            )
        };
        
        if result < 0 {
            let error_msg = unsafe {
                let err_ptr = nfc_strerror(self.device);
                if err_ptr.is_null() {
                    "Unknown error"
                } else {
                    CStr::from_ptr(err_ptr).to_str().unwrap_or("Unknown error")
                }
            };
            anyhow::bail!("libnfc transceive hiba: {} (code: {})", error_msg, result);
        }
        
        rx.truncate(result as usize);
        Ok(rx)
    }

    pub fn list_readers(&self) -> Result<Vec<String>> {
        // libnfc-ben nincs közvetlen reader listázás, csak az aktív device neve
        let mut readers = Vec::new();
        
        if !self.device.is_null() {
            let device_name = unsafe {
                let name_ptr = nfc_device_get_name(self.device);
                if name_ptr.is_null() {
                    "Unknown"
                } else {
                    CStr::from_ptr(name_ptr).to_str().unwrap_or("Unknown")
                }
            };
            readers.push(device_name.to_string());
        }
        
        Ok(readers)
    }
}

impl Drop for LibnfcReader {
    fn drop(&mut self) {
        if !self.device.is_null() {
            unsafe {
                nfc_close(self.device);
            }
        }
        if !self.context.is_null() {
            unsafe {
                nfc_exit(self.context);
            }
        }
    }
}

