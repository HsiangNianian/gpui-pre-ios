//! Picking files to open with the document picker, and handing a file the app
//! wrote to the share sheet.

use futures::channel::oneshot;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSURL};
use objc2_ui_kit::{
    UIActivityViewController, UIDocumentPickerDelegate, UIDocumentPickerMode,
    UIDocumentPickerViewController, UIViewController,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

type PathsSender = oneshot::Sender<anyhow::Result<Option<Vec<PathBuf>>>>;

thread_local! {
    // UIKit holds a picker's delegate weakly: keep the latest one alive.
    static PICKER_DELEGATE: RefCell<Option<Retained<PickerDelegate>>> = const { RefCell::new(None) };
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[name = "GPUIIosDocumentPickerDelegate"]
    #[ivars = RefCell<Option<PathsSender>>]
    struct PickerDelegate;

    unsafe impl NSObjectProtocol for PickerDelegate {}

    unsafe impl UIDocumentPickerDelegate for PickerDelegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        fn did_pick(&self, _controller: &UIDocumentPickerViewController, urls: &NSArray<NSURL>) {
            let paths = urls
                .iter()
                .filter_map(|url| url.path())
                .map(|path| PathBuf::from(path.to_string()))
                .collect();
            self.answer(Ok(Some(paths)));
        }

        #[unsafe(method(documentPickerWasCancelled:))]
        fn was_cancelled(&self, _controller: &UIDocumentPickerViewController) {
            self.answer(Ok(None));
        }
    }
);

impl PickerDelegate {
    fn new(sender: PathsSender, main_thread: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(main_thread).set_ivars(RefCell::new(Some(sender)));
        unsafe { msg_send![super(this), init] }
    }

    fn answer(&self, result: anyhow::Result<Option<Vec<PathBuf>>>) {
        let sender = self.ivars().borrow_mut().take();
        if let Some(sender) = sender {
            if sender.send(result).is_err() {
                log::debug!("GPUI iOS: File picker receiver was dropped");
            }
        }
    }
}

/// Presents the document picker in import mode, so the picked files are copied
/// into the app's temporary Inbox, and answers with their paths.
#[allow(deprecated)] // The UTI-string initializer avoids a UniformTypeIdentifiers dependency.
pub(super) fn pick_files(presenter: &UIViewController, multiple: bool, sender: PathsSender) {
    let main_thread = presenter.mtm();
    let types = NSArray::from_retained_slice(&[NSString::from_str("public.item")]);
    let picker = UIDocumentPickerViewController::initWithDocumentTypes_inMode(
        UIDocumentPickerViewController::alloc(main_thread),
        &types,
        UIDocumentPickerMode::Import,
    );
    picker.setAllowsMultipleSelection(multiple);
    let delegate = PickerDelegate::new(sender, main_thread);
    picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    // Replacing an unanswered delegate drops its sender, which cancels that prompt.
    PICKER_DELEGATE.with_borrow_mut(|current| *current = Some(delegate));
    presenter.presentViewController_animated_completion(&picker, true, None);
}

/// Where `prompt_for_new_path` asks the app to write: a fresh file in the app's
/// temporary directory, to be handed to `share_file` once written.
pub(super) fn new_export_path(suggested_name: Option<&str>) -> anyhow::Result<Option<PathBuf>> {
    let directory = std::env::temp_dir().join("Export");
    std::fs::create_dir_all(&directory)?;
    let name = suggested_name
        .and_then(|name| Path::new(name).file_name())
        .map(|name| name.to_owned())
        .unwrap_or_else(|| "Untitled".into());
    let path = directory.join(name);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(Some(path))
}

/// Presents the share sheet (Save to Files, AirDrop, Mail…) for a file, as a
/// popover centred on the presenter's view.
pub(super) fn share(presenter: &UIViewController, path: &Path) {
    let main_thread = presenter.mtm();
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    let item: &AnyObject = &url;
    let items = NSArray::<AnyObject>::from_slice(&[item]);
    let sheet = unsafe {
        UIActivityViewController::initWithActivityItems_applicationActivities(
            UIActivityViewController::alloc(main_thread),
            &items,
            None,
        )
    };
    if let (Some(popover), Some(view)) = (sheet.popoverPresentationController(), presenter.view()) {
        let bounds = view.bounds();
        popover.setSourceView(Some(&view));
        popover.setSourceRect(CGRect::new(
            CGPoint::new(bounds.size.width / 2., bounds.size.height / 2.),
            CGSize::new(0., 0.),
        ));
    }
    presenter.presentViewController_animated_completion(&sheet, true, None);
}
