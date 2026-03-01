use proc_macro::TokenStream;
mod data_io;
mod property_tracker;
#[proc_macro_derive(DataIO)]
pub fn derive_data_io(input: TokenStream) -> TokenStream {
    // drive data io macro, which generates a read() method to read the struct from a ByteReader, and a write() method to write the struct to a ByteWriter
    data_io::derive_data_io(input)
}


#[proc_macro_derive(PropertyTracked)]
pub fn derive_tracked(input: TokenStream) -> TokenStream {
    // drive property tracker macro, which generates setters that set is_changed to true when called, and a reset_changed() method to reset it to false
    property_tracker::derive_property_tracker(input)
}
