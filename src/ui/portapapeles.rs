//! El portapapeles de Windows, llamado directamente con la API del sistema (sin librerías).
//!
//! Copiar es un pequeño trámite con Windows: (1) pedir memoria "global" que Windows pueda
//! guardar aunque Farol se cierre, (2) escribir el texto ahí en UTF-16, que es el formato de
//! texto de Windows, (3) abrir el portapapeles, vaciarlo y entregarle esa memoria. Desde ese
//! momento la memoria es de Windows, no nuestra: por eso no se libera si todo salió bien.

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;

    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    // Las firmas tal como las documenta Microsoft. `unsafe extern` porque Rust no puede revisar
    // lo que pasa del otro lado: si una firma estuviera mal, el programa se rompería.
    #[link(name = "user32")]
    unsafe extern "system" {
        fn OpenClipboard(ventana: *mut c_void) -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(formato: u32, memoria: *mut c_void) -> *mut c_void;
        fn CloseClipboard() -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalAlloc(banderas: u32, bytes: usize) -> *mut c_void;
        fn GlobalLock(memoria: *mut c_void) -> *mut c_void;
        fn GlobalUnlock(memoria: *mut c_void) -> i32;
        fn GlobalFree(memoria: *mut c_void) -> *mut c_void;
    }

    pub fn copiar(texto: &str) -> Result<(), &'static str> {
        // Windows separa las líneas con "\r\n"; se termina con un 0 de 16 bits.
        let unidades: Vec<u16> =
            texto.replace("\r\n", "\n").replace('\n', "\r\n").encode_utf16().chain([0]).collect();
        let bytes = unidades.len() * 2;
        // SAFETY: cada llamada sigue el contrato documentado de la API de Windows: la memoria se
        // bloquea antes de escribir en ella, se escribe exactamente `bytes` bytes, se desbloquea
        // antes de entregarla, y solo se libera si Windows no se quedó con ella.
        unsafe {
            let memoria = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if memoria.is_null() {
                return Err("no hubo memoria para el portapapeles");
            }
            let destino = GlobalLock(memoria) as *mut u16;
            if destino.is_null() {
                GlobalFree(memoria);
                return Err("no se pudo escribir la memoria del portapapeles");
            }
            std::ptr::copy_nonoverlapping(unidades.as_ptr(), destino, unidades.len());
            GlobalUnlock(memoria);

            if OpenClipboard(std::ptr::null_mut()) == 0 {
                GlobalFree(memoria);
                return Err("otro programa tiene abierto el portapapeles");
            }
            EmptyClipboard();
            let entregado = !SetClipboardData(CF_UNICODETEXT, memoria).is_null();
            CloseClipboard();
            if !entregado {
                GlobalFree(memoria);
                return Err("Windows no aceptó el texto");
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
pub use windows::copiar;

#[cfg(not(windows))]
pub fn copiar(_texto: &str) -> Result<(), &'static str> {
    Err("el portapapeles solo está implementado para Windows")
}
