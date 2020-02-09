use crate::transliterate::TextTransliterate;
use async_std::sync::{channel as async_channel, Receiver as AsyncReceiver, Sender as AsyncSender};
use async_std::task::block_on;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug)]
enum TransliterateRequest {
    Die,
    Transliterate(TransliterationData),
}

#[derive(Debug)]
enum MessageCallback {
    Sync(Sender<Result<String, &'static str>>),
    Async(AsyncSender<Result<String, &'static str>>),
}

#[derive(Debug)]
struct TransliterationData {
    text: String,
    sender: MessageCallback,
    locale: String,
}

#[derive(Debug)]
pub struct TextTransliterateOffThread {
    sender: Sender<TransliterateRequest>,
}

impl Default for TextTransliterateOffThread {
    fn default() -> Self {
        Self::new()
    }
}

impl TextTransliterateOffThread {
    pub fn new() -> TextTransliterateOffThread {
        let sender = TextTransliterateOffThread::generate_transliterator();

        TextTransliterateOffThread { sender }
    }

    fn generate_transliterator() -> Sender<TransliterateRequest> {
        let (sender, receiver): (Sender<TransliterateRequest>, Receiver<TransliterateRequest>) =
            mpsc::channel();

        let text_transliterate = TextTransliterate::new();

        TextTransliterateOffThread::create_thread(receiver, text_transliterate);

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
        let sender = TextTransliterateOffThread::generate_transliterator();
        self.sender = sender;
    }

    pub fn transliterate<S: Into<String>>(
        &mut self,
        text: S,
        locale: S,
    ) -> Result<String, &'static str> {
        type SenderReceiver = (
            Sender<Result<String, &'static str>>,
            Receiver<Result<String, &'static str>>,
        );
        let (sender, receiver): SenderReceiver = mpsc::channel();

        let text = text.into();
        let locale = locale.into();

        let send_result =
            self.sender
                .send(TransliterateRequest::Transliterate(TransliterationData {
                    text: text.clone(),
                    sender: MessageCallback::Sync(sender),
                    locale: locale.clone(),
                }));

        if send_result.is_err() {
            self.regenerate_transliterator();
            return self.transliterate(text, locale);
        }

        if let Ok(result) = receiver.recv() {
            match result {
                Ok(text) => Ok(text),
                Err(message) => Err(message),
            }
        } else {
            self.regenerate_transliterator();
            Err("Error communicating with the thread. Regenerating thread")
        }
    }

    pub async fn async_transliterate<S: Into<String>>(
        &mut self,
        text: S,
        locale: S,
    ) -> Result<String, &'static str> {
        type SenderReceiver = (
            AsyncSender<Result<String, &'static str>>,
            AsyncReceiver<Result<String, &'static str>>,
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
            return Err("Send failed. Try again.");
        }

        if let Some(result) = receiver.recv().await {
            match result {
                Ok(text) => Ok(text),
                Err(message) => Err(message),
            }
        } else {
            self.regenerate_transliterator();
            Err("Error communicating with the thread. Regenerating thread")
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
        let mut tt = TextTransliterateOffThread::new();
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
        if let Ok(result) = result {
            assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn japanse_dont_crash() {
        let mut tt = TextTransliterateOffThread::new();
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "ja_JP.UTF-8");
        if let Ok(result) = result {
            assert_eq!("u  a  o  ss  U  A  O c n ? ?", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn chinese_dont_crash() {
        let mut tt = TextTransliterateOffThread::new();
        let result = tt.transliterate("ウェブ全体から検索", "zh_CN.UTF-8");
        if let Ok(result) = result {
            assert_eq!("?????????", result);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn coins() {
        let mut tt = TextTransliterateOffThread::new();
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
            let mut tt = TextTransliterateOffThread::new();
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
            let mut tt = TextTransliterateOffThread::new();
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
            let mut tt = TextTransliterateOffThread::new();
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
            let mut tt = TextTransliterateOffThread::new();
            let result = tt.transliterate("€ £ $ ¥", "en_US.UTF-8");
            if let Ok(result) = result {
                assert_eq!("EUR GBP $ JPY", result);
            } else {
                unreachable!();
            }
        });
    }
}
