//! Minimal Rust syntax parsing for proc macros
//! 
//! Provides basic AST types and parsing utilities as a lightweight alternative to `syn`.

use std::fmt;

/// An identifier in Rust code
#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub span: Span,
    pub sym: String,
}

impl Ident {
    pub fn new(s: &str, span: Span) -> Self {
        Self {
            span,
            sym: s.to_string(),
        }
    }
    
    pub fn new_static(s: &'static str) -> Self {
        Self {
            span: Span::call_site(),
            sym: s.to_string(),
        }
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sym)
    }
}

/// A span representing a source code location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub(crate) _private: (),
}

impl Span {
    pub fn call_site() -> Self {
        Self { _private: () }
    }
    
    pub fn mixed_site() -> Self {
        Self { _private: () }
    }
}

/// A Rust path like `std::vec::Vec`
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }
    
    pub fn single(ident: Ident) -> Self {
        Self {
            segments: vec![PathSegment {
                ident,
                arguments: PathArguments::None,
            }],
        }
    }
    
    pub fn is_ident(&self, s: &str) -> bool {
        self.segments.len() == 1 && self.segments[0].ident.sym == s
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

/// A segment of a path
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub ident: Ident,
    pub arguments: PathArguments,
}

/// Arguments to a path segment (e.g., `<T>` in `Vec<T>`)
#[derive(Debug, Clone, PartialEq)]
pub enum PathArguments {
    None,
    AngleBracketed(AngleBracketedArgs),
    Parenthesized(ParenthesizedArgs),
}

/// Generic arguments in angle brackets: `<T, U>`
#[derive(Debug, Clone, PartialEq)]
pub struct AngleBracketedArgs {
    pub args: Vec<GenericArgument>,
}

/// Generic arguments in parentheses: `(T, U) -> V`
#[derive(Debug, Clone, PartialEq)]
pub struct ParenthesizedArgs {
    pub inputs: Vec<Type>,
    pub output: Option<Type>,
}

/// A generic argument
#[derive(Debug, Clone, PartialEq)]
pub enum GenericArgument {
    Type(Type),
    Lifetime(Lifetime),
}

/// A lifetime like `'a`
#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub span: Span,
    pub sym: String,
}

impl Lifetime {
    pub fn new(s: &str, span: Span) -> Self {
        Self {
            span,
            sym: s.to_string(),
        }
    }
}

/// A Rust type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Path(TypePath),
    Tuple(TypeTuple),
    Reference(TypeReference),
    Slice(TypeSlice),
    Array(TypeArray),
    BareFn(Box<TypeBareFn>),
    Verbatim(TokenStream),
}

/// A type that is a path
#[derive(Debug, Clone, PartialEq)]
pub struct TypePath {
    pub qself: Option<QSelf>,
    pub path: Path,
}

/// A type in angle brackets with an optional type
#[derive(Debug, Clone, PartialEq)]
pub struct QSelf {
    pub ty: Box<Type>,
    pub position: usize,
}

/// A tuple type: `(T, U, V)`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeTuple {
    pub elems: Vec<Type>,
}

/// A reference type: `&T` or `&mut T`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeReference {
    pub and_token: Span,
    pub lifetime: Option<Lifetime>,
    pub mutability: Option<Span>,
    pub elem: Box<Type>,
}

/// A slice type: `[T]`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSlice {
    pub elem: Box<Type>,
}

/// An array type: `[T; N]`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeArray {
    pub elem: Box<Type>,
    pub len: Expr,
}

/// A function pointer type: `fn(T, U) -> V`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeBareFn {
    pub lifetimes: Option<Vec<Lifetime>>,
    pub inputs: Vec<BareFnArg>,
    pub output: Option<Box<Type>>,
}

/// An argument to a bare function type
#[derive(Debug, Clone, PartialEq)]
pub struct BareFnArg {
    pub name: Option<Ident>,
    pub ty: Type,
}

/// A Rust expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Path(ExprPath),
    Lit(Lit),
    Call(ExprCall),
    MethodCall(ExprMethodCall),
    Field(ExprField),
    Index(ExprIndex),
    Binary(ExprBinary),
    Unary(ExprUnary),
    Group(ExprGroup),
    Verbatim(TokenStream),
}

