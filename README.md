# gpui-pre-ios (Alcyo's patches)

This is the source of the crates.io crate [`gpui-pre-ios` 0.3.11](https://crates.io/crates/gpui-pre-ios/0.3.11)
(Zed's `gpui_ios` crate, snapshot of zed-industries/zed@f816e1f), with a few small patches the
[Alcyo](https://github.com/HsiangNianian/alcyo) iPad app needs. The first commit is the published source
unchanged; every later commit is one patch. The version stays 0.3.11 and the crate still depends on
`gpui-pre = "=0.3.11"` from crates.io, so it can replace the published crate through `[patch.crates-io]`:

```toml
[patch.crates-io]
gpui-pre-ios = { git = "https://github.com/HsiangNianian/gpui-pre-ios", rev = "<commit>" }
```

The patches:

- `gpui_ios::current_touch_is_pencil()`: whether the touch being delivered right now comes from an Apple Pencil.
- External displays: a cabled or AirPlay display's scene (`UIWindowSceneSessionRoleExternalDisplayNonInteractive`)
  is listed by `displays()`, `gpui_ios::external_display()` describes it, and a window opened with its
  `display_id` is shown on it (and closed when it disconnects).

Licensed under the Apache License 2.0, like the original (LICENSE-APACHE).
