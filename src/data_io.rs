use proc_macro::TokenStream;
use quote::quote;
use regex;
use syn::{Data, DeriveInput, Fields, parse_macro_input};
#[derive(Debug, Default)]
struct PropertyInfo {
    // Eg: _0_id_4 - index 0, name id, type u32, len -1 (not array)
    property_index: i32,
    property_full_name: String,
    property_name: String,
    property_type: String,
    property_len: i32,
}
const PROPERTY_PARTERN: &str = "_([0-9]+)_([a-zA-Z]+)(?:_((0[Xx])[0-9a-fA-F]+|([0-9]+))_?)?";
fn extract_properties(input: &DeriveInput) -> Vec<PropertyInfo> {
    let mut properties = Vec::new();
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            for field in &fields_named.named {
                let name = field.ident.as_ref().unwrap().to_string();
                let ty = field.ty.clone();
                regex::Regex::new(PROPERTY_PARTERN)
                    .unwrap()
                    .captures(&name)
                    .map(|cap| {
                        let mut property_info = PropertyInfo::default();
                        property_info.property_full_name = name.clone();
                        match &ty {
                            syn::Type::Path(field_type) => {
                                if let Some(segment) = field_type.path.segments.last() {
                                    property_info.property_type = segment.ident.to_string();
                                    if property_info.property_type == "Vec" {
                                        // Lấy kiểu bên trong Vec<T>
                                        if let syn::PathArguments::AngleBracketed(args) =
                                            &segment.arguments
                                        {
                                            if let Some(syn::GenericArgument::Type(
                                                syn::Type::Path(inner),
                                            )) = args.args.first()
                                            {
                                                if let Some(inner_segment) =
                                                    inner.path.segments.last()
                                                {
                                                    property_info.property_type =
                                                        format!("vec,{}", inner_segment.ident);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            syn::Type::Array(array_type) => {
                                // Handle array types like [u8; 8]
                                if let syn::Type::Path(elem_type) = &*array_type.elem {
                                    if let Some(segment) = elem_type.path.segments.last() {
                                        property_info.property_type =
                                            format!("array,{}", segment.ident.to_string());
                                    }
                                }
                                // Extract array length
                                if let syn::Expr::Lit(expr_lit) = &array_type.len {
                                    if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                                        property_info.property_len =
                                            lit_int.base10_parse::<i32>().unwrap_or(-1);
                                    }
                                }
                            }
                            _ => {}
                        }
                        // index
                        property_info.property_index = cap[1].parse::<i32>().unwrap_or(-1);
                        if properties.iter().any(|p: &PropertyInfo| {
                            p.property_index == property_info.property_index
                        }) {
                            panic!(
                                "Duplicate property index: {}, field: {}",
                                property_info.property_index, property_info.property_type
                            );
                        }
                        //name and length
                        property_info.property_name = cap[2].to_string();
                        // length (optional)
                        if cap.get(3).is_some() {
                            // Handle hex length (e.g., 0x4) or decimal length
                            let len_str = cap[3].to_string();
                            #[allow(unused_assignments)]
                            let mut temp_len = 0;
                            if len_str.starts_with("0x") || len_str.starts_with("0X") {
                                temp_len = i32::from_str_radix(&len_str[2..], 16).unwrap_or(-1);
                            } else {
                                temp_len = len_str.parse::<i32>().unwrap_or(-1);
                            }
                            if property_info.property_len > 0
                                && property_info.property_len != temp_len
                            {
                                panic!(
                                    "Conflicting length for field {}: name={} and value={}",
                                    property_info.property_full_name,
                                    temp_len,
                                    property_info.property_len
                                );
                            } else {
                                property_info.property_len = temp_len;
                            }
                        } else {
                            if property_info.property_len == 0 {
                                property_info.property_len = -1;
                            }
                        }
                        // Check type and set default length if not specified
                        if property_info.property_index >= 0
                            && property_info.property_len == -1
                            && let syn::Type::Path(field_type) = &ty
                        {
                            if let Some(segment) = field_type.path.segments.last() {
                                let type_name = segment.ident.to_string();
                                match type_name.to_lowercase().as_str() {
                                    "string" => property_info.property_len = 4, // Variable length
                                    "bool" => property_info.property_len = 1,   // Fixed length
                                    "i8" | "u8" => property_info.property_len = 1, // read until end of stream
                                    "i32" | "u32" => property_info.property_len = 4, // Fixed length
                                    "i64" | "u64" => property_info.property_len = 8, // Fixed length
                                    "vec" => {
                                        if property_info.property_type.starts_with("vec,u8") {
                                            property_info.property_len = -1; // read until end of stream
                                        } else {
                                            panic!(
                                                "Unsupported Vec type for field {}: {}, only Vec<u8> with unspecified length is supported for reading until end of stream",
                                                property_info.property_full_name, property_info.property_type
                                            );
                                        }
                                    }
                                    _ => {
                                        // Handle array types like [u8; 8]
                                        if let syn::Type::Array(_) = &ty {
                                            // Already set property_type and property_len above
                                            // No panic here
                                        } else {
                                            panic!(
                                                "PARSE:Unsupported field type: '{}' for {}",
                                                type_name, property_info.property_full_name
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        properties.push(property_info);
                    });
            }
        } else {
            panic!("Only support struct with named fields");
        }
    }
    properties.sort_by_key(|p| p.property_index); // Sort properties by index
    let dupplicate_end: Vec<(i32,&str)> = properties.iter().filter(|p| p.property_len == -1).map(|p| (p.property_index, p.property_full_name.as_str())).collect();
    if dupplicate_end.len() > 1 {
        panic!("Multiple fields with 'read until end of stream' are not supported: {:?}", dupplicate_end.iter().map(|(_, name)| name.to_string()).collect::<Vec<String>>());
    }
    // std::eprintln!("\nproperties: {:?}", properties);
    if dupplicate_end[0].0 < (properties.last().unwrap().property_index) {
        panic!("'read until end of stream' {} must be the last read field, but found {:?}", dupplicate_end[0].1, properties.iter().filter(|p| p.property_index > dupplicate_end[0].0).map(|p| p.property_full_name.as_str()).collect::<Vec<&str>>());
    }
    properties
}
fn create_read(
    field_type: &str,
    field_ident: &proc_macro2::TokenStream,
    property: &PropertyInfo,
) -> proc_macro2::TokenStream {
    let (maintype, subtype) = if field_type.contains(',') {
        let parts: Vec<&str> = field_type.split(',').collect();
        (parts[0], parts[1])
    } else {
        (field_type, "")
    };
    match maintype.to_lowercase().as_str() {
        "string" => quote! {
            self.#field_ident = bytes_rd.read_string().unwrap().to_string();
        },
        "bool" => quote! {
            self.#field_ident = (bytes_rd.read_i8().unwrap() != 0);
        },
        "i8" => quote! {
            self.#field_ident = bytes_rd.read_i8().unwrap();
        },
        "u8" => quote! {
            self.#field_ident = bytes_rd.read_u8().unwrap();
        },
        "i32" => {
            if property.property_len == 4 {
                quote! {
                    self.#field_ident = bytes_rd.read_i32().unwrap();
                }
            } else {
                panic!(
                    "READ: Unsupported length type: '{}', field: {}",
                    field_type, property.property_full_name
                );
            }
        }
        "u32" => {
            if property.property_len == 4 {
                quote! {
                    self.#field_ident = bytes_rd.read_u32().unwrap();
                }
            } else {
                panic!(
                    "READ: Unsupported length type: '{}', field: {}",
                    field_type, property.property_full_name
                );
            }
        }
        "i64" => {
            if property.property_len == 8 {
                quote! {
                    self.#field_ident = bytes_rd.read_i64().unwrap();
                }
            } else {
                panic!(
                    "READ: Unsupported length type: '{}', field: {}",
                    field_type, property.property_full_name
                );
            }
        }
        "u64" => {
            if property.property_len == 8 {
                quote! {
                    self.#field_ident = bytes_rd.read_u64().unwrap();
                }
            } else {
                panic!(
                    "READ: Unsupported length type: '{}', field: {}",
                    field_type, property.property_full_name
                );
            }
        }
        "array" => {
            let _elem_type = subtype;
            let readers = (0..property.property_len).map(|index| {
                let name = &quote::format_ident!("{}", property.property_full_name);
                let index = syn::Index::from(index as usize);
                create_read(_elem_type, &quote::quote! { #name[#index] }, property)
            });
            quote! {
                #(#readers)*
            }
        }
        "vec" => {
            let _elem_type = subtype;
            let len = property.property_len.clone();
            let _elem_qoute = quote::format_ident!("{}", _elem_type);
            if property.property_len > 0 {
                let readers = (0..property.property_len).map(|index| {
                    let name = &quote::format_ident!("{}", property.property_full_name);
                    let index = syn::Index::from(index as usize);
                    create_read(_elem_type, &quote::quote! { #name[#index] }, property)
                });
                
                quote! {
                    self.#field_ident = std::iter::repeat_with(|| #_elem_qoute::default()).take(#len as usize).collect::<Vec<_>>(); // Initialize vec with default values
                    #(#readers)*
                }
            } else if _elem_type == "u8" {
                quote! {
                    let mut buffer = Vec::new();
                    let _ = bytes_rd.read_to_end(&mut buffer).unwrap();
                    self.#field_ident = buffer;
                }
            } else {
                panic!(
                    "READ: Unsupported vec type: '{}<{}>', field: {}, only Vec<u8> with unspecified length is supported for reading until end of stream",
                    maintype, _elem_type, property.property_full_name
                );
            }
        }
        _ => panic!(
            "READ:Unsupported field type: '{}' for {}",
            field_type, property.property_full_name
        ),
    }
}
fn create_write(
    field_type: &str,
    field_ident: &proc_macro2::TokenStream,
    property: &PropertyInfo,
) -> proc_macro2::TokenStream {
    let (maintype, subtype) = if field_type.contains(',') {
        let parts: Vec<&str> = field_type.split(',').collect();
        (parts[0], parts[1])
    } else {
        (field_type, "")
    };
    match maintype.to_lowercase().as_str() {
        "string" => quote! {
            doc.write_string(&self.#field_ident);
        },
        "bool" => quote! {
            doc.write_i8(if self.#field_ident { 1 } else { 0 });
        },
        "i8" => quote! {
            doc.write_i8(self.#field_ident);
        },
        "u8" => quote! {
            doc.write_u8(self.#field_ident);
        },
        "i32" => quote! {
            doc.write_i32(self.#field_ident);
        },
        "u32" => quote! {
            doc.write_u32(self.#field_ident);
        },
        "i64" => quote! {
            doc.write_i64(self.#field_ident);
        },
        "u64" => quote! {
            doc.write_u64(self.#field_ident);
        },
        "array" => {
            let _elem_type = subtype;
            let writers = (0..property.property_len).map(|index| {
                let name = &quote::format_ident!("{}", property.property_full_name);
                let index = syn::Index::from(index as usize);
                create_write(_elem_type, &quote::quote! { #name[#index] }, property)
            });
            quote! {
                #(#writers)*
            }
        }
        "vec" => {
            let _elem_type = subtype;
            let _len = property.property_len.clone();
            let _elem_qoute = quote::format_ident!("{}", _elem_type);
            if property.property_len > 0 {
                let writers = (0..property.property_len).map(|index| {
                    let name = &quote::format_ident!("{}", property.property_full_name);
                    let index = syn::Index::from(index as usize);
                    create_write(_elem_type, &quote::quote! { #name[#index] }, property)
                });
                
                quote! {
                    #(#writers)*
                }
            } else if _elem_type == "u8" {
                quote! {
                    doc.write_bytes(&self.#field_ident);
                }
            } else {
                panic!(
                    "READ: Unsupported vec type: '{}<{}>', field: {}, only Vec<u8> with unspecified length is supported for reading until end of stream",
                    maintype, _elem_type, property.property_full_name
                );
            }
        },
        _ => panic!(
            "WRITE:Unsupported field type: '{}' for {}",
            field_type, property.property_full_name
        ),
    }
}
pub fn derive_data_io(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Lấy danh sách các field
    _ = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("Struct not have named fields"),
        },
        _ => panic!("Only support struct"),
    };
    let properties = extract_properties(&input);
    // read
    let auto_reads = properties.iter().map(|property| {
        let name = quote::format_ident!("{}", property.property_full_name);
        let field_ident = quote::quote! {#name};
        let field_type = &property.property_type;
        create_read(field_type, &field_ident, property)
    });
    // write
    let write = properties.iter().map(|property| {
        let name = quote::format_ident!("{}", property.property_full_name);
        let field_ident = quote::quote! {#name};
        let field_type = &property.property_type;
        create_write(field_type, &field_ident, property)
    });
    // std::eprintln!("\nauto_reads: {:?}", auto_reads.len());
    // std::eprintln!("\nwrite: {:?}", write.len());
    let new_struct = quote! {
        use bytebuffer::{ByteReader, ByteBuffer, Endian}; // Import ByteReader, ByteBuffer, and Endian for the generated code
        impl #struct_name {
            pub fn read(&mut self, bytes_rd: &mut ByteReader) {
                if self.is_big_endian {
                    bytes_rd.set_endian(bytebuffer::Endian::BigEndian);
                } else {
                    bytes_rd.set_endian(bytebuffer::Endian::LittleEndian);
                }
                #(#auto_reads)*
            }
            pub fn write(&self) -> Vec<u8> {
                let mut doc = ByteBuffer::new();
                #(#write)*
                doc.into_vec()
            }
        }
    };

    new_struct.into()
}
