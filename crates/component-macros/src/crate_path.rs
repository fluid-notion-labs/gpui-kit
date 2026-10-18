use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Resolve the consumer's GPUI API, preferring the Kit facade over a direct engine.
pub(crate) fn gpui() -> syn::Result<TokenStream> {
    for package in ["gpui-kit", "gpui-pre"] {
        if let Ok(found) = crate_name(package) {
            return Ok(found_crate_path(found));
        }
    }
    // `gpui-fast` is an optional dependency wherever it appears, and
    // proc-macro-crate reports optional deps whether or not the feature is
    // enabled — so gate it on the feature Cargo exposes to proc macros.
    // Without this, e.g. `gpui-component` without its `gpui-fast` feature
    // resolves here and expands to paths into a crate that is not compiled.
    if std::env::var_os("CARGO_FEATURE_GPUI_FAST").is_some()
        && let Ok(found) = crate_name("gpui-fast")
    {
        return Ok(found_crate_path(found));
    }
    // Fall back to the engine itself: crates like gpui-component depend
    // directly on `gpui` (which may be a renamed/forked package).
    // proc-macro-crate keys the lookup by *package* name, so check both the
    // upstream name and the fork's (dep key `gpui`, package
    // `gpui-unofficial`); either way it returns the dep key to use in code.
    for package in ["gpui", "gpui-unofficial"] {
        if let Ok(found) = crate_name(package) {
            return Ok(found_crate_path(found));
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        "IntoPlot requires a direct dependency on gpui-kit, gpui-pre, gpui-fast, or gpui",
    ))
}

fn found_crate_path(found: FoundCrate) -> TokenStream {
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}

/// Resolve the `gpui-component` API exposed to the crate where a macro is
/// expanded, mirroring [`gpui`]: `gpui-kit` consumers reach it as
/// `gpui_kit::component`, standalone consumers as `gpui_component`, and the
/// crate itself as `crate`.
pub(crate) fn component() -> syn::Result<TokenStream> {
    match crate_name("gpui-kit") {
        Ok(found) => {
            let kit = found_crate_path(found);
            Ok(quote!(#kit::component))
        }
        Err(kit_error) => crate_name("gpui-component").map(found_crate_path).map_err(
            |component_error| {
                syn::Error::new(
                    Span::call_site(),
                    format!(
                        "IntoPlot requires a direct dependency on `gpui-kit` or `gpui-component`: \
                         gpui-kit lookup failed: {kit_error}; gpui-component lookup failed: \
                         {component_error}"
                    ),
                )
            },
        ),
    }
}