/// A path expression
#[derive(Debug, Clone, PartialEq)]
pub struct ExprPath {
    pub qself: Option<QSelf>,
    pub path: Path,
}

/// A literal expression
#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Str(LitStr),
    Int(LitInt),
    Float(LitFloat),
    Bool(LitBool),
    Char(LitChar),
    Byte(LitByte),
    ByteStr(LitByteStr),
    Verbatim(TokenStream),
}

/// A string literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitStr {
    pub span: Span,
    pub value: String,
}

/// An integer literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitInt {
    pub span: Span,
    pub value: String,
}

/// A float literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitFloat {
    pub span: Span,
    pub value: String,
}

/// A boolean literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitBool {
    pub span: Span,
    pub value: bool,
}

/// A character literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitChar {
    pub span: Span,
    pub value: char,
}

/// A byte literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitByte {
    pub span: Span,
    pub value: u8,
}

/// A byte string literal
#[derive(Debug, Clone, PartialEq)]
pub struct LitByteStr {
    pub span: Span,
    pub value: Vec<u8>,
}

/// A function call expression
#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
}

/// A method call expression
#[derive(Debug, Clone, PartialEq)]
pub struct ExprMethodCall {
    pub method: Ident,
    pub args: Vec<Expr>,
}

/// A field access expression
#[derive(Debug, Clone, PartialEq)]
pub struct ExprField {
    pub base: Box<Expr>,
    pub member: Ident,
}

/// An indexing expression
#[derive(Debug, Clone, PartialEq)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

/// A binary operation
#[derive(Debug, Clone, PartialEq)]
pub struct ExprBinary {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

/// A unary operation
#[derive(Debug, Clone, PartialEq)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Deref,
    Not,
    Neg,
}

/// A grouped expression
#[derive(Debug, Clone, PartialEq)]
pub struct ExprGroup {
    pub expr: Box<Expr>,
}

/// A Rust item (struct, enum, fn, etc.)
#[derive(Debug, Clone)]
pub enum Item {
    Struct(ItemStruct),
    Enum(ItemEnum),
    Fn(ItemFn),
    Const(ItemConst),
    Type(ItemType),
    Mod(ItemMod),
    Use(ItemUse),
    Verbatim(TokenStream),
}

/// A struct definition
#[derive(Debug, Clone)]
pub struct ItemStruct {
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Fields,
    pub semi_token: Option<Span>,
}

/// An enum definition
#[derive(Debug, Clone)]
pub struct ItemEnum {
    pub ident: Ident,
    pub generics: Generics,
    pub variants: Vec<Variant>,
}

/// A function definition
#[derive(Debug, Clone)]
pub struct ItemFn {
    pub ident: Ident,
    pub sig: Signature,
    pub block: Block,
}

/// A function signature
#[derive(Debug, Clone)]
pub struct Signature {
    pub constness: Option<Span>,
    pub asyncness: Option<Span>,
    pub unsafety: Option<Span>,
    pub abi: Option<Abi>,
    pub fn_token: Span,
    pub generics: Generics,
    pub inputs: Vec<FnArg>,
    pub output: ReturnType,
}

/// Function arguments
#[derive(Debug, Clone)]
pub enum FnArg {
    Receiver(Receiver),
    Typed(PatType),
}

/// A function receiver (`&self`, `&mut self`, `self`)
#[derive(Debug, Clone)]
pub struct Receiver {
    pub reference: Option<Span>,
    pub lifetime: Option<Lifetime>,
    pub mutability: Option<Span>,
    pub colon_token: Option<Span>,
    pub ty: Box<Type>,
}

/// A pattern with a type annotation
#[derive(Debug, Clone)]
pub struct PatType {
    pub pat: Box<Pat>,
    pub ty: Box<Type>,
}

/// A pattern
#[derive(Debug, Clone)]
pub enum Pat {
    Wild,
    Ident(PatIdent),
    Path(PatPath),
    Or(PatOr),
    Tuple(PatTuple),
}

/// An identifier pattern
#[derive(Debug, Clone)]
pub struct PatIdent {
    pub by_ref: Option<Span>,
    pub mutability: Option<Span>,
    pub ident: Ident,
    pub subpat: Option<(Span, Box<Pat>)>,
}

/// A path pattern
#[derive(Debug, Clone)]
pub struct PatPath {
    pub path: Path,
}

