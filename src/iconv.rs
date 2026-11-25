/*
code copyinspired from: git@github.com:y-stm/rust-iconv.git (witch have a MIT license: https://raw.githubusercontent.com/andelf/rust-iconv/9e2fecaa09c5d1d632fc34b0291d31002f3053c0/Cargo.toml)
*/
use errno::{errno, Errno};
use libc::{iconv, iconv_close, iconv_open, iconv_t};
use log::info;
use std::borrow::Cow;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};

/// The error given by iconv.
///
/// This include the below:
/// * Input/output buffer has no sufficient room during conversion.
/// * Failed to create iconv object
/// * Failed to make CString of encoding name (at `Iconv::new`)
#[derive(Debug)]
pub enum IconvError {
    OnFindingConversion(Errno),
    OnCStringConversion(std::ffi::NulError),
    OnConversion(Cow<'static, str>),
    InvalidSequence(usize, usize),
    InsufficientOutBuffer(usize, usize),
    InsufficientInBuffer(usize, usize),
}

impl IconvError {
    /// Returns short description of `IconvError`
    ///
    /// This is used in `(IconvError as Display)::fmt`
    pub fn to_str(&self) -> String {
        use self::IconvError::*;
        match *self {
            OnFindingConversion(ref e) => format!("C function `iconv_open` failed: {}", e),
            OnCStringConversion(ref e) => format!("CString::new failed: {}", e),
            OnConversion(ref cow_str) => cow_str.to_owned().to_string(),
            InsufficientOutBuffer(ref left_to_convert, ref wrote_bytes) => format!(
                "Need more room in dst buffer. {} bytes remain. {} bytes written",
                left_to_convert, wrote_bytes
            ),
            InsufficientInBuffer(ref remain_index, ref wrote_index) => format!(
                "Need more input to complete conversion. {} bytes remain, {} bytes written",
                remain_index, wrote_index
            ),
            InvalidSequence(ref remain_index, ref wrote_index) => format!(
                "Source text has invalid multibyte charactor sequence at {}. {} bytes \
                         written",
                remain_index, wrote_index
            ),
        }
    }
}

impl Display for IconvError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(&self.to_str())
    }
}

impl Error for IconvError {
    fn description(&self) -> &str {
        "Error happend using Iconv"
    }
}

pub struct Iconv {
    iconv: iconv_t,
}

impl Iconv {
    pub fn new(tocode: &str, fromcode: &str) -> Result<Iconv, IconvError> {
        let to_iconv_err = |null_err| Err(IconvError::OnCStringConversion(null_err));
        let tocode_c = CString::new(tocode).or_else(&to_iconv_err)?;
        let fromcode_c = CString::new(fromcode).or_else(&to_iconv_err)?;
        unsafe {
            let cd = iconv_open(tocode_c.as_ptr(), fromcode_c.as_ptr());
            if cd == (-1_isize) as iconv_t {
                return Err(IconvError::OnFindingConversion(errno()));
            }
            Ok(Iconv { iconv: cd })
        }
    }

    /// Convert src sequence into anotherencoding.
    ///
    /// Convert src sequence into another encoding, putting them into dst buffer from start.
    /// # Arguments
    /// * `src` - The input text (&[u8] encoded in "from encoding")
    /// * `dst` - Where the converted text is stored.
    /// # Returns
    /// * `Result::Ok(usize)` - When all text in `src` are converted successfully,
    ///                         it returns the bytes of written to `dst`
    /// * `Result::Err(InsufficientOutBuffer(usize, usize)) -
    ///         If there's no sufficient room in `dst`, returns the index of `src` where conversion
    ///         is processed and the index of `dst` where the converted text is written.
    pub fn convert_raw(&mut self, src: &[u8], dst: &mut [u8]) -> Result<usize, IconvError> {
        let mut input = src.as_ptr().cast_mut().cast();
        let mut output = dst.as_mut_ptr().cast();
        let mut input_left = src.len();
        let mut output_left = dst.len();
        // The buffers are valid for the supplied lengths. iconv updates the local
        // pointers and counts and only writes to the destination buffer.
        let result = unsafe {
            iconv(
                self.iconv,
                &mut input,
                &mut input_left,
                &mut output,
                &mut output_left,
            )
        };
        let read = src.len() - input_left;
        let written = dst.len() - output_left;
        if result != usize::MAX {
            return Ok(written);
        }

        match errno().0 {
            libc::E2BIG => Err(IconvError::InsufficientOutBuffer(read, written)),
            libc::EILSEQ => Err(IconvError::InvalidSequence(read, written)),
            libc::EINVAL => Err(IconvError::InsufficientInBuffer(read, written)),
            _ => Err(IconvError::OnConversion(Cow::Borrowed("Unknown Error"))),
        }
    }

