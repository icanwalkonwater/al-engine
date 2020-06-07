use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::iter::FromIterator;
use syn::{Data, DataStruct, Fields, FieldsNamed, Meta, Ident};

type VertexLocation = u32;

struct VertexAttribute {
    ident: Ident,
    location: VertexLocation,
}

/// Can be used to provide an implementation of `[Vertex]`.
/// Usage:
/// ```rust,norun
/// #[derive(Vertex)
/// struct MyVertex {
///     #[location = 0]
///     position: [f32; 3],
///     #[location = 1]
///     color: [f32; 3],
/// }
/// ```
#[proc_macro_derive(Vertex)]
pub fn vertex_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse AST !");

    impl_vertex_macro(ast)
}

fn impl_vertex_macro(ast: syn::DeriveInput) -> TokenStream {
    match ast.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    let attributes = extract_vertex_attributes(fields);

                    // Sanity checks: unique locations
                    {
                        let unique = HashSet::from_iter(
                            attributes
                                .iter()
                                .map(|VertexAttribute { location, .. }| *location),
                        );

                        if unique.len() != attributes.len() {
                            panic!("You have a duplicate location attribute !");
                        }
                    }

                    gen_vertex_impl(ast.ident, &attributes)
                }
                _ => unreachable!(),
            }
        }
        _ => panic!("Can only derive structs !"),
    }
}

fn extract_vertex_attributes(fields: FieldsNamed) -> Vec<VertexAttribute> {
    fields
        .named
        .iter()
        .filter_map(|field| {
            // Find location attribute
            let location_attr = field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("location"))
                .expect("All attributes must have a `#[location = X]` attribute !");

            if let Some(location_attr) = location_attr {
                // Parse attribute
                // Must be a name value aka the format #[location = 0]
                if let Ok(Meta::NameValue(name_value)) = location_attr.parse_meta() {
                    // The argument must be an u32
                    if let syn::Lit::Int(location) = name_value.lit {
                        // Parse the argument
                        let location = location
                            .base10_parse::<VertexLocation>()
                            .expect("Failed to parse location attribute: must be a u32 !");

                        // Done, get the name of the field on the fly.
                        Some(VertexAttribute {
                            ident: field.ident.unwrap(),
                            location,
                        })
                    } else {
                        panic!("Failed to parse location attribute: must be a u32 !");
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

macro_rules! impl_vertex {
    (
        $type:ty;
        $( layout(location = $location:literal) in $format:ident $attribute:ident; )*
    ) => {
        impl $crate::renderer::vertex::Vertex for $type {
            fn get_binding_descriptions() -> [ash::vk::VertexInputBindingDescription; 1] {
                [
                    ash::vk::VertexInputBindingDescription::builder()
                        .binding(0)
                        .stride(std::mem::size_of::<Self>() as u32)
                        .input_rate(vk::VertexInputRate::VERTEX)
                        .build()
                ]
            }

            fn get_attribute_descriptions() -> Vec<ash::vk::VertexInputAttributeDescription> {
                vec![$(
                    ash::vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location($location)
                        .format(vulkan_format_trans!($format))
                        .offset(memoffset::offset_of!(Self, $attribute) as u32)
                        .build(),
                )*]
            }
        }
    };
}

fn gen_vertex_impl(ty: Ident, attributes: &[VertexAttribute]) -> TokenStream {
    let (attribute_names, attribute_locations) = attributes.into_iter()
        .map(|&vertex| (vertex.ident, vertex.location))
        .unzip();
    let len = attributes.len();

    let gen = quote! {
        impl Vertex for #ty {
            fn get_binding_descriptions() -> [ash::vk::VertexInputBindingDescription; 1] {
                [
                    ash::vk::VertexInputBindingDescription::builder()
                        .binding(0)
                        .stride(std::mem::size_of::<Self>() as u32)
                        .input_rate(vk::VertexInputRate::VERTEX)
                        .build()
                ]
            }

            fn get_attribute_descriptions() -> [ash::vk::VertexInputAtributeDescription; #len] {
                #(
                    [
                        ash::vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location(#attribute_locations)
                        .format() // fuck
                    ]
                ),*
            }
        }
    };
}