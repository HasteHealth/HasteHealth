use std::sync::Arc;

use axum::{
    Form,
    extract::{OriginalUri, State},
};
use axum_extra::routing::TypedPath;
use maud::{Markup, html};
use oxidized_fhir_client::request::Operation;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    types::user::{LoginMethod, LoginResult},
};
use serde::Deserialize;

use crate::{AppState, extract::path_tenant::Tenant};

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct Login;

pub async fn login_get(_: Login, uri: OriginalUri) -> Result<Markup, String> {
    let route = uri.path().to_string();
    let response = html! {
        head {
            meta charset="utf-8" {}
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            link rel="preload" as="image" href="/img/logo.svg" {}
            title { "Oxidized Health" }
            link rel="icon" href="/img/logo.svg" {}
            link rel="stylesheet" href="/css/app.css" {}
        }
        body {
            section class="bg-gray-50  h-screen" {
                div class="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0" {
                    a href="#" class="flex items-center mb-6 text-2xl font-semibold text-gray-900" {
                        img class="w-8 h-8 mr-2" src="/img/logo.svg" alt="logo" {}
                        "Oxidized Health"
                    }
                    div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 sm:max-w-md" {
                        div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                            div {}
                            div {}
                            h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl " { "Sign in to your account" }
                            form class="space-y-4 md:space-y-6" action=(route) method="POST" {
                                div {
                                    label for="email" class="block mb-2 text-sm font-medium text-gray-900 " { "Your email" }
                                    input type="email" id="email" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5 " placeholder="name@company.com" required name="email" value="" {}
                                }
                                div {
                                    label for="password" class="block mb-2 text-sm font-medium text-gray-900" { "Password" }
                                    input type="password" id="password" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5" required name="password" {}
                                }
                                div class="flex items-center justify-between" {
                                    div class="flex items-start" {
                                        div class="flex items-center h-5" {
                                            input id="remember" aria-describedby="remember" type="checkbox" class="w-4 h-4" required {}
                                        }
                                        div class="ml-3 text-sm" {
                                            label for="remember" class="text-gray-500" { "Remember me" }
                                        }
                                    }
                                    a href="/w/2ld12f8nbrz80m3asevbk/oidc/interaction/password-reset" class="text-sm font-medium text-teal-600 hover:underline " { "Forgot password?" }
                                }
                                button type="submit" class="w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Sign in" }
                            }
                            div class="mt-4 space-y-2" {}
                        }
                    }
                }
            }
        }

    };

    Ok(response)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

pub async fn login_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    _: Login,
    State(state): State<Arc<AppState<Repo, Search>>>,
    Tenant { tenant }: Tenant,
    Form(login_data): Form<LoginForm>,
) -> Result<Markup, OperationOutcomeError> {
    // Handle the login post request here
    // For now, we will just return a simple message

    let login_result = state
        .repo
        .login(
            &tenant,
            &LoginMethod::EmailPassword {
                email: login_data.email,
                password: login_data.password,
            },
        )
        .await?;

    match login_result {
        LoginResult::Success { user } => {
            let message = format!("User {} logged in successfully", user.id);
            // Handle successful login, e.g., set session, redirect, etc.
            Ok(html! {
                h1 { (message) }
            })
        }
    }
}
