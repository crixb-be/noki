//! Read-only Shell Jump List access. Never call BeginList/CommitList: those overwrite another
//! application's list. The Shell reads AutomaticDestinations and CustomDestinations itself.
//! Private, versioned interfaces are queried by IID and fail closed; documented document lists
//! remain a fallback. ABI references: Open-Shell JumpLists.cpp and System Informer appresolverp.h.
#![allow(non_snake_case)] // Match the COM ABI names.
use std::{ffi::c_void, ptr::null_mut};
use windows::{
    core::{IUnknown, IUnknown_Vtbl, Interface, BOOL, GUID, HRESULT, HSTRING, PCWSTR, PWSTR},
    Win32::{
        Foundation::HWND,
        Storage::EnhancedStorage::PKEY_Title,
        System::Com::{
            CoCreateInstance, CoTaskMemFree, StructuredStorage::PropVariantToStringAlloc,
            CLSCTX_INPROC_SERVER,
        },
        UI::Shell::{
            ApplicationDocumentLists, Common::IObjectArray, DestinationList,
            IApplicationDocumentLists, IShellItem, IShellLinkW, PropertiesSystem::IPropertyStore,
            SHLoadIndirectString, ADLT_FREQUENT, ADLT_RECENT, SIGDN_NORMALDISPLAY,
        },
    },
};