    /// Converts text into another encoding.
    ///
    /// #Arguments
    /// * `src` - The input text
    /// * `dst` - The output text encoded in this struct.
    /// * `start_index` - The index of `dst` where output will be written.
    ///
    /// #Returns
    /// * Ok(unit) - Returns when all text in `src` has converted into `dst`
    /// * IconvError(InsufficientInBuffer(usize, usize))
    ///     - Returns when `src` has incomplete sequence.
    ///       It may be recovered by adding more sequence of input.
    /// * IconvError(InvalidSequence(usize, usize))
    ///     - Returns when `src` has invalid multibyte sequence.
    /// * IconvError(OnConversion(_)) - Something wrong happend due to any other reason
    pub fn convert(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
        start_index: usize,
    ) -> Result<(), IconvError> {
        let mut src_index = 0;
        let mut dst_index = start_index;
        dst.resize(start_index + src.len(), 0);
        loop {
            match self.convert_raw(&src[src_index..], &mut dst[dst_index..]) {
                Ok(written) => {
                    dst.truncate(dst_index + written);
                    return Ok(());
                }
                Err(IconvError::InsufficientOutBuffer(read, written)) => {
                    src_index += read;
                    dst_index += written;
                    dst.resize(dst.len() + dst.len().max(16), 0);
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for Iconv {
    fn drop(&mut self) {
        unsafe {
            if iconv_close(self.iconv) == -1 {
                info!("Error on disposing iconv descriptor: {}", errno());
            }
        }
    }
}

#[test]
fn test_iconv_raw() {
    let mut iconv = Iconv::new("cp932", "utf-8").unwrap();
    let src = "あいうえお".bytes().collect::<Vec<u8>>();
    let mut outbuf = [0u8; 1000];
    let res = iconv.convert_raw(&src, &mut outbuf).unwrap();
    info!("First result: {}", res);
    info!("CP932 converted: {:?}", &outbuf[0..11]);
    let mut outbuf2 = [0u8; 1000];
    let mut iconv = Iconv::new("utf-8", "cp932").unwrap();
    let res2 = iconv.convert_raw(&outbuf[0..res], &mut outbuf2).unwrap();
    info!("Second result: {}", res2);
    let s = String::from_utf8_lossy(&outbuf2[0..res2]);
    info!("Recoverd: {}", s);
    assert_eq!(&s, "あいうえお");
}

/// Confirm that `Iconv::convert_raw` set errno to E2BIG when output buffer has no room
#[test]
fn what_if_dst_array_is_short() {
    let mut iconv = Iconv::new("cp932", "utf-8").unwrap();
    let src = "あいうえお".bytes().collect::<Vec<u8>>();
    let mut outbuf = [0u8; 4];
    info!("Let's begin shortcomming\n");
    if let Err(IconvError::InsufficientOutBuffer(_, _)) = iconv.convert_raw(&src, &mut outbuf) {
    } else {
        unreachable!();
    }
}

/// Test `Iconv::convert` gives the same result as the input through utf-8 -> cp932 -> utf-8
/// conversions
#[test]
fn test_convert_raw_turn() {
    let mut iconv = Iconv::new("cp932", "utf-8").unwrap();
    let mut iconv_rev = Iconv::new("utf-8", "cp932").unwrap();
    let src = "あいうえお".bytes().collect::<Vec<u8>>();
    let mut dst = Vec::new();
    let res = iconv.convert(&src, &mut dst, 0);
    match res {
        Ok(_) => {}
        _ => {
            panic!("Conversion failed");
        }
    }
    info!("line {}: Result of convert: {:?}", line!(), dst);
    let mut dst2 = Vec::new();
    let _ = iconv_rev.convert(&dst, &mut dst2, 0);
    let s_recoverd = String::from_utf8(dst2).unwrap();
    assert_eq!("あいうえお".to_string(), s_recoverd);
}
