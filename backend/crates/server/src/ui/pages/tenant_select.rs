use crate::ui::components::{banner, page_html};
use maud::{Markup, html};

pub fn tenant_select_form_html(tenant_select_route: &str, errors: Option<Vec<String>>) -> Markup {
    page_html(html! {
        (banner("Enter your tenant identifier", None))
        div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 w-md sm:max-w-md text-slate-700" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                @if let Some(errors) = errors {
                    div class="mb-4" {
                        @for error in errors {
                            div class="text-red-600 text-sm" { (error) }
                        }
                    }
                }
                form class="space-y-4 md:space-y-6" action=(tenant_select_route) method="POST" {
                    div class="grid grid-cols-4 gap-1" {
                        div class="col-span-3" {
                            label for="tenant" class="block text-sm font-medium text-slate-600" { "Tenant" }
                            input type="tenant" id="tenant" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " placeholder="Tenant id" required name="tenant" value="" {}
                        }
                        div class="col-span-1" {
                            label for="project" class="block text-sm font-medium text-slate-600" { "Project" }
                            input type="project" id="project" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " placeholder="system" name="project" value="" {}
                        }
                    }

                    div class="space-y-4" {
                        button type="submit" class="w-full text-white bg-orange-500 hover:bg-orange-500 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Continue" }
                        div class="flex items-center justify-start" {
                            a href=("/") class="text-sm font-medium text-orange-600 hover:underline " { "Sign up" }
                        }
                    }
                }
            }
        }
    })
}
