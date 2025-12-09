use crate::ui::components::page_html;
use maud::{Markup, html};

pub fn tenant_select_form_html(tenant_select_route: &str, errors: Option<Vec<String>>) -> Markup {
    page_html(html! {
        div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 sm:max-w-md text-slate-700" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                @if let Some(errors) = errors {
                    div class="mb-4" {
                        @for error in errors {
                            div class="text-red-600 text-sm" { (error) }
                        }
                    }
                }
                h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl" { "Tenant Select" }
                form class="space-y-4 md:space-y-6" action=(tenant_select_route) method="POST" {
                    div {
                        label for="tenant" class="block mb-2 text-sm font-medium text-gray-900 " { "Your Tenant" }
                        input type="tenant" id="tenant" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " placeholder="Enter your tenant identifier" required name="tenant" value="" {}
                    }
                    div class="space-y-2" {
                        div class="flex items-center justify-between" {
                            a href=("/") class="text-sm font-medium text-orange-600 hover:underline " { "Sign up" }
                        }
                    }
                    button type="submit" class="w-full text-white bg-orange-500 hover:bg-orange-500 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Sign in" }
                }
            }
        }
    })
}
