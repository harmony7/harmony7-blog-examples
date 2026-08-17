mod bindings;

use bindings::{exports::fastly::compute::http_incoming, fastly::compute::{compute_runtime, http_body, http_resp}};

struct ComputeRuntimeExample;

impl http_incoming::Guest for ComputeRuntimeExample {
    fn handle(_request: http_incoming::Request, _request_body: http_body::Body) -> Result<(), ()> {
        let mut lines = String::new();

        lines.push_str(&format!("get-vcpu-ms():          {}\n", compute_runtime::get_vcpu_ms()));
        lines.push_str(&format!("get-heap-mib():         {}\n", compute_runtime::get_heap_mib()));
        lines.push_str(&format!("get-sandbox-id():       {}\n", compute_runtime::get_sandbox_id()));
        lines.push_str(&format!("get-hostname():         {}\n", compute_runtime::get_hostname()));
        lines.push_str(&format!("get-pop():              {}\n", compute_runtime::get_pop()));
        lines.push_str(&format!("get-region():           {}\n", compute_runtime::get_region()));
        lines.push_str(&format!("get-cache-generation(): {}\n", compute_runtime::get_cache_generation()));
        lines.push_str(&format!("get-customer-id():      {}\n", compute_runtime::get_customer_id()));
        lines.push_str(&format!("get-is-staging():       {}\n", compute_runtime::get_is_staging()));
        lines.push_str(&format!("get-service-id():       {}\n", compute_runtime::get_service_id()));
        lines.push_str(&format!("get-service-version():  {}\n", compute_runtime::get_service_version()));
        lines.push_str(&format!("get-namespace-id():     {}\n", compute_runtime::get_namespace_id()));

        let response = http_resp::Response::new().map_err(|_| ())?;
        response
            .insert_header("content-type", b"text/plain")
            .map_err(|_| ())?;

        let out_body = http_body::new().map_err(|_| ())?;
        http_body::write(&out_body, lines.as_bytes()).map_err(|_| ())?;
        http_resp::send_downstream(response, out_body).map_err(|_| ())?;

        Ok(())
    }
}

bindings::export!(ComputeRuntimeExample with_types_in bindings);
