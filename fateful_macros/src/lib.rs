extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitByteStr};

#[proc_macro_attribute]
pub fn peripheral(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut name: Option<LitByteStr> = None;
    let peripheral_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            name = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported peripheral property"))
        }
    });
    parse_macro_input!(attr with peripheral_parser);

    let cname = name.map(|n| {
        let span = n.span();
        let mut value = n.value();
        value.push(b'\0');

        let cname = LitByteStr::new(&value, span);
        quote! {
            #[no_mangle]
            pub const unsafe extern "C" fn name() -> *const ::std::ffi::c_char {
                ::std::ffi::CStr::from_bytes_with_nul_unchecked(#cname).as_ptr()
            }
        }
    });

    let input = parse_macro_input!(item as DeriveInput);
    let ident = input.ident.clone();

    quote! {
        #input

        #[allow(missing_docs)]
        #[doc(hidden)]
        pub mod __fateful_peripheral_export {
            use super::#ident;

            #[no_mangle]
            pub unsafe extern "C" fn stateful_init(n: u8) -> *mut ::std::ffi::c_void {
                match <#ident as fateful_peripheral::Peripheral>::init(n) {
                    Ok(state) => ::std::boxed::Box::into_raw(::std::boxed::Box::new(state)) as *mut ::std::ffi::c_void,
                    Err(err) => {
                        fateful_peripheral::update_last_error(err);
                        ::std::ptr::null_mut()
                    }
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn stateful_read(state: *mut ::std::ffi::c_void, n: u8) -> u8 {
                let mut boxed = ::std::mem::ManuallyDrop::new(::std::boxed::Box::from_raw(state as *mut #ident));
                <#ident as fateful_peripheral::Peripheral>::read(&mut boxed, n)
            }

            #[no_mangle]
            pub unsafe extern "C" fn stateful_write(state: *mut ::std::ffi::c_void, n: u8, data: u8) {
                let mut boxed = ::std::mem::ManuallyDrop::new(::std::boxed::Box::from_raw(state as *mut #ident));
                <#ident as fateful_peripheral::Peripheral>::write(&mut boxed, n, data);
            }

            #[no_mangle]
            pub unsafe extern "C" fn stateful_drop(state: *mut ::std::ffi::c_void) {
                let boxed = ::std::boxed::Box::from_raw(state as *mut #ident);
                <#ident as fateful_peripheral::Peripheral>::drop(*boxed);
            }

            #[no_mangle]
            pub unsafe extern "C" fn stateful_reset(state: *mut ::std::ffi::c_void) {
                let mut boxed = ::std::mem::ManuallyDrop::new(::std::boxed::Box::from_raw(state as *mut #ident));
                <#ident as fateful_peripheral::Peripheral>::reset(&mut boxed);
            }

            #cname

            #[no_mangle]
            pub unsafe extern "C" fn last_error_length() -> ::std::ffi::c_int {
                fateful_peripheral::last_error_length()
            }

            #[no_mangle]
            pub unsafe extern "C" fn get_last_error(buf: *mut ::std::ffi::c_char, length: ::std::ffi::c_int) -> ::std::ffi::c_int {
                fateful_peripheral::get_last_error(buf, length)
            }
        }
    }
    .into()
}
