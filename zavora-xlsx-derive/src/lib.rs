extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

#[proc_macro_derive(ExcelRow, attributes(excel))]
pub fn derive_excel_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_excel_row(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

// FieldConfig: parsed from #[excel(...)] attributes
struct FieldConfig {
    header: Option<String>,
    format: Option<String>,
    skip: bool,
}

// FieldInfo: complete analysis of a struct field
struct FieldInfo {
    ident: syn::Ident,
    ty: syn::Type,
    config: FieldConfig,
    col_index: u16,
    is_option: bool,
    inner_type: Option<syn::Type>,
}

fn parse_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<FieldConfig> {
    let mut config = FieldConfig {
        header: None,
        format: None,
        skip: false,
    };
    for attr in attrs {
        if !attr.path().is_ident("excel") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("header") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                config.header = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("format") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                config.format = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("skip") {
                config.skip = true;
                Ok(())
            } else {
                Err(meta.error(format!(
                    "unknown excel attribute: '{}'",
                    meta.path
                        .get_ident()
                        .map_or("?".to_string(), |i| i.to_string())
                )))
            }
        })?;
    }
    if config.skip && (config.header.is_some() || config.format.is_some()) {
        return Err(syn::Error::new_spanned(
            &attrs[0],
            "'skip' cannot be combined with other excel attributes",
        ));
    }
    Ok(config)
}

fn is_option_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(tp) = ty {
        let seg = tp.path.segments.last()?;
        if seg.ident == "Option"
            && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
            && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
        {
            return Some(inner.clone());
        }
    }
    None
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        return seg.ident == "String";
    }
    false
}

fn is_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        return seg.ident == "bool";
    }
    false
}