#[windows_core::interface("e9c5ef8d-fd41-4f72-ba87-eb03bad5817c")]
unsafe trait IAutomaticList: IUnknown {
    fn Initialize(&self, app: PCWSTR, link: PCWSTR, unused: PCWSTR) -> HRESULT;
    fn HasList(&self, has: *mut BOOL) -> HRESULT;
    fn GetList(
        &self,
        kind: i32,
        count: u32,
        flags: u32,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT;
}

#[repr(C)]
#[derive(Default)]
struct Category {
    kind: i32,
    value: usize,
    count: i32,
    reserved: [i32; 10],
}

#[windows_core::interface("507101cd-f6ad-46c8-8e20-eeb9e6bac47f")]
unsafe trait ICustomList: IUnknown {
    fn SetMinItems(&self) -> HRESULT;
    fn SetApplicationID(&self, app: PCWSTR) -> HRESULT;
    fn GetSlotCount(&self) -> HRESULT;
    fn GetCategoryCount(&self, count: *mut u32) -> HRESULT;
    fn GetCategory(&self, index: u32, flags: i32, category: *mut Category) -> HRESULT;
    fn DeleteCategory(&self) -> HRESULT;
    fn EnumerateCategoryDestinations(
        &self,
        index: u32,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT;
}

#[windows_core::interface("de25675a-72de-44b4-9373-05170450c140")]
unsafe trait IAppResolver: IUnknown {
    fn GetAppIDForShortcut(&self, item: *mut c_void, id: *mut PWSTR) -> HRESULT;
    fn GetAppIDForShortcutObject(
        &self,
        link: *mut c_void,
        item: *mut c_void,
        id: *mut PWSTR,
    ) -> HRESULT;
    fn GetAppIDForWindow(
        &self,
        hwnd: HWND,
        id: *mut PWSTR,
        prevented: *mut BOOL,
        explicit: *mut BOOL,
        embedded: *mut BOOL,
    ) -> HRESULT;
    fn GetAppIDForProcess(
        &self,
        pid: u32,
        id: *mut PWSTR,
        prevented: *mut BOOL,
        explicit: *mut BOOL,
        embedded: *mut BOOL,
    ) -> HRESULT;
    fn GetShortcutForProcess(&self, pid: u32, item: *mut *mut c_void) -> HRESULT;
    fn GetBestShortcutForAppID(&self, id: PCWSTR, item: *mut *mut c_void) -> HRESULT;
    fn GetBestShortcutAndAppIDForAppPath(
        &self,
        path: PCWSTR,
        item: *mut *mut c_void,
        id: *mut PWSTR,
    ) -> HRESULT;
}

pub struct JumpGroup {
    pub name: String,
    pub items: Vec<JumpItem>,
}
pub struct JumpItem {
    pub name: String,
    pub object: IUnknown,
}

unsafe fn take_string(raw: PWSTR) -> String {
    let text = raw.to_string().unwrap_or_default();
    CoTaskMemFree(Some(raw.0.cast()));
    text
}

pub unsafe fn app_id(explicit: &str, path: &str, windows: &[isize]) -> String {
    if !explicit.is_empty() {
        return explicit.to_string();
    }
    // Explorer's executable also owns the desktop; resolving its path can return a generated
    // desktop identity rather than the file-manager identity used by AutomaticDestinations.
    if path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(path)
        .eq_ignore_ascii_case("explorer.exe")
    {
        return "Microsoft.Windows.Explorer".into();
    }
    let Ok(resolver) = CoCreateInstance::<_, IAppResolver>(
        &GUID::from_u128(0x660b90c8_73a9_4b58_8cae_355b7f55341b),
        None,
        CLSCTX_INPROC_SERVER,
    ) else {
        return String::new();
    };
    for hwnd in windows {
        let mut raw = PWSTR::null();
        if resolver
            .GetAppIDForWindow(
                HWND(*hwnd as *mut _),
                &mut raw,
                null_mut(),
                null_mut(),
                null_mut(),
            )
            .is_ok()
        {
            let id = take_string(raw);
            if !id.is_empty() {
                return id;
            }
        }
    }
    let mut raw = PWSTR::null();
    if resolver
        .GetBestShortcutAndAppIDForAppPath(
            PCWSTR(HSTRING::from(path).as_ptr()),
            null_mut(),
            &mut raw,
        )
        .is_ok()
    {
        return take_string(raw);
    }
    String::new()
}

unsafe fn indirect(text: &str) -> String {
    let mut buffer = [0u16; 2048];
    if SHLoadIndirectString(&HSTRING::from(text), &mut buffer, None).is_ok() {
        return String::from_utf16_lossy(
            &buffer[..buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len())],
        );
    }
    text.to_string()
}

unsafe fn item_name(object: &IUnknown) -> Option<String> {
    if let Ok(item) = object.cast::<IShellItem>() {
        return item
            .GetDisplayName(SIGDN_NORMALDISPLAY)
            .ok()
            .map(|raw| take_string(raw));
    }
    let link = object.cast::<IShellLinkW>().ok()?;
    if let Ok(store) = link.cast::<IPropertyStore>() {
        if let Ok(value) = store.GetValue(&PKEY_Title) {
            if let Ok(raw) = PropVariantToStringAlloc(&value) {
                let name = indirect(&take_string(raw));
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    let mut description = [0u16; 2048];
    let _ = link.GetDescription(&mut description);
    let length = description
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(description.len());
    (length > 0).then(|| String::from_utf16_lossy(&description[..length]))
}

unsafe fn collection(array: &IObjectArray) -> Vec<JumpItem> {
    (0..array.GetCount().unwrap_or(0).min(100))
        .filter_map(|index| {
            let object: IUnknown = array.GetAt(index).ok()?;
            let name = item_name(&object)?;
            Some(JumpItem { name, object })
        })
        .collect()
}

pub unsafe fn read(app: &str) -> Vec<JumpGroup> {
    if app.is_empty() {
        return Vec::new();
    }
    let id = HSTRING::from(app);
    let mut groups = Vec::new();
    let automatic = CoCreateInstance::<_, IAutomaticList>(
        &GUID::from_u128(0xf0ae1542_f497_484b_a175_a20db09144ba),
        None,
        CLSCTX_INPROC_SERVER,
    )
    .ok()
    .filter(|list| {
        list.Initialize(PCWSTR(id.as_ptr()), PCWSTR::null(), PCWSTR::null())
            .is_ok()
    });
    let documented = CoCreateInstance::<_, IApplicationDocumentLists>(
        &ApplicationDocumentLists,
        None,
        CLSCTX_INPROC_SERVER,
    )
    .ok()
    .filter(|list| list.SetAppID(&id).is_ok());
    for (kind, name) in [(0, "Anclado"), (1, "Recientes"), (2, "Frecuentes")] {
        let mut raw = null_mut();
        let array = automatic
            .as_ref()
            .and_then(|list| {
                list.GetList(kind, 30, 1, &IObjectArray::IID, &mut raw)
                    .ok()
                    .ok()?;
                (!raw.is_null()).then(|| IObjectArray::from_raw(raw))
            })
            .or_else(|| {
                if kind == 0 {
                    return None;
                }
                documented
                    .as_ref()?
                    .GetList::<IObjectArray>(
                        if kind == 1 {
                            ADLT_RECENT
                        } else {
                            ADLT_FREQUENT
                        },
                        30,
                    )
                    .ok()
            });
        if let Some(array) = array {
            let items = collection(&array);
            if !items.is_empty() {
                groups.push(JumpGroup {
                    name: name.into(),
                    items,
                });
            }
        }
    }
    if let Ok(custom) =
        CoCreateInstance::<_, ICustomList>(&DestinationList, None, CLSCTX_INPROC_SERVER)
    {
        if custom.SetApplicationID(PCWSTR(id.as_ptr())).is_ok() {
            let mut count = 0;
            if custom.GetCategoryCount(&mut count).is_ok() {
                for index in 0..count.min(50) {
                    let mut category = Category::default();
                    if custom.GetCategory(index, 1, &mut category).is_err() {
                        continue;
                    }
                    let name = match category.kind {
                        0 if category.value != 0 => {
                            indirect(&take_string(PWSTR(category.value as *mut u16)))
                        }
                        2 => "Tareas".into(),
                        _ => continue, // Known categories already read above.
                    };
                    let mut raw = null_mut();
                    if custom
                        .EnumerateCategoryDestinations(index, &IObjectArray::IID, &mut raw)
                        .is_ok()
                        && !raw.is_null()
                    {
                        let items = collection(&IObjectArray::from_raw(raw));
                        if !items.is_empty() {
                            groups.push(JumpGroup { name, items });
                        }
                    }
                }
            }
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_real_shell_lists_without_mutating_them() {
        std::thread::spawn(|| unsafe {
            use windows::Win32::System::Com::{
                CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
            };
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
            {
                let id = app_id("", r"C:\Windows\explorer.exe", &[]);
                assert!(!id.is_empty(), "Shell resolver must identify Explorer");
                for app in [id.as_str(), "Chrome", "MSEdge", "Noki.Nonexistent.JumpList"] {
                    let groups = read(app);
                    eprintln!(
                        "{app}: {:?}",
                        groups
                            .iter()
                            .map(|g| (&g.name, g.items.len()))
                            .collect::<Vec<_>>()
                    );
                    for group in groups {
                        assert!(!group.name.is_empty());
                        for item in group.items {
                            assert!(!item.name.is_empty());
                        }
                    }
                }
                assert!(read("Noki.Nonexistent.JumpList").is_empty());
            }
            CoUninitialize();
        })
        .join()
        .unwrap();
    }
}