/// An or pattern (`A | B`)
#[derive(Debug, Clone)]
pub struct PatOr {
    pub cases: Vec<Pat>,
}

/// A tuple pattern
#[derive(Debug, Clone)]
pub struct PatTuple {
    pub elems: Vec<Pat>,
}

/// Return type of a function
#[derive(Debug, Clone)]
pub enum ReturnType {
    Default,
    Type(Span, Box<Type>),
}

/// A block of code
#[derive(Debug, Clone)]
pub struct Block {
    pub brace_token: Span,
    pub stmts: Vec<Stmt>,
}

/// A statement
#[derive(Debug, Clone)]
pub enum Stmt {
    Local(Local),
    Item(Item),
    Expr(Expr, Option<Span>),
    Semi(Expr, Span),
}

/// A local binding (`let x = 5`)
#[derive(Debug, Clone)]
pub struct Local {
    pub let_token: Span,
    pub pat: Pat,
    pub init: Option<(Span, Box<Expr>)>,
    pub semi_token: Span,
}

/// ABI specification
#[derive(Debug, Clone)]
pub struct Abi {
    pub extern_token: Span,
    pub name: Option<LitStr>,
}

/// Generic parameters
#[derive(Debug, Clone, Default)]
pub struct Generics {
    pub lt_token: Option<Span>,
    pub params: Vec<GenericParam>,
    pub gt_token: Option<Span>,
    pub where_clause: Option<WhereClause>,
}

/// A generic parameter
#[derive(Debug, Clone)]
pub enum GenericParam {
    Type(TypeParam),
    Lifetime(ParamLifetime),
    Const(ConstParam),
}

/// A type parameter
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub ident: Ident,
    pub colon_token: Option<Span>,
    pub bounds: Vec<TypeParamBound>,
    pub eq_token: Option<Span>,
    pub default: Option<Type>,
}

/// A lifetime parameter
#[derive(Debug, Clone)]
pub struct ParamLifetime {
    pub lifetime: Lifetime,
    pub colon_token: Option<Span>,
    pub bounds: Vec<Lifetime>,
}

/// A const parameter
#[derive(Debug, Clone)]
pub struct ConstParam {
    pub const_token: Span,
    pub ident: Ident,
    pub colon_token: Span,
    pub ty: Type,
    pub eq_token: Option<Span>,
    pub default: Option<Expr>,
}

/// A bound on a type parameter
#[derive(Debug, Clone)]
pub enum TypeParamBound {
    Trait(TraitBound),
    Lifetime(Lifetime),
}

/// A trait bound
#[derive(Debug, Clone)]
pub struct TraitBound {
    pub paren_token: Option<Span>,
    pub modifier: TraitBoundModifier,
    pub lifetimes: Option<Vec<Lifetime>>,
    pub path: Path,
}

/// A trait bound modifier
#[derive(Debug, Clone)]
pub enum TraitBoundModifier {
    None,
    Maybe(Span),
}

/// A where clause
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub where_token: Span,
    pub predicates: Vec<WherePredicate>,
}

/// A where predicate
#[derive(Debug, Clone)]
pub enum WherePredicate {
    Lifetime(PredicateLifetime),
    Type(PredicateType),
}

/// A lifetime predicate
#[derive(Debug, Clone)]
pub struct PredicateLifetime {
    pub lifetime: Lifetime,
    pub colon_token: Span,
    pub bounds: Vec<Lifetime>,
}

/// A type predicate
#[derive(Debug, Clone)]
pub struct PredicateType {
    pub lifetimes: Option<Vec<Lifetime>>,
    pub bounded_ty: Type,
    pub colon_token: Span,
    pub bounds: Vec<TypeParamBound>,
}

/// Fields of a struct
#[derive(Debug, Clone)]
pub enum Fields {
    Named(FieldsNamed),
    Unnamed(FieldsUnnamed),
    Unit,
}

/// Named fields (`struct Foo { x: i32, y: i32 }`)
#[derive(Debug, Clone)]
pub struct FieldsNamed {
    pub brace_token: Span,
    pub named: Vec<Field>,
}

/// Unnamed fields (`struct Foo(i32, String)`)
#[derive(Debug, Clone)]
pub struct FieldsUnnamed {
    pub paren_token: Span,
    pub unnamed: Vec<Field>,
}

