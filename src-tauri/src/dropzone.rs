//! Destino OLE propio para arrastrar archivos al notch.
//!
//! El destino de arrastre que registra wry solo vive en una de las ventanas hijas de WebView2 y, en una
//! ventana que cambia de estilos (click-through, ventana de herramientas) y se muestra/oculta, no recibía
//! nada: el cursor mostraba «no permitido» y el archivo no llegaba. Aquí se registra un destino propio en la
//! ventana y en todas sus hijas (el sistema usa el más cercano al cursor) y se reenvía al frontend con el
//! evento `notch-drag` `{ label, kind, paths }`.

use std::{
    cell::Cell,
    collections::HashMap,
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    sync::Mutex,
};
use tauri::{AppHandle, Emitter, WebviewWindow};
use windows::{
    core::{implement, BOOL},
    Win32::{
        Foundation::{HWND, LPARAM, POINTL},
        System::{
            Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL},
            Ole::{
                IDropTarget, IDropTarget_Impl, ReleaseStgMedium, RegisterDragDrop, RevokeDragDrop, CF_HDROP,
                DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
            },
            SystemServices::MODIFIERKEYS_FLAGS,
        },
        UI::{
            Shell::{DragQueryFileW, HDROP},
            WindowsAndMessaging::{EnumChildWindows, GetPropW, IsWindow},
        },
    },
};

#[derive(Clone, serde::Serialize)]
struct DragEvent {
    label: String,
    kind: &'static str,
    paths: Vec<String>,
}

#[implement(IDropTarget)]
struct NotchDropTarget {
    app: AppHandle,
    label: String,
    /// Si lo que se arrastra trae archivos (se decide en DragEnter).
    files: Cell<bool>,
}

impl NotchDropTarget {
    fn emit(&self, kind: &'static str, paths: Vec<String>) {
        let _ = self.app.emit("notch-drag", DragEvent { label: self.label.clone(), kind, paths });
    }
}

/// Rutas de los archivos del objeto arrastrado; `None` si no trae archivos.
fn paths_of(data: &IDataObject) -> Option<Vec<String>> {
    let fmt = FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    unsafe {
        let mut medium = data.GetData(&fmt).ok()?;
        let hdrop = HDROP(medium.u.hGlobal.0 as _);
        let count = DragQueryFileW(hdrop, 0xFFFF_FFFF, None);
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count {
            let len = DragQueryFileW(hdrop, i, None) as usize;
            let mut buf = vec![0u16; len + 1];
            DragQueryFileW(hdrop, i, Some(&mut buf));
            out.push(PathBuf::from(OsString::from_wide(&buf[..len])).to_string_lossy().into_owned());
        }
        ReleaseStgMedium(&mut medium);
        Some(out)
    }
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for NotchDropTarget_Impl {
    fn DragEnter(
        &self,
        data: windows::core::Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        let paths = data.as_ref().and_then(paths_of);
        let ok = paths.is_some();
        self.files.set(ok);
        unsafe { *effect = if ok { DROPEFFECT_COPY } else { DROPEFFECT_NONE } };
        if ok {
            self.emit("enter", paths.unwrap_or_default());
        }
        Ok(())
    }

    fn DragOver(&self, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows::core::Result<()> {
        unsafe { *effect = if self.files.get() { DROPEFFECT_COPY } else { DROPEFFECT_NONE } };
        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        if self.files.replace(false) {
            self.emit("leave", vec![]);
        }
        Ok(())
    }

    fn Drop(
        &self,
        data: windows::core::Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        let was = self.files.replace(false);
        let paths = if was { data.as_ref().and_then(paths_of) } else { None };
        unsafe { *effect = if paths.is_some() { DROPEFFECT_COPY } else { DROPEFFECT_NONE } };
        match paths {
            Some(p) => self.emit("drop", p),
            None => self.emit("leave", vec![]),
        }
        Ok(())
    }
}

/// Destinos registrados: etiqueta de ventana → (ventana hija → puntero de nuestro IDropTarget).
static TARGETS: Mutex<Option<HashMap<String, HashMap<isize, isize>>>> = Mutex::new(None);

unsafe fn registered_ptr(hwnd: HWND) -> isize {
    GetPropW(hwnd, windows::core::w!("OleDropTargetInterface")).0 as isize
}

/// Registra (o repone) nuestro destino en la ventana del notch y en todas sus hijas. Debe llamarse en el
/// hilo de la interfaz (los destinos OLE viven en el hilo que los registra); es barato y se repite cada segundo
/// porque WebView2 crea y reemplaza ventanas hijas por su cuenta.
pub fn ensure(app: &AppHandle, window: &WebviewWindow) {
    let label = window.label().to_string();
    let Ok(top) = window.hwnd().map(|h| h.0 as isize) else { return };
    let app2 = app.clone();
    let _ = window.run_on_main_thread(move || unsafe {
        let mut guard = TARGETS.lock().unwrap();
        let map = guard.get_or_insert_with(HashMap::new).entry(label.clone()).or_default();

        let top = HWND(top as _);
        let mut hwnds: Vec<HWND> = vec![top];
        unsafe extern "system" fn collect(h: HWND, l: LPARAM) -> BOOL {
            (*(l.0 as *mut Vec<HWND>)).push(h);
            BOOL(1)
        }
        let _ = EnumChildWindows(Some(top), Some(collect), LPARAM(&mut hwnds as *mut Vec<HWND> as isize));

        // Las que ya no existen se olvidan; las demás se reponen si alguien (wry, Chromium) cambió su destino.
        map.retain(|k, _| IsWindow(Some(HWND(*k as _))).as_bool());
        let pending: Vec<HWND> =
            hwnds.into_iter().filter(|h| map.get(&(h.0 as isize)).copied() != Some(registered_ptr(*h)) || registered_ptr(*h) == 0).collect();
        if pending.is_empty() {
            return;
        }
        let target: IDropTarget = NotchDropTarget { app: app2, label, files: Cell::new(false) }.into();
        for h in pending {
            let _ = RevokeDragDrop(h);
            if RegisterDragDrop(h, &target).is_ok() {
                map.insert(h.0 as isize, registered_ptr(h));
            }
        }
    });
}
