/// Quick Define Errors
/// # Examples
///
/// ```rust
///
/// use common::err_kinds;
///
/// err_kinds! {
///     enum ErrorKind {
///         (InvalidParams(a: String, b: String), "invalid params: {}, expect {}")
///         (InvalidMethod(a: String), "invalid method: {}")
///     }
/// }
///
/// struct Error(ErrorKind);
///
/// type Result<T> = std::result::Result<T, Error>;
///
/// #[test]
/// fn test() {
///     println!("{}", ErrorKind::InvalidMethod("GET".to_string()).to_string()); // "invalid method: GET"
/// }
///
/// ```
#[macro_export]
macro_rules! err_kinds {
    (
        $(enum $enum_name:ident$(<$($lifetime:lifetime),+>)* {
            $(($konst:ident$(($($typ_name:ident:$typ:ty),+))*, $msg_pattern:literal))+
        })+
    ) => {
        $(
            pub enum $enum_name$(<$($lifetime),+>)* {
                $(
                    $konst$(($($typ),+))*
                ),+
            }

            impl$(<$($lifetime),+>)* ToString for $enum_name$(<$($lifetime),+>)* {
                fn to_string(&self) -> String {
                    match self {
                        $(
                            $enum_name::$konst$(($($typ_name),+))* => {
                                format!($msg_pattern, $($($typ_name),+)*)
                            }
                        )+
                    }
                }
            }
        )+
    };
}