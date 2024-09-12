pub mod lnd {
    use std::error::Error;
    use tonic_lnd::lnrpc::payment::PaymentStatus;

    use crate::{
        config::config::{ConfigJob, ValidConfig},
        lnurl::lnurl::LnurlPaymentRequestResponse,
    };

    pub async fn pay_invoice(
        invoice: LnurlPaymentRequestResponse,
        job: &ConfigJob,
        config: ValidConfig,
    ) -> Result<(), Box<dyn Error>> {
        let mut client = tonic_lnd::connect(
            config.server_url.to_owned(),
            config.cert_path.to_owned(),
            config.macaroon_path.to_owned(),
        )
        .await?;

        let payment_response = client
            .router()
            .send_payment_v2(tonic_lnd::routerrpc::SendPaymentRequest {
                payment_request: invoice.pr,
                timeout_seconds: 30,
                fee_limit_sat: (job.amount_in_sats as f32 * 0.01).ceil() as i64, // max 1% fee
                ..Default::default()
            })
            .await?;

        let mut payment_stream = payment_response.into_inner();

        while let Some(payment) = payment_stream.message().await? {
            println!("Payment update: {:?}", payment);

            let payment_status = PaymentStatus::from_i32(payment.status).unwrap();

            match payment_status {
                PaymentStatus::Succeeded => {
                    println!("Payment success...");
                }
                PaymentStatus::InFlight => {
                    println!("Payment in process...");
                }
                _ => {
                    println!("Payment failed...");
                }
            }
        }

        Ok(())
    }
}
