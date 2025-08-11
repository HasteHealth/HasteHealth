use axum::response::Response;
use axum_extra::routing::TypedPath;
use maud::{Markup, html};

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct Login;

pub async fn login(_: Login) -> Result<Markup, String> {
    // class="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-teal-300 dark:bg-gray-700 dark:border-gray-600 dark:focus:ring-teal-600 dark:ring-offset-gray-800"
    let response = html! {
        link rel="stylesheet" href="/css/app.css" {}
        body {
            section class="bg-gray-50 h-screen" {
                div class="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0" {
                    a href="#" class="flex items-center mb-6 text-2xl font-semibold text-gray-900 " {
                        img class="w-8 h-8 mr-2" src="/img/logo.svg" alt="Oxidized Health" { "Oxidized Health" }
                    }
                    div class="w-full bg-white rounded-lg shadow sm:max-w-md" {
                        div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                            div {}
                            h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl " { "Sign in to your account" }
                            form class="space-y-4 md:space-y-6" action="/auth/login" method="POST" {
                                div {
                                    label for="email" class="block mb-2 text-sm font-medium text-gray-900 " { "Your email" }
                                    input
                                      type="email"
                                      id="email"
                                      class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5"
                                      placeholder="name@company.com"
                                      required=""
                                      name="email" value="" {}
                                }
                                div class="flex items-center justify-between" {
                                    div class="flex items-start" {
                                        div class="flex items-center h-5" {
                                            input id="remember" aria-describedby="remember"
                                            type="checkbox"
                                            class="w-4 h-4"
                                            required="true"
                                            {}
                                        }
                                        div class="ml-3 text-sm" {
                                            label for="remember" class="text-gray-500" { "Remember me" }
                                        }
                                    }
                                }
                                button
                                  type="submit"
                                  class="w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium
                                        rounded-lg text-sm px-5 py-2.5 text-center" { "Sign in" }
                                p class="text-sm font-light text-gray-500" {
                                    "Don't have an account yet? "
                                    a href="/auth/signup" class="font-medium text-teal-600 hover:underline " { "Sign up" }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(response)
}
