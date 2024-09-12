use std::{error::Error, str::from_utf8};

use serde::Deserialize;

use crate::config::ConfigJob;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LnurlInfoResponse {
    callback: String,
    //  max_sendable: u64,
    //  min_sendable: u64,
    //  comment_allowed: u64,
    // ...
}

// #[derive(Deserialize)]
// struct LnurlResponseSuccessAction {
//     tag: String,
//     message: String,
// }

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LnurlPaymentRequestResponse {
    pub pr: String,
    //  routes: Vec<String>,
    //  success_action: LnurlResponseSuccessAction,
}

pub struct LnurlService {
    config_job: ConfigJob,
    info: Option<LnurlInfoResponse>,
    payment_request: Option<LnurlPaymentRequestResponse>,
}

impl LnurlService {
    pub fn new(config_job: ConfigJob) -> Self {
        LnurlService {
            config_job,
            info: None,
            payment_request: None,
        }
    }

    async fn get_info(&mut self, url: &str) -> Result<&mut Self, reqwest::Error> {
        let lnurl_init_response = reqwest::get(url).await?.json::<LnurlInfoResponse>().await?;

        self.info = Some(lnurl_init_response);

        Ok(self)
    }

    fn validate(&mut self) -> Result<&mut Self, String> {
        // todo
        Ok(self)
    }

    async fn get_payment_request(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let callback = match self.info.clone() {
            Some(info) => info.callback,
            None => return Err("No callback found".into()),
        };

        let request_url = format!(
            "{}?amount={}&comment={}",
            callback,
            self.config_job.amount_in_sats * 1000,
            self.config_job.memo.to_owned().unwrap_or("".to_string())
        );

        let response = reqwest::get(request_url)
            .await?
            .json::<LnurlPaymentRequestResponse>()
            .await?;

        self.payment_request = Some(response);

        Ok(self)
    }

    pub async fn get_invoice(
        &mut self,
        url: &str,
    ) -> Result<LnurlPaymentRequestResponse, Box<dyn Error>> {
        self.get_info(url)
            .await?
            .validate()?
            .get_payment_request()
            .await?;

        match self.payment_request.clone() {
            Some(payment_request) => Ok(payment_request),
            None => Err("No payment request found".into()),
        }
    }
}

pub fn get_url_from_ln_address_or_lnurl(ln_address_or_lnurl: &str) -> String {
    if ln_address_or_lnurl.contains("@") {
        let url_parts: Vec<&str> = ln_address_or_lnurl.split("@").collect();

        format!(
            "https://{}/.well-known/lnurlp/{}",
            url_parts[1], url_parts[0]
        )
    } else {
        let lnurl_upper = ln_address_or_lnurl.to_uppercase();

        let (_hrp, data) = bech32::decode(&lnurl_upper).expect("Failed to decode lnurl");

        let url = from_utf8(&data).expect("Failed to convert lnurl bytes to utf8");

        url.to_string()
    }
}
