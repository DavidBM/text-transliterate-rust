use crate::transliterate::{TextTransliterate, TransliterationError};
use async_channel::{bounded as async_channel, Receiver as AsyncReceiver, Sender as AsyncSender};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::thread;

#[derive(Debug)]
enum TransliterateRequest {
    Die,
    Transliterate(TransliterationData),
}

#[derive(Debug)]
enum MessageCallback {
    Sync(Sender<Result<String, TransliterationError>>),
    Async(AsyncSender<Result<String, TransliterationError>>),
}

#[derive(Debug)]
struct TransliterationData {
    text: String,
    sender: MessageCallback,
    locale: String,
}

#[derive(Debug)]
pub enum TransliterateFailure {
    TransliterationError(TransliterationError),
    ErrorCommunicationWithPool,
}

/// Run conversions on a dedicated pool for applications that need to keep
/// CPU work off their async executor.

#[derive(Debug)]
pub struct TextTransliterateOffThread {
    sender: Sender<TransliterateRequest>,
    pool_size: usize,
}

impl Default for TextTransliterateOffThread {
    fn default() -> Self {
        Self::new(4)
    }
}

impl TextTransliterateOffThread {
    pub fn new(pool_size: usize) -> TextTransliterateOffThread {
        let sender = TextTransliterateOffThread::generate_transliterator_pool(pool_size);

        TextTransliterateOffThread { sender, pool_size }
    }

    fn generate_transliterator_pool(pool_size: usize) -> Sender<TransliterateRequest> {
        let (sender, receiver): (Sender<TransliterateRequest>, Receiver<TransliterateRequest>) =
            unbounded();

        for _ in 0..pool_size {
            let text_transliterate = TextTransliterate::new();

            TextTransliterateOffThread::create_thread(receiver.clone(), text_transliterate);
        }

        sender
    }

    fn create_thread(receiver: Receiver<TransliterateRequest>, tt: TextTransliterate) {
        thread::spawn(move || {
            while let Ok(request) = receiver.recv() {
                match request {
                    TransliterateRequest::Die => break,
                    TransliterateRequest::Transliterate(request) => {
                        let result = tt.transliterate(request.text, request.locale);
                        match request.sender {
                            MessageCallback::Async(sender) => {
                                let _ = sender.send_blocking(result);
                            }
                            MessageCallback::Sync(sender) => {
                                let _ = sender.send(result);
                            }
                        }
                    }
                }
            }
        });
    }

    fn regenerate_transliterator(&mut self) {
        let _ = self.sender.send(TransliterateRequest::Die);
        let sender = TextTransliterateOffThread::generate_transliterator_pool(self.pool_size);
        let old_sender = std::mem::replace(&mut self.sender, sender);
        drop(old_sender); // This will "disconnect" the channel, making the threads to consume all messages and finish.
    }

    /// Synchronous transliteration. It will pause the thread until the second thread
    /// finish the transliteration.
    pub fn transliterate<S: Into<String>>(
        &mut self,
        text: S,
        locale: S,
    ) -> Result<String, TransliterateFailure> {
        type SenderReceiver = (
            Sender<Result<String, TransliterationError>>,
            Receiver<Result<String, TransliterationError>>,
        );
        let (sender, receiver): SenderReceiver = bounded(1);

        let text = text.into();
        let locale = locale.into();

        let send_result =
            self.sender
                .send(TransliterateRequest::Transliterate(TransliterationData {
                    text,
                    sender: MessageCallback::Sync(sender),
                    locale,
                }));

        if send_result.is_err() {
            self.regenerate_transliterator();
            return Err(TransliterateFailure::ErrorCommunicationWithPool);
        }

        if let Ok(result) = receiver.recv() {
            match result {
                Ok(text) => Ok(text),
                Err(message) => Err(TransliterateFailure::TransliterationError(message)),
            }
        } else {
            self.regenerate_transliterator();
            Err(TransliterateFailure::ErrorCommunicationWithPool)
        }
    }

    /// Async version of the `transliterate` method. This will send the transliteration
    /// to the thread pool and return once finished
    pub async fn async_transliterate<S: Into<String>>(
        &mut self,
        text: S,
        locale: S,
    ) -> Result<String, TransliterateFailure> {
        type SenderReceiver = (
            AsyncSender<Result<String, TransliterationError>>,
            AsyncReceiver<Result<String, TransliterationError>>,
        );
        let (sender, receiver): SenderReceiver = async_channel(1);

        let text = text.into();
        let locale = locale.into();

        let send_result =
            self.sender
                .send(TransliterateRequest::Transliterate(TransliterationData {
                    text,
                    sender: MessageCallback::Async(sender),
                    locale,
                }));

        if send_result.is_err() {
            self.regenerate_transliterator();
            return Err(TransliterateFailure::ErrorCommunicationWithPool);
        }

        if let Ok(result) = receiver.recv().await {
            match result {
                Ok(text) => Ok(text),
                Err(message) => Err(TransliterateFailure::TransliterationError(message)),
            }
        } else {
            self.regenerate_transliterator();
            Err(TransliterateFailure::ErrorCommunicationWithPool)
        }
    }
}

impl Drop for TextTransliterateOffThread {
    fn drop(&mut self) {
        let _ = self.sender.send(TransliterateRequest::Die);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "de")]
    use futures_lite::future::block_on;

    #[test]
    fn uses_the_safe_engine_synchronously() {
        let mut converter = TextTransliterateOffThread::default();
        assert_eq!(
            converter.transliterate("Straße", "en_US").unwrap(),
            "Strasse"
        );
    }

    #[cfg(feature = "de")]
    #[test]
    fn uses_locale_rules_asynchronously() {
        block_on(async {
            let mut converter = TextTransliterateOffThread::default();
            assert_eq!(
                converter
                    .async_transliterate("Grüße", "de_DE")
                    .await
                    .unwrap(),
                "Gruesse"
            );
        });
    }
}
