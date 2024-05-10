use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

/// A procedure marco implements `to_gl_enum` and `from_gl_enum` methods for a Rust enum.
#[proc_macro_derive(GlEnum, attributes(gl_enum))]
pub fn gl_enum_derive(input: TokenStream) -> TokenStream {
    const ATTR_IDENT: &'static str = "gl_enum";

    let ast: DeriveInput = syn::parse(input).expect("failed to parse");

    let name = ast.ident;
    let Data::Enum(data_enum) = ast.data else {
        panic!("GlEnum only available on enum")
    };
    let variants = data_enum.variants.iter().filter_map(|variant| {
        let mut target = None;
        variant.attrs.iter().for_each(|attr| {
            if attr.path().is_ident(ATTR_IDENT) {
                if target.is_some() {
                    panic!("multiple gl_enum attributes unavailable")
                }
            } else {
                return;
            }

            attr.parse_nested_meta(|meta| {
                let Some(ident) = meta.path.get_ident() else {
                    panic!("unrecognized gl_enum");
                };

                if target.is_some() {
                    panic!("multiple gl_enum attributes unavailable")
                }

                target = Some(ident.clone());

                Ok(())
            })
            .unwrap();
        });

        match target {
            Some(attr) => Some((&variant.ident, attr)),
            None => None,
        }
    });
    let variant1 = variants.clone().map(|(k, _)| k);
    let variant2 = variants.clone().map(|(k, _)| k);
    let gl_enum1 = variants.clone().map(|(_, v)| v);
    let gl_enum2 = variants.map(|(_, v)| v);

    let gen = quote! {
        impl #name {
            #[inline]
            pub fn to_gl_enum(&self) -> u32 {
                match self {
                    #( Self::#variant1 => web_sys::WebGl2RenderingContext::#gl_enum1, )*
                }
            }

            #[inline]
            pub fn from_gl_enum(value: u32) -> Result<Self, u32> {
                match value {
                    #( web_sys::WebGl2RenderingContext::#gl_enum2 => Ok(Self::#variant2), )*
                    _ => Err(value),
                }
            }
        }
    };

    gen.into()
}
