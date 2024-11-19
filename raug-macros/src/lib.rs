use proc_macro::TokenStream;
use quote::quote;

struct SplitOutputs {
    output: syn::Expr,
    count: syn::LitInt,
}

impl syn::parse::Parse for SplitOutputs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let output = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let count = input.parse()?;
        Ok(Self { output, count })
    }
}

#[proc_macro]
pub fn split_outputs(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as SplitOutputs);

    let output = input.output;

    let count = input.count.base10_parse().unwrap();

    let mut idents = vec![];
    for i in 0..count {
        let ident = syn::Ident::new(&format!("out{}", i), proc_macro2::Span::call_site());
        idents.push(ident);
    }

    let start = quote! {
        let raug::processor::ProcessorOutputs {
            output_spec,
            outputs,
            mode,
        } = #output;

        let [#(#idents),*] = outputs else {
            panic!("Expected {} outputs, got {}", #count, outputs.len());
        };
    };

    let mut chunks = vec![];

    for (i, ident) in idents.iter().enumerate() {
        let chunk = quote! {
            raug::processor::ProcessorOutputs::new(
                std::slice::from_ref(&output_spec[#i]),
                std::slice::from_mut(#ident),
                mode,
            )
        };
        chunks.push(chunk);
    }

    let output = quote! {{
        #start

        (#(#chunks),*)
    }};

    output.into()
}

struct IterProcIoAs {
    inputs: syn::Ident,
    input_types: syn::punctuated::Punctuated<syn::Type, syn::Token![,]>,
    outputs: syn::Ident,
    output_types: syn::punctuated::Punctuated<syn::Type, syn::Token![,]>,
}

impl syn::parse::Parse for IterProcIoAs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inputs = input.parse()?;
        input.parse::<syn::Token![as]>()?;
        let input_types;
        syn::bracketed!(input_types in input);
        let input_types = input_types.parse_terminated(syn::Type::parse, syn::Token![,])?;
        input.parse::<syn::Token![,]>()?;
        let outputs = input.parse()?;
        input.parse::<syn::Token![as]>()?;
        let output_types;
        syn::bracketed!(output_types in input);
        let output_types = output_types.parse_terminated(syn::Type::parse, syn::Token![,])?;
        Ok(Self {
            inputs,
            outputs,
            input_types,
            output_types,
        })
    }
}

#[proc_macro]
pub fn iter_proc_io_as(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as IterProcIoAs);

    let inputs = input.inputs;
    let outputs = input.outputs;

    let input_count = input.input_types.len();
    let output_count = input.output_types.len();

    let mut input_idents = vec![];
    for i in 0..input_count {
        let ident = syn::Ident::new(&format!("in{}", i), proc_macro2::Span::call_site());
        input_idents.push(ident);
    }

    let mut output_idents = vec![];
    for i in 0..output_count {
        let ident = syn::Ident::new(&format!("out{}", i), proc_macro2::Span::call_site());
        output_idents.push(ident);
    }

    let start = quote! {
        let raug::processor::ProcessorInputs {
            input_specs,
            inputs,
            mode,
            sample_rate,
            block_size,
        } = #inputs;

        let [#(#input_idents),*] = inputs else {
            panic!("Expected {} inputs, got {}", #input_count, inputs.len());
        };

        let raug::processor::ProcessorOutputs {
            output_spec,
            outputs,
            mode,
        } = #outputs;

        let [#(#output_idents),*] = outputs else {
            panic!("Expected {} outputs, got {}", #output_count, outputs.len());
        };
    };

    let mut chunks = vec![];

    for (i, (input_ident, input_typ)) in input_idents
        .iter()
        .zip(input.input_types.iter())
        .enumerate()
    {
        if let syn::Type::Path(path) = input_typ {
            if path.path.get_ident().unwrap() == "Any" {
                let chunk = quote! {
                    raug::processor::ProcessorInputs::new(
                        std::slice::from_ref(&input_specs[#i]),
                        std::slice::from_ref(#input_ident),
                        mode,
                        sample_rate,
                        block_size,
                    ).iter_input(0)
                };
                chunks.push(chunk);
                continue;
            }
        }
        let chunk = quote! {
            raug::processor::ProcessorInputs::new(
                std::slice::from_ref(&input_specs[#i]),
                std::slice::from_ref(#input_ident),
                mode,
                sample_rate,
                block_size,
            ).iter_input_as::<#input_typ>(0)?
        };
        chunks.push(chunk);
    }

    for (i, (output_ident, output_typ)) in output_idents
        .iter()
        .zip(input.output_types.iter())
        .enumerate()
    {
        if let syn::Type::Path(path) = output_typ {
            if path.path.get_ident().unwrap() == "Any" {
                let chunk = quote! {
                    raug::processor::ProcessorOutputs::new(
                        std::slice::from_ref(&output_spec[#i]),
                        std::slice::from_mut(#output_ident),
                        mode,
                    ).iter_output_mut(0)
                };
                chunks.push(chunk);
                continue;
            }
        }
        let chunk = quote! {
            raug::processor::ProcessorOutputs::new(
                std::slice::from_ref(&output_spec[#i]),
                std::slice::from_mut(#output_ident),
                mode,
            ).iter_output_mut_as::<#output_typ>(0)?
        };
        chunks.push(chunk);
    }

    let output = quote! {{
        #start

        raug::__itertools::izip!(#(#chunks),*)
    }};

    output.into()
}
