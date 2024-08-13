pub(crate) fn number<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    position: &str,
) -> Result<usize, String> {
    substring.next().ok_or_else(||format! ("invalid options for function_in_lines filter\nexpected {format}\n missing {position} [number]"))
                .and_then(|end| end.parse().map_err(|_| format! ("invalid options for function_in_lines filter\nexpected {format}\n cannot parse {position} [number]")))
}
pub(crate) fn extra<'a>(substring: &mut impl Iterator<Item = &'a str>, format: &str) -> Result<(), String> {
    substring.next().map_or(Ok(()), |extra| Err(format!( "invalid options for function_in_lines filter\nexpected {format}\n, found extra stuff after {format}: {extra}")))
}
pub(crate) fn label<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    label: &str,
) -> Result<(), String> {
    substring.next().ok_or_else(||format! ("invalid options for function_in_lines filter\nexpected {format}\n missing label {label}"))
                .and_then(|l| {
                    if label == l {
                        Ok(())
                    } else {
                        Err(format!("invalid options for function_in_lines filter\n expected {format}\nexpected {label}, found {l}")) 
                    }
                }
        )
}
