use crate::transliterate::{TextTransliterate, TransliterationError};
use async_std::sync::{channel as async_channel, Receiver as AsyncReceiver, Sender as AsyncSender};
use async_std::task::block_on;
use crossbeam::{crossbeam_channel::bounded, crossbeam_channel::unbounded, Receiver, Sender};
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

/// Transliterate from the chosen locale to ascii.
/// This function will create another thread in order to isolate the
/// locale set for iconv effects.
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
                        match tt.transliterate(request.text, request.locale) {
                            Ok(result) => match request.sender {
                                MessageCallback::Async(sender) => {
                                    block_on(async { sender.send(Ok(result)).await })
                                }
                                MessageCallback::Sync(sender) => sender.send(Ok(result)).unwrap(),
                            },
                            Err(error) => match request.sender {
                                MessageCallback::Async(sender) => {
                                    block_on(async { sender.send(Err(error)).await })
                                }
                                MessageCallback::Sync(sender) => sender.send(Err(error)).unwrap(),
                            },
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
                    text: text,
                    sender: MessageCallback::Sync(sender),
                    locale: locale,
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
                    text: text.clone(),
                    sender: MessageCallback::Async(sender),
                    locale: locale.clone(),
                }));

        if send_result.is_err() {
            self.regenerate_transliterator();
            return Err(TransliterateFailure::ErrorCommunicationWithPool);
        }

        if let Some(result) = receiver.recv().await {
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

    #[test]
    fn it_works() {
        let mut tt: TextTransliterateOffThread = Default::default();
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
        if let Ok(result) = result {
            assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn japanse_dont_crash() {
        let mut tt: TextTransliterateOffThread = Default::default();
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "ja_JP.UTF-8");
        if let Ok(result) = result {
            assert_eq!("u  a  o  ss  U  A  O c n ? ?", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn chinese_dont_crash() {
        let mut tt: TextTransliterateOffThread = Default::default();
        let result = tt.transliterate("ウェブ全体から検索", "zh_CN.UTF-8");
        if let Ok(result) = result {
            assert_eq!("?????????", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn coins() {
        let mut tt: TextTransliterateOffThread = Default::default();
        let result = tt.transliterate("€ £ $ ¥", "en_US.UTF-8");
        if let Ok(result) = result {
            assert_eq!("EUR GBP $ JPY", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn it_works_async() {
        block_on(async {
            let mut tt: TextTransliterateOffThread = Default::default();
            let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
            if let Ok(result) = result {
                assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
            } else {
                unreachable!();
            }
        });
    }

    #[test]
    fn japanse_dont_crash_async() {
        block_on(async {
            let mut tt: TextTransliterateOffThread = Default::default();
            let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "ja_JP.UTF-8");
            if let Ok(result) = result {
                assert_eq!("u  a  o  ss  U  A  O c n ? ?", result);
            } else {
                unreachable!();
            }
        });
    }

    #[test]
    fn chinese_dont_crash_async() {
        block_on(async {
            let mut tt: TextTransliterateOffThread = Default::default();
            let result = tt.transliterate("ウェブ全体から検索", "zh_CN.UTF-8");
            if let Ok(result) = result {
                assert_eq!("?????????", result);
            } else {
                unreachable!();
            }
        });
    }

    #[test]
    fn coins_async() {
        block_on(async {
            let mut tt: TextTransliterateOffThread = Default::default();
            let result = tt.transliterate("€ £ $ ¥", "en_US.UTF-8");
            if let Ok(result) = result {
                assert_eq!("EUR GBP $ JPY", result);
            } else {
                unreachable!();
            }
        });
    }
}
