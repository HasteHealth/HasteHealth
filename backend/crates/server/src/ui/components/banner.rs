use haste_jwt::TenantId;
use maud::{Markup, html};

use crate::static_assets::asset_route;

#[derive(Clone, Debug, Default)]
pub struct TenantName(pub Option<String>);

/// Bundles a request's tenant identity together with its display branding, so extractors
/// and page-render functions that need both don't have to take two separate parameters.
/// Populated once per request by [`crate::extract::path_tenant`]'s `FromRequestParts` impl.
#[derive(Clone)]
pub struct TenantContext {
    pub tenant: TenantId,
    pub branding: TenantName,
}

/// Renders the banner for pages with no tenant context (e.g. before a tenant has been
/// identified). `header` is shown as-is, and the static default logo is used.
pub fn banner(header: &str, subheader: Option<&str>) -> Markup {
    html! {
        div class="flex flex-col items-center justify-center space-y-1" {
            a href="#" class="h-16 w-54 relative flex justify-center items-center overflow-hidden text-2xl font-semibold text-gray-900" {
                img class="max-h-full max-w-full object-contain" src=(asset_route("img/logo_text.svg")) alt="logo" {}
            }
            div class="flex space-x-1 items-center justify-center text-sm text-slate-400 mt-[-12px]" {
                div { span class="font-bold" { (header) } }
                @if let Some(subheader) = subheader {
                    div { span { (subheader) } }
                }
            }
        }
    }
}

/// Renders the banner for a tenant-scoped page. `header` must be the raw tenant id — it's used
/// to build the tenant's logo route and as the display-name fallback when `name` has none set.
pub fn tenant_banner(header: &str, subheader: Option<&str>, name: &TenantName) -> Markup {
    let display_name = name.0.as_deref().unwrap_or(header);

    html! {
        div class="flex flex-col items-center justify-center space-y-1" {
            a href="#" class="h-16 w-54 relative flex justify-center items-center overflow-hidden text-2xl font-semibold text-gray-900" {
                img class="max-h-full max-w-full object-contain" src=(format!("/w/{}/branding/logo", header)) alt=(format!("{} logo", display_name)) {}
            }
            div class="flex space-x-1 items-center justify-center text-sm text-slate-400" {
                div { span class="font-bold" { (display_name) } }
                @if let Some(project_name) = subheader {
                    div { span { (project_name) } }
                }
            }
            a href="https://haste.health" target="_blank" rel="noopener noreferrer"
                class="text-xs text-slate-300 hover:text-slate-400" {
                "Powered by Haste Health"
            }
        }
    }
}

/// Renders [`tenant_banner`] when tenant branding is available, otherwise falls back to the
/// tenant-less [`banner`]. Exists to bridge callers whose own signature still supports both
/// tenant-scoped and system-level pages (and so only have `Option<&TenantName>` on hand).
pub fn page_banner(header: &str, subheader: Option<&str>, name: Option<&TenantName>) -> Markup {
    match name {
        Some(name) => tenant_banner(header, subheader, name),
        None => banner(header, subheader),
    }
}

#[cfg(test)]
mod tests {
    use super::{TenantName, banner, tenant_banner};

    #[test]
    fn tenant_banner_uses_custom_name_and_logo_route() {
        let name = TenantName(Some("Acme Health".to_string()));

        let rendered = tenant_banner("acme", None, &name).into_string();

        assert!(rendered.contains("Acme Health"));
        assert!(rendered.contains("/w/acme/branding/logo"));
    }

    #[test]
    fn tenant_banner_falls_back_to_tenant_id_and_default_logo() {
        let name = TenantName(None);

        let rendered = tenant_banner("acme", None, &name).into_string();

        assert!(rendered.contains("acme"));
        assert!(rendered.contains("/w/acme/branding/logo"));
    }

    #[test]
    fn tenant_banner_shows_powered_by_watermark_for_custom_branding() {
        let name = TenantName(Some("Acme Health".to_string()));

        let rendered = tenant_banner("acme", None, &name).into_string();

        assert!(rendered.contains("Powered by Haste Health"));
    }

    #[test]
    fn banner_without_tenant_uses_static_logo() {
        let rendered = banner("Login", None).into_string();

        assert!(!rendered.contains("/branding/logo"));
        assert!(rendered.contains("logo_text.svg"));
        assert!(!rendered.contains("Powered by Haste Health"));
    }
}
