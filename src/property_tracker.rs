use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_property_tracker(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let mut have_is_changed = false;
    // Lấy danh sách các field
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("Struct not have named fields"),
        },
        _ => panic!("Only support struct"),
    };

    // Sinh getter/setter cho từng field
    let setters = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let kieu_field = &f.ty;
        if field_name == "is_changed" {
            have_is_changed = true;
            quote! {} // Không tạo setter cho is_changed
        } else {
            let setter_name =
                quote::format_ident!("set_{}", field_name.to_string().trim_matches('_')); // Remove leading/trailing underscores for setter name
            quote! {
                pub fn #setter_name(&mut self, value: #kieu_field) {
                    self.#field_name = value;
                    self.is_changed = true;
                }
            }
        }
    });
    let new_struct = quote! {
        impl #struct_name {
            pub fn reset_changed(&mut self) {
                self.is_changed = false;
            }
            #(#setters)*
        }
    };
    new_struct.into()
}
