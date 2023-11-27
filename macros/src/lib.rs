extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, FnArg, ItemFn, LitByteStr, Type};

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

        #[no_mangle]
        #[allow(private_interfaces)]
        pub unsafe extern "C" fn stateful_init(n: u8) -> *mut #ident {
            match <#ident as fateful_peripheral::Peripheral>::init(n) {
                Ok(state) => &mut **::std::mem::ManuallyDrop::new(::std::boxed::Box::new(state)) as *mut #ident,
                Err(err) => {
                    fateful_peripheral::update_last_error(err);
                    ::std::ptr::null_mut()
                }
            }
        }

        #[no_mangle]
        #[allow(private_interfaces)]
        pub unsafe extern "C" fn stateful_read(state: *mut #ident, n: u8) -> u8 {
            let mut boxed = ::std::mem::ManuallyDrop::new(::std::boxed::Box::from_raw(state));
            <#ident as fateful_peripheral::Peripheral>::read(&mut boxed, n)
        }

        #[no_mangle]
        #[allow(private_interfaces)]
        pub unsafe extern "C" fn stateful_write(state: *mut #ident, n: u8, data: u8) {
            let mut boxed = ::std::mem::ManuallyDrop::new(::std::boxed::Box::from_raw(state));
            <#ident as fateful_peripheral::Peripheral>::write(&mut boxed, n, data);
        }

        #[no_mangle]
        #[allow(private_interfaces)]
        pub unsafe extern "C" fn stateful_drop(state: *mut #ident) {
            let boxed = ::std::boxed::Box::from_raw(state);
            <#ident as fateful_peripheral::Peripheral>::drop(*boxed);
        }

        #cname
    }
    .into()
}

#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::parse::Error::new(Span::call_site(), "this macro accepts no arguments")
            .to_compile_error()
            .into();
    }

    let f = parse_macro_input!(item as ItemFn);
    let fnspan = f.span();

    let mut valid = f.sig.unsafety.is_some()
        && f.sig.ident == "init"
        && matches!(f.vis, syn::Visibility::Inherited)
        && f.sig.variadic.is_none()
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && match f.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ref ty) => match **ty {
                syn::Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                _ => false,
            },
        };

    let mut inputs = f.sig.inputs.iter();

    match inputs.next() {
        Some(var) => valid &= check_type(var, "u8"),
        _ => valid = false,
    }

    if !valid {
        return syn::parse::Error::new(
            fnspan,
            "expected a function signature of `unsafe fn init(u8)`",
        )
        .to_compile_error()
        .into();
    }

    let inputs = f.sig.inputs;
    let block = f.block;

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn init(#inputs) #block
    }
    .into()
}

#[proc_macro_attribute]
pub fn read(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::parse::Error::new(Span::call_site(), "this macro accepts no arguments")
            .to_compile_error()
            .into();
    }

    let f = parse_macro_input!(item as ItemFn);
    let fnspan = f.span();

    let mut valid = f.sig.unsafety.is_some()
        && f.sig.ident == "read"
        && matches!(f.vis, syn::Visibility::Inherited)
        && f.sig.variadic.is_none()
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && match f.sig.output {
            syn::ReturnType::Default => false,
            syn::ReturnType::Type(_, ref ty) => match **ty {
                Type::Path(ref path) => path
                    .path
                    .get_ident()
                    .map(|ty| ty.to_string() == "u8")
                    .unwrap_or(false),
                Type::Tuple(ref tup) => tup
                    .elems
                    .first()
                    .map(|ty| {
                        if let Type::Path(ref path) = ty {
                            path.path
                                .get_ident()
                                .map(|ty| ty.to_string() == "u8")
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false),
                _ => false,
            },
        };

    let mut inputs = f.sig.inputs.iter();

    match inputs.next() {
        Some(var) => valid &= check_type(var, "u8"),
        _ => valid = false,
    }

    if !valid {
        return syn::parse::Error::new(
            fnspan,
            "expected a function signature of `unsafe fn read(u8) -> u8`",
        )
        .to_compile_error()
        .into();
    }

    let inputs = f.sig.inputs;
    let block = f.block;

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn read(#inputs) -> u8 #block
    }
    .into()
}

#[proc_macro_attribute]
pub fn write(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::parse::Error::new(Span::call_site(), "this macro accepts no arguments")
            .to_compile_error()
            .into();
    }

    let f = parse_macro_input!(item as ItemFn);
    let fnspan = f.span();

    let mut valid = f.sig.unsafety.is_some()
        && f.sig.ident == "write"
        && f.sig.inputs.len() == 2
        && matches!(f.vis, syn::Visibility::Inherited)
        && f.sig.variadic.is_none()
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && match f.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ref ty) => match **ty {
                syn::Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                _ => false,
            },
        };

    let mut inputs = f.sig.inputs.iter();

    match inputs.next() {
        Some(var) => valid &= check_type(var, "u8"),
        _ => valid = false,
    }

    match inputs.next() {
        Some(var) => valid &= check_type(var, "u8"),
        _ => valid = false,
    }

    if !valid {
        return syn::parse::Error::new(
            fnspan,
            "expected a function signature of `unsafe fn write(u8, u8)`",
        )
        .to_compile_error()
        .into();
    }

    let inputs = f.sig.inputs;
    let block = f.block;

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn write(#inputs) #block
    }
    .into()
}

#[proc_macro_attribute]
pub fn drop(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::parse::Error::new(Span::call_site(), "this macro accepts no arguments")
            .to_compile_error()
            .into();
    }

    let f = parse_macro_input!(item as ItemFn);
    let fnspan = f.span();

    let valid = f.sig.unsafety.is_some()
        && f.sig.inputs.is_empty()
        && f.sig.ident == "drop"
        && matches!(f.vis, syn::Visibility::Inherited)
        && f.sig.variadic.is_none()
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && match f.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ref ty) => match **ty {
                syn::Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                _ => false,
            },
        };

    if !valid {
        return syn::parse::Error::new(
            fnspan,
            "expected a function signature of `unsafe fn drop()`",
        )
        .to_compile_error()
        .into();
    }

    let block = f.block;

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn drop() #block
    }
    .into()
}

fn check_type(fn_arg: &FnArg, ty: &str) -> bool {
    if let FnArg::Typed(arg) = fn_arg {
        if let Type::Path(ref path) = *arg.ty {
            if let Some(ident) = path.path.get_ident() {
                return ident.to_string() == ty;
            }
        }
    }
    false
}
