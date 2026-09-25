// <fold imports>
mod bindings;

use bindings::{
    exports::fastly::compute::http_incoming,
    fastly::compute::{backend, http_body, http_req, http_resp, image_optimizer},
};
// </fold>

const IMAGE_URL: &str = "https://http-me.fastly.dev/image-jpeg";

fn origin_request() -> Result<http_req::Request, ()> {
    let request = http_req::Request::new().map_err(|_| ())?;
    request.set_uri(IMAGE_URL).map_err(|_| ())?;
    Ok(request)
}

struct ImageOptimizer;

impl http_incoming::Guest for ImageOptimizer {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let origin = backend::Backend::open("origin").map_err(|_| ())?;

        // Image Optimizer API parameters, including the region, as a query string.
        // <highlight>
        let options = image_optimizer::ImageOptimizerTransformOptions {
            sdk_claims_opts: Some("region=us_east&width=200&format=webp".to_string()),
            extra: None,
        };
        // </highlight>

        // <highlight>
        let result = image_optimizer::send_to_image_optimizer(origin_request()?, None, &origin, &options);
        // </highlight>

        let (response, body, note) = match result {
            Ok((response, body)) => (response, body, "transformed".to_string()),
            Err(e) => {
                // The request was moved into the call and didn't come back. Build it again.
                let (response, body) = http_req::send(origin_request()?, http_body::new().map_err(|_| ())?, &origin)
                    .map_err(|_| ())?;
                (response, body, format!("untransformed, send-to-image-optimizer: Err({e:?})"))
            }
        };

        response.insert_header("x-image-optimizer", note.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, body).map_err(|_| ())
    }
}

bindings::export!(ImageOptimizer with_types_in bindings);
