use quote::quote;

pub(crate) fn bad_request_static_response_extender(
    ast: &syn::DeriveInput,
) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::atom_core::router::response::StaticResponseExtender for #name
            #ty_generics #where_clause
        {
            type ResBody = ::atom_core::body::Body;

            fn extend(state: &mut ::atom_core::state::State, res: &mut ::atom_core::hyper::Response<Self::ResBody>) {
                res.headers_mut().insert(::atom_core::helpers::http::header::X_REQUEST_ID,
                                         ::atom_core::state::request_id(state).parse().unwrap());
                *res.status_mut() = ::atom_core::hyper::StatusCode::BAD_REQUEST;
            }
        }
    };

    expanded.into()
}
