use transliterate::TextTransliterate;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
enum TransliterateRequestAction {
	Die,
	Transliterate
}

#[derive(Debug)]
struct TransliterationRequest {
	text: String,
	sender: Sender<Result<String, &'static str>>,
	locale: String,
	action: TransliterateRequestAction
}

#[derive(Debug)]
struct TransliterateAsync {
	sender: Sender<TransliterationRequest>
}

impl TransliterateAsync {
	pub fn new() -> TransliterateAsync {
		let (sender, receiver): (Sender<TransliterationRequest>, Receiver<TransliterationRequest>) = mpsc::channel();

		let text_transliterate = TextTransliterate::new();

		TransliterateAsync::create_thread(receiver, text_transliterate);

		TransliterateAsync {sender: sender}
	}

	pub fn transliterate<S: Into<String>>(&self, text: S, locale: S) -> Result<String, &'static str> {
		let (sender, receiver): (Sender<Result<String, &'static str>>, Receiver<Result<String, &'static str>>) = mpsc::channel();

		self.sender.send(TransliterationRequest {
			text: text.into(),
			sender: sender,
			locale: locale.into(),
			action: TransliterateRequestAction::Transliterate
		});

		if let Ok(result) = receiver.recv() {
			match result {
				Ok(text) => Ok(text),
				Err(message) => Err(message),
			}
		} else {
			Err("Error communicating with the thread")
		}

	}

	fn create_thread(receiver: Receiver<TransliterationRequest>, tt: TextTransliterate) {
		thread::spawn(move || {
			while let Ok(request) = receiver.recv() {
				if let Ok(result) = tt.transliterate(request.text, request.locale){
					request.sender.send(Ok(result)).unwrap();
				}
			}
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let tt = TransliterateAsync::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
		if let Ok(result) = result {
			assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
		} else {
			assert!(false);
		}
	}
}
