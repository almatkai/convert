impl Host {
fn granted(&self, method: &str, cap: &str) -> bool {
        self.manifest["capabilities"].as_array().is_some_and(|a| {
            a.iter()
                .any(|v| v == cap || v == method || v.as_str() == method.strip_prefix("host."))
        })
    }
}
