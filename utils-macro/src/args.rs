use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, Lit, Meta, NestedMeta, Result, Token};

pub struct Args<T> {
    pub vars: Vec<T>,
}

impl<T: Parse> Parse for Args<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<T, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}

pub struct Command;

impl Command {
    pub fn get_varname(ident: &Ident) -> Ident {
        format_ident!("static_command_for_{}", ident.to_string())
    }
}

pub struct Resource;

impl Resource {
    pub fn get_name(ident: &Ident) -> String {
        ident
            .to_string()
            .split("_")
            .skip(1)
            .next()
            .expect("Failed to parse resource name!")
            .to_string()
    }
}

pub fn get_string_value(meta: &NestedMeta, key: &str) -> String {
    let expect_msg = format!("Expected argument `{} = \"...\"`", key);
    let argument_name_and_value = match meta {
        NestedMeta::Meta(Meta::NameValue(meta)) => meta,
        _ => panic!("{}", expect_msg),
    };
    assert_eq!(
        argument_name_and_value
            .path
            .segments
            .first()
            .expect(&expect_msg)
            .ident,
        key
    );
    match &argument_name_and_value.lit {
        Lit::Str(lit) => lit.value(),
        _ => panic!("{} argument must be a string", key),
    }
}
