// function getRegexForRedirect(urlPattern: string): RegExp {
//   const regex = new RegExp(urlPattern.replaceAll("*", "(.+)"));
//   return regex;
// }

// export function isInvalidRedirectUrl(
//   redirectUrl: string | undefined,
//   client: ClientApplication,
// ): boolean {
//   return (
//     !redirectUrl ||
//     !client.redirectUri?.find((v) => getRegexForRedirect(v).test(redirectUrl))
//   );
// }

use oxidized_fhir_model::r4::types::ClientApplication;
use regex::Regex;

pub fn is_valid_redirect_url(redirect_url: &str, client: &ClientApplication) -> bool {
    let k = client.redirectUri.as_ref().and_then(|redirect_uris| {
        redirect_uris.iter().find(|redirect_pattern| {
            if let Some(redirect_pattern) = redirect_pattern.value.as_ref()
                && let Ok(pattern) = Regex::new(&redirect_pattern.replace("*", "(.+)"))
            {
                pattern.is_match(redirect_url)
            } else {
                false
            }
        })
    });

    k.is_some() && !redirect_url.is_empty()
}