fn impl_excel_row(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ExcelRow can only be derived for named structs",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ExcelRow can only be derived for named structs",
            ));
        }
    };

    // Analyze fields
    let mut field_infos: Vec<FieldInfo> = Vec::new();
    let mut col_index: u16 = 0;
    for field in fields {
        let ident = field.ident.clone().unwrap();
        let ty = field.ty.clone();
        let config = parse_field_attrs(&field.attrs)?;
        let is_option = is_option_type(&ty);
        let current_col = if config.skip {
            0
        } else {
            let c = col_index;
            col_index += 1;
            c
        };
        field_infos.push(FieldInfo {
            ident,
            ty: ty.clone(),
            config,
            col_index: current_col,
            is_option: is_option.is_some(),
            inner_type: is_option,
        });
    }

    // Generate write_header body
    let header_writes: Vec<proc_macro2::TokenStream> = field_infos
        .iter()
        .filter(|f| !f.config.skip)
        .map(|f| {
            let col = f.col_index;
            let header = f
                .config
                .header
                .as_deref()
                .unwrap_or(&f.ident.to_string())
                .to_string();
            quote! { ws.write(row, #col, #header)?; }
        })
        .collect();

    // Generate write_row body
    let row_writes: Vec<proc_macro2::TokenStream> = field_infos
        .iter()
        .filter(|f| !f.config.skip)
        .map(|f| {
            let ident = &f.ident;
            let col = f.col_index;
            let is_string = if f.is_option {
                f.inner_type.as_ref().is_some_and(is_string_type)
            } else {
                is_string_type(&f.ty)
            };

            if let Some(ref fmt_str) = f.config.format {
                if f.is_option {
                    if is_string {
                        quote! {
                            if let Some(ref __v) = self.#ident {
                                let __fmt = zavora_xlsx::Format::new().num_format(#fmt_str);
                                ws.write_with_format(row, #col, __v.as_str(), &__fmt)?;
                            }
                        }
                    } else {
                        quote! {
                            if let Some(__v) = self.#ident {
                                let __fmt = zavora_xlsx::Format::new().num_format(#fmt_str);
                                ws.write_with_format(row, #col, __v, &__fmt)?;
                            }
                        }
                    }
                } else if is_string {
                    quote! {
                        let __fmt = zavora_xlsx::Format::new().num_format(#fmt_str);
                        ws.write_with_format(row, #col, self.#ident.as_str(), &__fmt)?;
                    }
                } else {
                    quote! {
                        let __fmt = zavora_xlsx::Format::new().num_format(#fmt_str);
                        ws.write_with_format(row, #col, self.#ident, &__fmt)?;
                    }
                }
            } else if f.is_option {
                if is_string {
                    quote! {
                        if let Some(ref __v) = self.#ident {
                            ws.write(row, #col, __v.as_str())?;
                        }
                    }
                } else {
                    quote! {
                        if let Some(__v) = self.#ident {
                            ws.write(row, #col, __v)?;
                        }
                    }
                }
            } else if is_string {
                quote! { ws.write(row, #col, self.#ident.as_str())?; }
            } else {
                quote! { ws.write(row, #col, self.#ident)?; }
            }
        })
        .collect();

    // Generate read_row body — build each field assignment
    let field_reads: Vec<proc_macro2::TokenStream> = field_infos
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if f.config.skip {
                return quote! { #ident: Default::default(), };
            }
            let header = f
                .config
                .header
                .as_deref()
                .unwrap_or(&f.ident.to_string())
                .to_string();
            let field_name_str = f.ident.to_string();

            if f.is_option {
                let inner = f.inner_type.as_ref().unwrap();
                let convert = cell_value_convert_expr(inner, &field_name_str);
                quote! {
                    #ident: {
                        match headers.iter().position(|h| h == #header) {
                            Some(__col) => {
                                let __cv = ws.read_cell(row, __col as u16);
                                match __cv {
                                    zavora_xlsx::CellValue::Empty => None,
                                    _ => Some(#convert),
                                }
                            }
                            None => None,
                        }
                    },
                }
            } else {
                let convert = cell_value_convert_expr(&f.ty, &field_name_str);
                quote! {
                    #ident: {
                        let __col = headers.iter().position(|h| h == #header)
                            .ok_or_else(|| zavora_xlsx::Error::InvalidData(
                                format!("missing column header '{}' for field '{}'", #header, #field_name_str)
                            ))?;
                        let __cv = ws.read_cell(row, __col as u16);
                        #convert
                    },
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl zavora_xlsx::ExcelRowWriter for #name {
            fn write_header(&self, ws: &mut zavora_xlsx::Worksheet, row: zavora_xlsx::RowNum) -> zavora_xlsx::Result<()> {
                #(#header_writes)*
                Ok(())
            }
            fn write_row(&self, ws: &mut zavora_xlsx::Worksheet, row: zavora_xlsx::RowNum) -> zavora_xlsx::Result<()> {
                #(#row_writes)*
                Ok(())
            }
        }

        impl zavora_xlsx::ExcelRowReader for #name {
            fn read_row(ws: &zavora_xlsx::Worksheet, row: zavora_xlsx::RowNum, headers: &[String]) -> zavora_xlsx::Result<Self> {
                Ok(Self {
                    #(#field_reads)*
                })
            }
        }
    };

    Ok(expanded)
}

// Generate the conversion expression from CellValue to a target type
fn cell_value_convert_expr(ty: &syn::Type, field_name: &str) -> proc_macro2::TokenStream {
    if is_string_type(ty) {
        quote! {
            match &__cv {
                zavora_xlsx::CellValue::String(s) => s.clone(),
                zavora_xlsx::CellValue::Number(n) => n.to_string(),
                zavora_xlsx::CellValue::Bool(b) => b.to_string(),
                zavora_xlsx::CellValue::Empty => String::new(),
                zavora_xlsx::CellValue::RichText(rt) => rt.plain_text(),
                other => return Err(zavora_xlsx::Error::InvalidData(
                    format!("field '{}': expected string, got {:?}", #field_name, other)
                )),
            }
        }
    } else if is_bool_type(ty) {
        quote! {
            match &__cv {
                zavora_xlsx::CellValue::Bool(b) => *b,
                other => return Err(zavora_xlsx::Error::InvalidData(
                    format!("field '{}': expected boolean, got {:?}", #field_name, other)
                )),
            }
        }
    } else {
        // Numeric type — extract from Number and cast
        quote! {
            match &__cv {
                zavora_xlsx::CellValue::Number(n) => *n as #ty,
                other => return Err(zavora_xlsx::Error::InvalidData(
                    format!("field '{}': expected numeric value, got {:?}", #field_name, other)
                )),
            }
        }
    }
}
