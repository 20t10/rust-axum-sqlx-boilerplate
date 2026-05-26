pub trait JobPublisher: Send + Sync {
    fn publish(&self, topic: &str, payload: &str);
}
