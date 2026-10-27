impl Host {
fn require(&self, method: &str, cap: &str) -> Result<(), String> {
        if !self.granted(method, cap) {
            Err(format!("Capability not granted: {cap}"))
        } else {
            Ok(())
        }
    }
}
