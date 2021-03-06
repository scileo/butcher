use std::iter::{self, FromIterator};

use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Binding, Constraint, DeriveInput,
    GenericArgument, GenericParam, Ident, LifetimeDef, ParenthesizedGenericArguments, Path,
    PathArguments, PathSegment, ReturnType, Token, Type, TypeParam, TypeParamBound, TypePath,
};

use proc_macro2::TokenStream;

pub(super) trait ReplaceSelf {
    fn replace(&mut self, rep: &Type);
}

macro_rules! impl_replace_self_struct {
    (
        $rep:ident,
        $(
            $ty:ident {
                $( $name:ident => $fun:expr ),* $(,)?
            } $(,)?
        ),*
    ) => {
        $(
            impl_replace_self_struct!(
                @single_type
                $rep,
                $ty {
                    $( $name => $fun ),*
                });
        )*
    };

    (
        @single_type
        $rep:ident,
        $ty:ident {
            $( $name:ident => $fun:expr ),* $(,)?
        } $(,)?
    ) => {
        impl ReplaceSelf for syn::$ty {
            #[allow(unused_variables)]
            fn replace(&mut self, $rep: &Type) {
                let syn::$ty { $( $name, )* .. } = self;
                $( $fun; )*
            }
        }
    }
}

impl ReplaceSelf for Type {
    fn replace(&mut self, rep: &Type) {
        match self {
            Type::Array(v) => v.replace(rep),
            Type::BareFn(bf) => bf.replace(rep),
            Type::Group(g) => g.replace(rep),
            Type::ImplTrait(it) => it.replace(rep),
            Type::Infer(i) => i.replace(rep),
            Type::Macro(m) => m.replace(rep),
            Type::Never(n) => n.replace(rep),
            Type::Paren(p) => p.replace(rep),
            Type::Path(TypePath { path, .. }) if path.is_ident("Self") => {
                *self = rep.clone();
            }
            Type::Path(p) => p.replace(rep),
            Type::Ptr(p) => p.replace(rep),
            Type::Reference(r) => r.replace(rep),
            Type::Slice(s) => s.replace(rep),
            Type::TraitObject(to) => to.replace(rep),
            Type::Tuple(t) => t.replace(rep),
            Type::Verbatim(v) => v.replace(rep),
            _ => unimplemented!(),
        }
    }
}

impl_replace_self_struct! {
    rep,

    TypeArray {
        elem => elem.as_mut().replace(rep),
    },

    TypeBareFn {
        inputs => inputs.iter_mut().for_each(|arg| arg.replace(rep)),
        output => output.replace(rep),
    },

    BareFnArg {
        ty => ty.replace(rep),
    },

    TypeGroup {
        elem => elem.replace(rep),
    },

    TypeImplTrait {
        bounds => bounds.iter_mut().for_each(|b| b.replace(rep)),
    },

    TraitBound {
        path => path.replace(rep),
    },

    Path {
        segments => segments.iter_mut().for_each(|s| s.replace(rep)),
    },

    PathSegment {
        arguments => arguments.replace(rep),
    },

    TypeSlice {
        elem => elem.replace(rep),
    },

    TypeParen {
        elem => elem.as_mut().replace(rep),
    },

    TypePath {
        qself => if let Some(qself) = qself { qself.replace(rep) },
        path => path.replace(rep),
    },

    QSelf {
        ty => ty.as_mut().replace(rep),
    },

    TypePtr {
        elem => elem.replace(rep),
    },

    TypeReference {
        elem => elem.replace(rep),
    },

    TypeTraitObject {
        bounds => bounds.iter_mut().for_each(|bound| bound.replace(rep)),
    },

    TypeTuple {
        elems => elems.iter_mut().for_each(|ty| ty.replace(rep)),
    },

    TypeInfer {},
    TypeMacro {},
    TypeNever {},
}

impl ReplaceSelf for ReturnType {
    fn replace(&mut self, rep: &Type) {
        if let ReturnType::Type(_, ty) = self {
            ty.replace(rep);
        }
    }
}

impl ReplaceSelf for TypeParamBound {
    fn replace(&mut self, rep: &Type) {
        if let TypeParamBound::Trait(tb) = self {
            tb.replace(rep);
        }
    }
}

impl ReplaceSelf for PathArguments {
    fn replace(&mut self, rep: &Type) {
        match self {
            PathArguments::None => {}
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                args.iter_mut().for_each(|arg| arg.replace(rep))
            }
            PathArguments::Parenthesized(ParenthesizedGenericArguments {
                inputs, output, ..
            }) => {
                inputs.iter_mut().for_each(|ty| ty.replace(rep));
                output.replace(rep);
            }
        }
    }
}

impl ReplaceSelf for GenericArgument {
    fn replace(&mut self, rep: &Type) {
        match self {
            GenericArgument::Lifetime(_) | GenericArgument::Const(_) => {}
            GenericArgument::Type(t) => t.replace(rep),
            GenericArgument::Binding(Binding { ty, .. }) => ty.replace(rep),
            GenericArgument::Constraint(Constraint { bounds, .. }) => {
                bounds.iter_mut().for_each(|b| b.replace(rep))
            }
        }
    }
}

impl ReplaceSelf for TokenStream {
    fn replace(&mut self, _rep: &Type) {}
}

