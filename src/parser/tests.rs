use super::*;

#[test]
fn display_error_unexpected_t() {
    let expected = "The parser encountered %break when it was expecting one of [%uint, %nint]";
    let error = Error::UnexpectedT(vec![Kind::Uint, Kind::Nint], Term::Break);
    let actual = format!("{error}");
    assert_eq!(expected, actual)
}

#[test]
fn display_error_unexpected_nt() {
    let expected = "The parser encountered <VALUE> when it was expecting one of [<BSTR>, <TSTR>]";
    let error = Error::UnexpectedNT(vec![NonTerm::Bstr, NonTerm::Tstr], NonTerm::Value);
    let actual = format!("{error}");
    assert_eq!(expected, actual)
}