/// A struct field or enum variant
#[derive(Debug, Clone)]
pub struct Field {
    pub vis: Visibility,
    pub ident: Option<Ident>,
    pub colon_token: Option<Span>,
    pub ty: Type,
}

/// Visibility modifier
#[derive(Debug, Clone)]
pub enum Visibility {
    Public(Span),
    Inherited,
    Restricted(PubRestriction),
}

/// Restricted visibility
#[derive(Debug, Clone)]
pub struct PubRestriction {
    pub paren_token: Span,
    pub path: Option<Path>,
}

/// An enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    pub ident: Ident,
    pub fields: Fields,
    pub discriminant: Option<(Span, Expr)>,
}

/// A use declaration
#[derive(Debug, Clone)]
pub struct ItemUse {
    pub vis: Visibility,
    pub use_token: Span,
    pub tree: UseTree,
    pub semi_token: Span,
}

/// A use tree
#[derive(Debug, Clone)]
pub enum UseTree {
    Path(UsePath),
    Group(UseGroup),
    Name(UseName),
    Rename(UseRename),
    Glob(UseGlob),
}

/// A use path
#[derive(Debug, Clone)]
pub struct UsePath {
    pub ident: Ident,
    pub colon2_token: Span,
    pub tree: Box<UseTree>,
}

/// A use group
#[derive(Debug, Clone)]
pub struct UseGroup {
    pub brace_token: Span,
    pub items: Vec<UseTree>,
}

/// A use name
#[derive(Debug, Clone)]
pub struct UseName {
    pub ident: Ident,
}

/// A renamed use
#[derive(Debug, Clone)]
pub struct UseRename {
    pub ident: Ident,
    pub as_token: Span,
    pub rename: Ident,
}

/// A glob import
#[derive(Debug, Clone)]
pub struct UseGlob {
    pub star_token: Span,
}

/// A module
#[derive(Debug, Clone)]
pub struct ItemMod {
    pub vis: Visibility,
    pub mod_token: Span,
    pub ident: Ident,
    pub content: Option<(Span, Vec<Item>, Span)>,
    pub semi_token: Option<Span>,
}

/// A type alias
#[derive(Debug, Clone)]
pub struct ItemType {
    pub vis: Visibility,
    pub type_token: Span,
    pub ident: Ident,
    pub generics: Generics,
    pub eq_token: Span,
    pub ty: Type,
    pub semi_token: Span,
}

/// A const item
#[derive(Debug, Clone)]
pub struct ItemConst {
    pub vis: Visibility,
    pub const_token: Span,
    pub ident: Ident,
    pub colon_token: Span,
    pub ty: Type,
    pub eq_token: Span,
    pub expr: Expr,
    pub semi_token: Span,
}

/// Token stream wrapper
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TokenStream {
    pub tokens: Vec<TokenTree>,
}

impl TokenStream {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }
    
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    
    pub fn iter(&self) -> std::slice::Iter<TokenTree> {
        self.tokens.iter()
    }
}

impl From<Vec<TokenTree>> for TokenStream {
    fn from(tokens: Vec<TokenTree>) -> Self {
        Self { tokens }
    }
}

/// A single token tree
#[derive(Debug, Clone, PartialEq)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

/// A grouped token sequence
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub delimiter: Delimiter,
    pub stream: TokenStream,
}

/// Delimiter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    None,
    Parenthesis,
    Brace,
    Bracket,
}

/// A punctuation token
#[derive(Debug, Clone, PartialEq)]
pub struct Punct {
    pub ch: char,
    pub spacing: Spacing,
}

impl Punct {
    pub fn new(ch: char, spacing: Spacing) -> Self {
        Self { ch, spacing }
    }
}

/// Punctuation spacing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    Alone,
    Joint,
}

/// A literal token
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub span: Span,
    pub symbol: String,
}

impl From<Ident> for TokenTree {
    fn from(ident: Ident) -> Self {
        TokenTree::Ident(ident)
    }
}

impl From<Punct> for TokenTree {
    fn from(punct: Punct) -> Self {
        TokenTree::Punct(punct)
    }
}

impl From<Literal> for TokenTree {
    fn from(lit: Literal) -> Self {
        TokenTree::Literal(lit)
    }
}

