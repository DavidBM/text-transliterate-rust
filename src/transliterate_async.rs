use transliterate::TextTransliterate;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
enum TransliterateRequest {
	Die,
	Transliterate(TransliterationData)
}

#[derive(Debug)]
struct TransliterationData {
	text: String,
	sender: Sender<Result<String, &'static str>>,
	locale: String
}

#[derive(Debug)]
pub struct TextTransliterateAsync {
	sender: Sender<TransliterateRequest>
}

impl TextTransliterateAsync {
	pub fn new() -> TextTransliterateAsync {
		let sender = TextTransliterateAsync::generate_transliterator();

		TextTransliterateAsync {sender: sender}
	}

	fn generate_transliterator() -> Sender<TransliterateRequest> {
		let (sender, receiver): (Sender<TransliterateRequest>, Receiver<TransliterateRequest>) = mpsc::channel();

		let text_transliterate = TextTransliterate::new();

		TextTransliterateAsync::create_thread(receiver, text_transliterate);

		sender
	}

	fn create_thread(receiver: Receiver<TransliterateRequest>, tt: TextTransliterate) {
		thread::spawn(move || {
			while let Ok(request) = receiver.recv() {
				match request {
					TransliterateRequest::Die => break,
					TransliterateRequest::Transliterate(request) => {
						match tt.transliterate(request.text, request.locale) {
							Ok(result) => request.sender.send(Ok(result)).unwrap(),
							Err(error) => request.sender.send(Err(error)).unwrap(),
						}
					}
				}
			}
		});
	}

 	fn regenerate_transliterator(&mut self) {
 		let _ = self.sender.send(TransliterateRequest::Die);
		let sender = TextTransliterateAsync::generate_transliterator();
		self.sender = sender;
	}

	pub fn transliterate<S: Into<String>>(&mut self, text: S, locale: S) -> Result<String, &'static str> {
		let (sender, receiver): (Sender<Result<String, &'static str>>, Receiver<Result<String, &'static str>>) = mpsc::channel();

		let text = text.into();
		let locale = locale.into();

		let send_result = self.sender.send(TransliterateRequest::Transliterate( TransliterationData {
			text: text.clone(),
			sender: sender,
			locale: locale.clone()
		}));

		if let Err(_) = send_result {
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
}

impl Drop for TextTransliterateAsync {
	fn drop(&mut self) {
		let _ = self.sender.send(TransliterateRequest::Die);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let mut tt = TextTransliterateAsync::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
		if let Ok(result) = result {
			assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
		} else {
			assert!(false);
		}
	}

	#[test]
	fn japanse_dont_crash() {
		let mut tt = TextTransliterateAsync::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "ja_JP.UTF-8");
		if let Ok(result) = result {
			assert_eq!("u  a  o  ss  U  A  O c n ? ?", result);
		} else {
			assert!(false);
		}
	}

	#[test]
	fn chinese_dont_crash() {
		let mut tt = TextTransliterateAsync::new();
		let result = tt.transliterate("ウェブ全体から検索", "zh_CN.UTF-8");
		if let Ok(result) = result {
			assert_eq!("?????????", result);
		} else {
			assert!(false);
		}
	}

	#[test]
	fn coins() {
		let mut tt = TextTransliterateAsync::new();
		let result = tt.transliterate("€ £ $ ¥", "en_US.UTF-8");
		if let Ok(result) = result {
			assert_eq!("EUR GBP $ JPY", result);
		} else {
			assert!(false);
		}
	}
}
