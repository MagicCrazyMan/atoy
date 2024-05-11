use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput};

/// A procedure macro implements
#[proc_macro_derive(AsAny)]
pub fn as_any_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("failed to parse");
    let name = ast.ident;
    quote! {
        impl crate::core::AsAny for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    }
    .into()
}

/// A procedure macro implements `to_gl_enum` and `from_gl_enum` methods for a Rust enum.
#[proc_macro_derive(GlEnum, attributes(gl_enum))]
pub fn gl_enum_derive(input: TokenStream) -> TokenStream {
    const ATTRIBUTE_NAME: &'static str = "gl_enum";
    const TEXTURE_MAX_ANISOTROPY_EXT: &'static str = "TEXTURE_MAX_ANISOTROPY_EXT";
    const ASTC: [&'static str; 28] = [
        "COMPRESSED_RGBA_ASTC_4X4_KHR",
        "COMPRESSED_RGBA_ASTC_5X4_KHR",
        "COMPRESSED_RGBA_ASTC_5X5_KHR",
        "COMPRESSED_RGBA_ASTC_6X5_KHR",
        "COMPRESSED_RGBA_ASTC_6X6_KHR",
        "COMPRESSED_RGBA_ASTC_8X5_KHR",
        "COMPRESSED_RGBA_ASTC_8X6_KHR",
        "COMPRESSED_RGBA_ASTC_8X8_KHR",
        "COMPRESSED_RGBA_ASTC_10X5_KHR",
        "COMPRESSED_RGBA_ASTC_10X6_KHR",
        "COMPRESSED_RGBA_ASTC_10X8_KHR",
        "COMPRESSED_RGBA_ASTC_10X10_KHR",
        "COMPRESSED_RGBA_ASTC_12X10_KHR",
        "COMPRESSED_RGBA_ASTC_12X12_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_4X4_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_5X4_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_5X5_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_6X5_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_6X6_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_8X5_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_8X6_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_8X8_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_10X5_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_10X6_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_10X8_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_10X10_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_12X10_KHR",
        "COMPRESSED_SRGB8_ALPHA8_ASTC_12X12_KHR",
    ];
    const ETC: [&'static str; 10] = [
        "COMPRESSED_R11_EAC",
        "COMPRESSED_SIGNED_R11_EAC",
        "COMPRESSED_RG11_EAC",
        "COMPRESSED_SIGNED_RG11_EAC",
        "COMPRESSED_RGB8_ETC2",
        "COMPRESSED_RGBA8_ETC2_EAC",
        "COMPRESSED_SRGB8_ETC2",
        "COMPRESSED_SRGB8_ALPHA8_ETC2_EAC",
        "COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2",
        "COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2",
    ];
    const ETC1: [&'static str; 1] = ["COMPRESSED_RGB_ETC1_WEBGL"];
    const PVRTC: [&'static str; 4] = [
        "COMPRESSED_RGB_PVRTC_2BPPV1_IMG",
        "COMPRESSED_RGBA_PVRTC_2BPPV1_IMG",
        "COMPRESSED_RGB_PVRTC_4BPPV1_IMG",
        "COMPRESSED_RGBA_PVRTC_4BPPV1_IMG",
    ];
    const S3TC: [&'static str; 4] = [
        "COMPRESSED_RGB_S3TC_DXT1_EXT",
        "COMPRESSED_RGBA_S3TC_DXT1_EXT",
        "COMPRESSED_RGBA_S3TC_DXT3_EXT",
        "COMPRESSED_RGBA_S3TC_DXT5_EXT",
    ];
    const S3TC_SRGB: [&'static str; 4] = [
        "COMPRESSED_SRGB_S3TC_DXT1_EXT",
        "COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT",
        "COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT",
        "COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT",
    ];
    const BPTC: [(&'static str, u32); 4] = [
        ("COMPRESSED_RGBA_BPTC_UNORM_EXT", 36492),
        ("COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT", 36493),
        ("COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT", 36494),
        ("COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT", 36495),
    ];
    const RGTC: [(&'static str, u32); 4] = [
        ("COMPRESSED_RED_RGTC1_EXT", 36283),
        ("COMPRESSED_SIGNED_RED_RGTC1_EXT", 36284),
        ("COMPRESSED_RED_GREEN_RGTC2_EXT", 36285),
        ("COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT", 36286),
    ];

    let astc_set: HashSet<&str> = HashSet::from_iter(ASTC);
    let etc_set: HashSet<&str> = HashSet::from_iter(ETC);
    let etc1_set: HashSet<&str> = HashSet::from_iter(ETC1);
    let pvrtc_set: HashSet<&str> = HashSet::from_iter(PVRTC);
    let s3tc_set: HashSet<&str> = HashSet::from_iter(S3TC);
    let s3tc_srgb_set: HashSet<&str> = HashSet::from_iter(S3TC_SRGB);
    let bptc_map: HashMap<&str, u32> = HashMap::from_iter(BPTC);
    let rgtc_map: HashMap<&str, u32> = HashMap::from_iter(RGTC);

    let ast: DeriveInput = syn::parse(input).expect("failed to parse");

    let name = ast.ident;
    let Data::Enum(data_enum) = ast.data else {
        panic!("GlEnum only available on enum")
    };
    let variants = data_enum.variants.iter().map(|variant| {
        let mut target = None;

        // finds the gl_enum attribute and extracts it.
        variant.attrs.iter().for_each(|attr| {
            if attr.path().is_ident(ATTRIBUTE_NAME) {
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

        // If the target is not specified, use the UPPER_SNAKE case of the variant name as the target.
        let target = match target {
            Some(target) => target,
            None => {
                let gl_enum_name = variant.ident.to_string().to_case(Case::UpperSnake);
                Ident::new(&gl_enum_name, Span::call_site())
            }
        };

        let target_str = target.to_string();
        let target_str = target_str.as_str();

        let gl_enum = if target_str == TEXTURE_MAX_ANISOTROPY_EXT {
            quote! { web_sys::ExtTextureFilterAnisotropic::#target }
        } else if astc_set.contains(target_str) {
            quote! { web_sys::WebglCompressedTextureAstc::#target }
        } else if etc_set.contains(target_str) {
            quote! { web_sys::WebglCompressedTextureEtc::#target }
        } else if etc1_set.contains(target_str) {
            quote! { web_sys::WebglCompressedTextureEtc1::#target }
        } else if pvrtc_set.contains(target_str) {
            quote! { web_sys::WebglCompressedTexturePvrtc::#target }
        } else if s3tc_set.contains(target_str) {
            quote! { web_sys::WebglCompressedTextureS3tc::#target }
        } else if s3tc_srgb_set.contains(target_str) {
            quote! { web_sys::WebglCompressedTextureS3tcSrgb::#target }
        } else if let Some(gl_enum) = bptc_map.get(target_str) {
            quote! { #gl_enum }
        } else if let Some(gl_enum) = rgtc_map.get(target_str) {
            quote! { #gl_enum }
        } else {
            quote! { web_sys::WebGl2RenderingContext::#target }
        };

        (&variant.ident, gl_enum)
    });
    let variant1 = variants.clone().map(|(k, _)| k);
    let variant2 = variants.clone().map(|(k, _)| k);
    let gl_enum1 = variants.clone().map(|(_, v)| v);
    let gl_enum2 = variants.clone().map(|(_, v)| v);

    let gen = quote! {
        impl #name {
            #[inline]
            pub fn to_gl_enum(&self) -> u32 {
                match self {
                    #( Self::#variant1 => #gl_enum1, )*
                }
            }

            #[inline]
            pub fn from_gl_enum(gl_enum: u32) -> Result<Self, u32> {
                match gl_enum {
                    #( #gl_enum2 => Ok(Self::#variant2), )*
                    _ => Err(gl_enum),
                }
            }
        }
    };

    gen.into()
}