impl From<Group> for TokenTree {
    fn from(group: Group) -> Self {
        TokenTree::Group(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ident tests
    #[test]
    fn test_ident_new() {
        let span = Span::call_site();
        let ident = Ident::new("foo", span);
        assert_eq!(ident.sym, "foo");
    }

    #[test]
    fn test_ident_display() {
        let span = Span::call_site();
        let ident = Ident::new("my_type", span);
        assert_eq!(format!("{}", ident), "my_type");
    }

    // Path tests
    #[test]
    fn test_path_single() {
        let ident = Ident::new_static("Vec");
        let path = Path::single(ident);
        assert!(path.is_ident("Vec"));
        assert_eq!(path.segments.len(), 1);
    }

    #[test]
    fn test_path_is_ident() {
        let path = Path::new();
        assert!(!path.is_ident("foo"));

        let ident = Ident::new_static("std");
        let path = Path::single(ident);
        assert!(path.is_ident("std"));
        assert!(!path.is_ident("Vec"));
    }

    // Type tests
    #[test]
    fn test_type_path() {
        let ident = Ident::new_static("String");
        let path = Path::single(ident);
        let ty = Type::Path(TypePath {
            qself: None,
            path,
        });
        match ty {
            Type::Path(TypePath { path, .. }) => {
                assert!(path.is_ident("String"));
            }
            _ => panic!("Expected Type::Path"),
        }
    }

    #[test]
    fn test_type_tuple() {
        let ty = Type::Tuple(TypeTuple {
            elems: vec![
                Type::Path(TypePath {
                    qself: None,
                    path: Path::single(Ident::new_static("i32")),
                }),
                Type::Path(TypePath {
                    qself: None,
                    path: Path::single(Ident::new_static("String")),
                }),
            ],
        });
        match ty {
            Type::Tuple(TypeTuple { elems }) => {
                assert_eq!(elems.len(), 2);
            }
            _ => panic!("Expected Type::Tuple"),
        }
    }

    #[test]
    fn test_type_reference() {
        let ty = Type::Reference(TypeReference {
            and_token: Span::call_site(),
            lifetime: None,
            mutability: Some(Span::call_site()),
            elem: Box::new(Type::Path(TypePath {
                qself: None,
                path: Path::single(Ident::new_static("str")),
            })),
        });
        match ty {
            Type::Reference(TypeReference { mutability, elem, .. }) => {
                assert!(mutability.is_some());
                match *elem {
                    Type::Path(TypePath { path, .. }) => {
                        assert!(path.is_ident("str"));
                    }
                    _ => panic!("Expected inner type to be Path"),
                }
            }
            _ => panic!("Expected Type::Reference"),
        }
    }

    // Expr tests
    #[test]
    fn test_expr_path() {
        let expr = Expr::Path(ExprPath {
            qself: None,
            path: Path::single(Ident::new_static("println")),
        });
        match expr {
            Expr::Path(ExprPath { path, .. }) => {
                assert!(path.is_ident("println"));
            }
            _ => panic!("Expected Expr::Path"),
        }
    }

    #[test]
    fn test_expr_lit_string() {
        let expr = Expr::Lit(Lit::Str(LitStr {
            span: Span::call_site(),
            value: "hello".to_string(),
        }));
        match expr {
            Expr::Lit(Lit::Str(LitStr { value, .. })) => {
                assert_eq!(value, "hello");
            }
            _ => panic!("Expected Expr::Lit(Str)"),
        }
    }

    #[test]
    fn test_expr_lit_int() {
        let expr = Expr::Lit(Lit::Int(LitInt {
            span: Span::call_site(),
            value: "42".to_string(),
        }));
        match expr {
            Expr::Lit(Lit::Int(LitInt { value, .. })) => {
                assert_eq!(value, "42");
            }
            _ => panic!("Expected Expr::Lit(Int)"),
        }
    }

    #[test]
    fn test_expr_call() {
        let expr = Expr::Call(ExprCall {
            func: Box::new(Expr::Path(ExprPath {
                qself: None,
                path: Path::single(Ident::new_static("foo")),
            })),
            args: vec![
                Expr::Lit(Lit::Int(LitInt {
                    span: Span::call_site(),
                    value: "1".to_string(),
                })),
            ],
        });
        match expr {
            Expr::Call(ExprCall { func, args }) => {
                assert_eq!(args.len(), 1);
                match *func {
                    Expr::Path(ExprPath { path, .. }) => {
                        assert!(path.is_ident("foo"));
                    }
                    _ => panic!("Expected func to be Path"),
                }
            }
            _ => panic!("Expected Expr::Call"),
        }
    }

    // Item tests
    #[test]
    fn test_item_struct_named_fields() {
        let item = Item::Struct(ItemStruct {
            ident: Ident::new_static("Point"),
            generics: Generics::default(),
            fields: Fields::Named(FieldsNamed {
                brace_token: Span::call_site(),
                named: vec![
                    Field {
                        vis: Visibility::Inherited,
                        ident: Some(Ident::new_static("x")),
                        colon_token: Some(Span::call_site()),
                        ty: Type::Path(TypePath {
                            qself: None,
                            path: Path::single(Ident::new_static("i32")),
                        }),
                    },
                    Field {
                        vis: Visibility::Inherited,
                        ident: Some(Ident::new_static("y")),
                        colon_token: Some(Span::call_site()),
                        ty: Type::Path(TypePath {
                            qself: None,
                            path: Path::single(Ident::new_static("i32")),
                        }),
                    },
                ],
            }),
            semi_token: None,
        });

        match item {
            Item::Struct(ItemStruct { ident, fields, .. }) => {
                assert_eq!(ident.sym, "Point");
                match fields {
                    Fields::Named(FieldsNamed { named, .. }) => {
                        assert_eq!(named.len(), 2);
                        assert_eq!(named[0].ident.as_ref().unwrap().sym, "x");
                        assert_eq!(named[1].ident.as_ref().unwrap().sym, "y");
                    }
                    _ => panic!("Expected named fields"),
                }
            }
            _ => panic!("Expected Item::Struct"),
        }
    }

    #[test]
    fn test_item_enum() {
        let item = Item::Enum(ItemEnum {
            ident: Ident::new_static("Result"),
            generics: Generics::default(),
            variants: vec![
                Variant {
                    ident: Ident::new_static("Ok"),
                    fields: Fields::Unit,
                    discriminant: None,
                },
                Variant {
                    ident: Ident::new_static("Err"),
                    fields: Fields::Unit,
                    discriminant: None,
                },
            ],
        });

        match item {
            Item::Enum(ItemEnum { ident, variants, .. }) => {
                assert_eq!(ident.sym, "Result");
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].ident.sym, "Ok");
                assert_eq!(variants[1].ident.sym, "Err");
            }
            _ => panic!("Expected Item::Enum"),
        }
    }

