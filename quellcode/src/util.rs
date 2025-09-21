/// Sends a value asynchronously over the provided channel.
///
/// # Arguments
///
/// * `channel` - The channel to send the value over.
/// * `value` - The value to send.
///
/// # Returns
///
/// None
///
/// # Errors
///
/// If the send operation fails, a warning will be logged.
pub async fn send_async_channel<T>(channel: &async_channel::Sender<T>, value: T) {
    let result = channel.send(value).await;
    if let Err(err) = result {
        log::warn!("Failed to send async channel: {}", err);
    }
}
