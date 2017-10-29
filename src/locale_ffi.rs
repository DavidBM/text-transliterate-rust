/* automatically generated by rust-bindgen */

pub const _LOCALE_H: ::std::os::raw::c_uint = 1;
pub const _FEATURES_H: ::std::os::raw::c_uint = 1;
pub const _DEFAULT_SOURCE: ::std::os::raw::c_uint = 1;
pub const __USE_ISOC11: ::std::os::raw::c_uint = 1;
pub const __USE_ISOC99: ::std::os::raw::c_uint = 1;
pub const __USE_ISOC95: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX_IMPLICITLY: ::std::os::raw::c_uint = 1;
pub const _POSIX_SOURCE: ::std::os::raw::c_uint = 1;
pub const _POSIX_C_SOURCE: ::std::os::raw::c_uint = 200809;
pub const __USE_POSIX: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX2: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX199309: ::std::os::raw::c_uint = 1;
pub const __USE_POSIX199506: ::std::os::raw::c_uint = 1;
pub const __USE_XOPEN2K: ::std::os::raw::c_uint = 1;
pub const __USE_XOPEN2K8: ::std::os::raw::c_uint = 1;
pub const _ATFILE_SOURCE: ::std::os::raw::c_uint = 1;
pub const __USE_MISC: ::std::os::raw::c_uint = 1;
pub const __USE_ATFILE: ::std::os::raw::c_uint = 1;
pub const __USE_FORTIFY_LEVEL: ::std::os::raw::c_uint = 0;
pub const _STDC_PREDEF_H: ::std::os::raw::c_uint = 1;
pub const __STDC_IEC_559__: ::std::os::raw::c_uint = 1;
pub const __STDC_IEC_559_COMPLEX__: ::std::os::raw::c_uint = 1;
pub const __STDC_ISO_10646__: ::std::os::raw::c_uint = 201605;
pub const __STDC_NO_THREADS__: ::std::os::raw::c_uint = 1;
pub const __GNU_LIBRARY__: ::std::os::raw::c_uint = 6;
pub const __GLIBC__: ::std::os::raw::c_uint = 2;
pub const __GLIBC_MINOR__: ::std::os::raw::c_uint = 24;
pub const _SYS_CDEFS_H: ::std::os::raw::c_uint = 1;
pub const __WORDSIZE: ::std::os::raw::c_uint = 64;
pub const __WORDSIZE_TIME64_COMPAT32: ::std::os::raw::c_uint = 1;
pub const __SYSCALL_WORDSIZE: ::std::os::raw::c_uint = 64;
pub const _BITS_LOCALE_H: ::std::os::raw::c_uint = 1;
pub const __LC_CTYPE: ::std::os::raw::c_uint = 0;
pub const __LC_NUMERIC: ::std::os::raw::c_uint = 1;
pub const __LC_TIME: ::std::os::raw::c_uint = 2;
pub const __LC_COLLATE: ::std::os::raw::c_uint = 3;
pub const __LC_MONETARY: ::std::os::raw::c_uint = 4;
pub const __LC_MESSAGES: ::std::os::raw::c_uint = 5;
pub const __LC_ALL: ::std::os::raw::c_uint = 6;
pub const __LC_PAPER: ::std::os::raw::c_uint = 7;
pub const __LC_NAME: ::std::os::raw::c_uint = 8;
pub const __LC_ADDRESS: ::std::os::raw::c_uint = 9;
pub const __LC_TELEPHONE: ::std::os::raw::c_uint = 10;
pub const __LC_MEASUREMENT: ::std::os::raw::c_uint = 11;
pub const __LC_IDENTIFICATION: ::std::os::raw::c_uint = 12;
#[allow(dead_code)]
pub const LC_CTYPE: ::std::os::raw::c_uint = 0;
#[allow(dead_code)]
pub const LC_NUMERIC: ::std::os::raw::c_uint = 1;
#[allow(dead_code)]
pub const LC_TIME: ::std::os::raw::c_uint = 2;
#[allow(dead_code)]
pub const LC_COLLATE: ::std::os::raw::c_uint = 3;
#[allow(dead_code)]
pub const LC_MONETARY: ::std::os::raw::c_uint = 4;
#[allow(dead_code)]
pub const LC_MESSAGES: ::std::os::raw::c_uint = 5;
#[allow(dead_code)]
pub const LC_ALL: ::std::os::raw::c_uint = 6;
#[allow(dead_code)]
pub const LC_PAPER: ::std::os::raw::c_uint = 7;
#[allow(dead_code)]
pub const LC_NAME: ::std::os::raw::c_uint = 8;
#[allow(dead_code)]
pub const LC_ADDRESS: ::std::os::raw::c_uint = 9;
#[allow(dead_code)]
pub const LC_TELEPHONE: ::std::os::raw::c_uint = 10;
#[allow(dead_code)]
pub const LC_MEASUREMENT: ::std::os::raw::c_uint = 11;
#[allow(dead_code)]
pub const LC_IDENTIFICATION: ::std::os::raw::c_uint = 12;
pub const _XLOCALE_H: ::std::os::raw::c_uint = 1;
#[allow(dead_code)]
pub const LC_CTYPE_MASK: ::std::os::raw::c_uint = 1;
#[allow(dead_code)]
pub const LC_NUMERIC_MASK: ::std::os::raw::c_uint = 2;
#[allow(dead_code)]
pub const LC_TIME_MASK: ::std::os::raw::c_uint = 4;
#[allow(dead_code)]
pub const LC_COLLATE_MASK: ::std::os::raw::c_uint = 8;
#[allow(dead_code)]
pub const LC_MONETARY_MASK: ::std::os::raw::c_uint = 16;
#[allow(dead_code)]
pub const LC_MESSAGES_MASK: ::std::os::raw::c_uint = 32;
#[allow(dead_code)]
pub const LC_PAPER_MASK: ::std::os::raw::c_uint = 128;
#[allow(dead_code)]
pub const LC_NAME_MASK: ::std::os::raw::c_uint = 256;
#[allow(dead_code)]
pub const LC_ADDRESS_MASK: ::std::os::raw::c_uint = 512;
#[allow(dead_code)]
pub const LC_TELEPHONE_MASK: ::std::os::raw::c_uint = 1024;
#[allow(dead_code)]
pub const LC_MEASUREMENT_MASK: ::std::os::raw::c_uint = 2048;
#[allow(dead_code)]
pub const LC_IDENTIFICATION_MASK: ::std::os::raw::c_uint = 4096;
#[allow(dead_code)]
pub const LC_ALL_MASK: ::std::os::raw::c_int = 8127;
#[repr(C)]
#[derive(Debug, Copy)]
#[allow(dead_code)]
pub struct lconv {
    pub decimal_point: *mut ::std::os::raw::c_char,
    pub thousands_sep: *mut ::std::os::raw::c_char,
    pub grouping: *mut ::std::os::raw::c_char,
    pub int_curr_symbol: *mut ::std::os::raw::c_char,
    pub currency_symbol: *mut ::std::os::raw::c_char,
    pub mon_decimal_point: *mut ::std::os::raw::c_char,
    pub mon_thousands_sep: *mut ::std::os::raw::c_char,
    pub mon_grouping: *mut ::std::os::raw::c_char,
    pub positive_sign: *mut ::std::os::raw::c_char,
    pub negative_sign: *mut ::std::os::raw::c_char,
    pub int_frac_digits: ::std::os::raw::c_char,
    pub frac_digits: ::std::os::raw::c_char,
    pub p_cs_precedes: ::std::os::raw::c_char,
    pub p_sep_by_space: ::std::os::raw::c_char,
    pub n_cs_precedes: ::std::os::raw::c_char,
    pub n_sep_by_space: ::std::os::raw::c_char,
    pub p_sign_posn: ::std::os::raw::c_char,
    pub n_sign_posn: ::std::os::raw::c_char,
    pub int_p_cs_precedes: ::std::os::raw::c_char,
    pub int_p_sep_by_space: ::std::os::raw::c_char,
    pub int_n_cs_precedes: ::std::os::raw::c_char,
    pub int_n_sep_by_space: ::std::os::raw::c_char,
    pub int_p_sign_posn: ::std::os::raw::c_char,
    pub int_n_sign_posn: ::std::os::raw::c_char,
}
#[test]
fn bindgen_test_layout_lconv() {
    assert_eq!(::std::mem::size_of::<lconv>() , 96usize , concat ! (
               "Size of: " , stringify ! ( lconv ) ));
    assert_eq! (::std::mem::align_of::<lconv>() , 8usize , concat ! (
                "Alignment of " , stringify ! ( lconv ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . decimal_point as * const _ as
                usize } , 0usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( decimal_point ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . thousands_sep as * const _ as
                usize } , 8usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( thousands_sep ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . grouping as * const _ as
                usize } , 16usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( grouping ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_curr_symbol as * const _
                as usize } , 24usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_curr_symbol ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . currency_symbol as * const _
                as usize } , 32usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( currency_symbol ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . mon_decimal_point as * const
                _ as usize } , 40usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( mon_decimal_point ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . mon_thousands_sep as * const
                _ as usize } , 48usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( mon_thousands_sep ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . mon_grouping as * const _ as
                usize } , 56usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( mon_grouping ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . positive_sign as * const _ as
                usize } , 64usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( positive_sign ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . negative_sign as * const _ as
                usize } , 72usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( negative_sign ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_frac_digits as * const _
                as usize } , 80usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_frac_digits ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . frac_digits as * const _ as
                usize } , 81usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( frac_digits ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . p_cs_precedes as * const _ as
                usize } , 82usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( p_cs_precedes ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . p_sep_by_space as * const _
                as usize } , 83usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( p_sep_by_space ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . n_cs_precedes as * const _ as
                usize } , 84usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( n_cs_precedes ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . n_sep_by_space as * const _
                as usize } , 85usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( n_sep_by_space ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . p_sign_posn as * const _ as
                usize } , 86usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( p_sign_posn ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . n_sign_posn as * const _ as
                usize } , 87usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( n_sign_posn ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_p_cs_precedes as * const
                _ as usize } , 88usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_p_cs_precedes ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_p_sep_by_space as * const
                _ as usize } , 89usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_p_sep_by_space ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_n_cs_precedes as * const
                _ as usize } , 90usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_n_cs_precedes ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_n_sep_by_space as * const
                _ as usize } , 91usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_n_sep_by_space ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_p_sign_posn as * const _
                as usize } , 92usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_p_sign_posn ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const lconv ) ) . int_n_sign_posn as * const _
                as usize } , 93usize , concat ! (
                "Alignment of field: " , stringify ! ( lconv ) , "::" ,
                stringify ! ( int_n_sign_posn ) ));
}
impl Clone for lconv {
    fn clone(&self) -> Self { *self }
}
extern "C" {
    #[allow(dead_code)]
    pub fn setlocale(__category: ::std::os::raw::c_int,
                     __locale: *const ::std::os::raw::c_char)
     -> *mut ::std::os::raw::c_char;
}
extern "C" {
    #[allow(dead_code)]
    pub fn localeconv() -> *mut lconv;
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct __locale_struct {
    pub __locales: [*mut __locale_data; 13usize],
    pub __ctype_b: *const ::std::os::raw::c_ushort,
    pub __ctype_tolower: *const ::std::os::raw::c_int,
    pub __ctype_toupper: *const ::std::os::raw::c_int,
    pub __names: [*const ::std::os::raw::c_char; 13usize],
}
#[test]
#[allow(non_snake_case)]
fn bindgen_test_layout___locale_struct() {
    assert_eq!(::std::mem::size_of::<__locale_struct>() , 232usize , concat !
               ( "Size of: " , stringify ! ( __locale_struct ) ));
    assert_eq! (::std::mem::align_of::<__locale_struct>() , 8usize , concat !
                ( "Alignment of " , stringify ! ( __locale_struct ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const __locale_struct ) ) . __locales as *
                const _ as usize } , 0usize , concat ! (
                "Alignment of field: " , stringify ! ( __locale_struct ) ,
                "::" , stringify ! ( __locales ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const __locale_struct ) ) . __ctype_b as *
                const _ as usize } , 104usize , concat ! (
                "Alignment of field: " , stringify ! ( __locale_struct ) ,
                "::" , stringify ! ( __ctype_b ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const __locale_struct ) ) . __ctype_tolower as
                * const _ as usize } , 112usize , concat ! (
                "Alignment of field: " , stringify ! ( __locale_struct ) ,
                "::" , stringify ! ( __ctype_tolower ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const __locale_struct ) ) . __ctype_toupper as
                * const _ as usize } , 120usize , concat ! (
                "Alignment of field: " , stringify ! ( __locale_struct ) ,
                "::" , stringify ! ( __ctype_toupper ) ));
    assert_eq! (unsafe {
                & ( * ( 0 as * const __locale_struct ) ) . __names as * const
                _ as usize } , 128usize , concat ! (
                "Alignment of field: " , stringify ! ( __locale_struct ) ,
                "::" , stringify ! ( __names ) ));
}
impl Clone for __locale_struct {
    fn clone(&self) -> Self { *self }
}
#[allow(non_camel_case_types)]
pub type __locale_t = *mut __locale_struct;
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub type locale_t = __locale_t;
extern "C" {
    pub fn newlocale(__category_mask: ::std::os::raw::c_int,
                     __locale: *const ::std::os::raw::c_char,
                     __base: __locale_t) -> __locale_t;
}
extern "C" {
    #[allow(dead_code)]
    pub fn duplocale(__dataset: __locale_t) -> __locale_t;
}
extern "C" {
    #[allow(dead_code)]
    pub fn freelocale(__dataset: __locale_t);
}
extern "C" {
    #[allow(dead_code)]
    pub fn uselocale(__dataset: __locale_t) -> __locale_t;
}
#[repr(C)]
#[derive(Debug, Copy)]
pub struct __locale_data {
    pub _address: u8,
}
impl Clone for __locale_data {
    fn clone(&self) -> Self { *self }
}