    #[test]
    fn test_item_fn() {
        let item = Item::Fn(ItemFn {
            ident: Ident::new_static("main"),
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Span::call_site(),
                generics: Generics::default(),
                inputs: vec![],
                output: ReturnType::Default,
            },
            block: Block {
                brace_token: Span::call_site(),
                stmts: vec![],
            },
        });

        match item {
            Item::Fn(ItemFn { ident, sig, .. }) => {
                assert_eq!(ident.sym, "main");
                assert!(sig.inputs.is_empty());
                assert!(matches!(sig.output, ReturnType::Default));
            }
            _ => panic!("Expected Item::Fn"),
        }
    }

    // Generics tests
    #[test]
    fn test_generics_with_type_param() {
        let generics = Generics {
            lt_token: Some(Span::call_site()),
            params: vec![GenericParam::Type(TypeParam {
                ident: Ident::new_static("T"),
                colon_token: None,
                bounds: vec![],
                eq_token: None,
                default: None,
            })],
            gt_token: Some(Span::call_site()),
            where_clause: None,
        };

        assert_eq!(generics.params.len(), 1);
        match &generics.params[0] {
            GenericParam::Type(TypeParam { ident, .. }) => {
                assert_eq!(ident.sym, "T");
            }
            _ => panic!("Expected Type param"),
        }
    }

    // TokenStream tests
    #[test]
    fn test_tokenstream_new() {
        let ts = TokenStream::new();
        assert!(ts.is_empty());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn test_tokenstream_from_vec() {
        let tokens = vec![TokenTree::Ident(Ident::new_static("foo"))];
        let ts: TokenStream = tokens.into();
        assert_eq!(ts.len(), 1);
    }

    #[test]
    fn test_tokenstream_iter() {
        let tokens = vec![
            TokenTree::Ident(Ident::new_static("fn")),
            TokenTree::Ident(Ident::new_static("main")),
        ];
        let ts: TokenStream = tokens.into();
        let count = ts.iter().count();
        assert_eq!(count, 2);
    }

    // TokenTree tests
    #[test]
    fn test_token_tree_from_ident() {
        let ident = Ident::new_static("x");
        let tt: TokenTree = ident.clone().into();
        match tt {
            TokenTree::Ident(i) => assert_eq!(i.sym, "x"),
            _ => panic!("Expected Ident"),
        }
    }

    #[test]
    fn test_token_tree_from_punct() {
        let punct = Punct::new('=', Spacing::Alone);
        let tt: TokenTree = punct.into();
        match tt {
            TokenTree::Punct(p) => assert_eq!(p.ch, '='),
            _ => panic!("Expected Punct"),
        }
    }

    #[test]
    fn test_token_tree_from_literal() {
        let lit = Literal {
            span: Span::call_site(),
            symbol: "42".to_string(),
        };
        let tt: TokenTree = lit.into();
        match tt {
            TokenTree::Literal(l) => assert_eq!(l.symbol, "42"),
            _ => panic!("Expected Literal"),
        }
    }

    // Lifetime tests
    #[test]
    fn test_lifetime() {
        let span = Span::call_site();
        let lt = Lifetime::new("'a", span);
        assert_eq!(lt.sym, "'a");
    }

    // Visibility tests
    #[test]
    fn test_visibility_inherited() {
        let vis = Visibility::Inherited;
        match vis {
            Visibility::Inherited => {}
            _ => panic!("Expected Inherited"),
        }
    }

    #[test]
    fn test_visibility_public() {
        let vis = Visibility::Public(Span::call_site());
        match vis {
            Visibility::Public(_) => {}
            _ => panic!("Expected Public"),
        }
    }

    // Fields tests
    #[test]
    fn test_fields_unit() {
        let fields = Fields::Unit;
        match fields {
            Fields::Unit => {}
            _ => panic!("Expected Unit"),
        }
    }

    // Pat tests
    #[test]
    fn test_pat_wildcard() {
        let pat = Pat::Wild;
        match pat {
            Pat::Wild => {}
            _ => panic!("Expected Wild"),
        }
    }

    #[test]
    fn test_pat_ident() {
        let pat = Pat::Ident(PatIdent {
            by_ref: None,
            mutability: None,
            ident: Ident::new_static("x"),
            subpat: None,
        });
        match pat {
            Pat::Ident(PatIdent { ident, .. }) => {
                assert_eq!(ident.sym, "x");
            }
            _ => panic!("Expected Ident"),
        }
    }

    // BinOp tests
    #[test]
    fn test_binop_add() {
        let op = BinOp::Add;
        match op {
            BinOp::Add => {}
            _ => panic!("Expected Add"),
        }
    }

    #[test]
    fn test_binop_eq() {
        let op = BinOp::Eq;
        match op {
            BinOp::Eq => {}
            _ => panic!("Expected Eq"),
        }
    }

    // UnOp tests
    #[test]
    fn test_unop_not() {
        let op = UnOp::Not;
        match op {
            UnOp::Not => {}
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_unop_neg() {
        let op = UnOp::Neg;
        match op {
            UnOp::Neg => {}
            _ => panic!("Expected Neg"),
        }
    }

    // Delimiter tests
    #[test]
    fn test_delimiter_parenthesis() {
        let d = Delimiter::Parenthesis;
        match d {
            Delimiter::Parenthesis => {}
            _ => panic!("Expected Parenthesis"),
        }
    }

    #[test]
    fn test_delimiter_brace() {
        let d = Delimiter::Brace;
        match d {
            Delimiter::Brace => {}
            _ => panic!("Expected Brace"),
        }
    }

    // Spacing tests
    #[test]
    fn test_spacing_alone() {
        let s = Spacing::Alone;
        match s {
            Spacing::Alone => {}
            _ => panic!("Expected Alone"),
        }
    }

    #[test]
    fn test_spacing_joint() {
        let s = Spacing::Joint;
        match s {
            Spacing::Joint => {}
            _ => panic!("Expected Joint"),
        }
    }
}