pub(super) fn create_type_signature(input: &DeriveInput) -> Type {
    let name = input.ident.clone();
    let params = input.generics.params.iter().cloned();
    let lt_token = input.generics.lt_token;
    let gt_token = input.generics.gt_token;

    create_type_signature_from_raws(name, params, lt_token, gt_token)
}

fn create_type_signature_from_raws(
    ident: Ident,
    params: impl Iterator<Item = GenericParam>,
    lt_token: Option<Token![<]>,
    gt_token: Option<Token![>]>,
) -> Type {
    let segments = create_path_segments(ident, params, lt_token, gt_token);
    let path = Path {
        leading_colon: None,
        segments,
    };

    Type::Path(TypePath { qself: None, path })
}

fn create_path_segments(
    ident: Ident,
    params: impl Iterator<Item = GenericParam>,
    lt_token: Option<Token![<]>,
    gt_token: Option<Token![>]>,
) -> Punctuated<PathSegment, Token![::]> {
    let args = Punctuated::from_iter(arguments_from_params(params));

    let arguments = match (lt_token, gt_token) {
        (Some(lt_token), Some(gt_token)) => {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token,
                gt_token,
                args,
            })
        }
        (None, None) => PathArguments::None,
        _ => unreachable!(),
    };

    let segment = PathSegment { ident, arguments };

    Punctuated::from_iter(iter::once(segment))
}

fn arguments_from_params(
    input: impl Iterator<Item = GenericParam>,
) -> impl Iterator<Item = GenericArgument> {
    input.flat_map(generic_param)
}

fn generic_param(generic_param: GenericParam) -> Option<GenericArgument> {
    match generic_param {
        GenericParam::Type(TypeParam { ident, .. }) => {
            let segments = Punctuated::from_iter(iter::once(PathSegment {
                ident,
                arguments: PathArguments::None,
            }));

            let path = Path {
                leading_colon: None,
                segments,
            };

            Some(GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path,
            })))
        }

        GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
            Some(GenericArgument::Lifetime(lifetime))
        }

        GenericParam::Const(_) => None,
    }
}

#[cfg(test)]
macro_rules! test_replace_self {
    ($rep:ty, $left:ty, $right:ty) => {
        let mut left: Type = syn::parse_quote! { $left };
        let rep: Type = syn::parse_quote! { $rep };
        left.replace(&rep);
        let right: Type = syn::parse_quote! { $right };
        assert_eq_tt!(left, right);
    };
}

#[cfg(test)]
mod replace_self {
    use super::*;

    // Note: the tests here deliberately do not test everything. The goal here
    // is to ensure it works in the most simple situation.
    //
    // Anyway, most of the ReplaceSelf implementation is macro-generated, so
    // it should not contain much errors.

    #[test]
    fn simple() {
        test_replace_self! { Foo, Self, Foo };
        test_replace_self! { Bar<Foo>, Self, Bar<Foo> };
    }

    #[test]
    fn generic() {
        test_replace_self! { Foo, Vec<Self>, Vec<Foo> };
        test_replace_self! { Foo, Vec<Vec<Self>>, Vec<Vec<Foo>> };
    }

    #[test]
    fn impl_trait() {
        test_replace_self! { Foo, impl AsRef<Self>, impl AsRef<Foo> };
    }

    #[test]
    fn tuple() {
        test_replace_self! { Foo, (Self, usize), (Foo, usize) };
    }
}

#[cfg(test)]
macro_rules! test_create_type_signature {
    ($left:item, $right:path) => {
        let tmp: DeriveInput = syn::parse_quote! { $left };
        let left = create_type_signature(&tmp);
        let right: Type = syn::parse_quote! { $right };
        assert_eq_tt!(left, right);
    };
}

#[cfg(test)]
mod create_type_signature {
    use super::*;

    #[test]
    fn test() {
        test_create_type_signature!(struct Foo<A, B>;, Foo<A, B>);
        test_create_type_signature!(struct Foo<'a, A: 'a, B: ToOwned>;, Foo<'a, A, B>);
        test_create_type_signature!(struct Vec<T: Clone>;, Vec<T>);
    }
}

// Note: here it is needed to break the left-right conversion because the
// matching rule $($_:tt)+ eats the whole remaining input, including $right.
#[cfg(test)]
macro_rules! test_generic_param {
    (None, $($left:tt)+ $(,)?) => {
        let tmp: GenericParam = parse_quote! { $($left)+ };
        assert!(generic_param(tmp).is_none());
    };

    ($right:tt, $($left:tt)+ $(,)?) => {
        let input: GenericParam = syn::parse_quote! { $($left)+ };
        let left = generic_param(input);
        let right: Option<GenericArgument> = Some(syn::parse_quote! { $right });
        assert_eq_tt!(left, right);
    };
}

#[cfg(test)]
mod generic_param {
    use super::*;

    use syn::parse_quote;

    #[test]
    fn handles_type() {
        test_generic_param!(T, T);
        test_generic_param!(T, T: AsRef<str> + 'a);
    }

    #[test]
    fn handles_lifetimes() {
        test_generic_param!('a, 'a);
        test_generic_param!('a, 'a: 'b);
    }

    #[test]
    fn does_not_hangle_consts() {
        test_generic_param!(None, const LENGTH: usize);
    }
}
