pub fn number<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    position: &str,
    filter: &str,
) -> Result<usize, String> {
    substring.next().ok_or_else(||format! ("invalid options for {filter} filter\nexpected {format}\n missing {position} [number]"))
                .and_then(|end| end.parse().map_err(|_| format! ("invalid options for {filter} filter\nexpected {format}\n cannot parse {position} [number]")))
}
pub fn extra<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    filter: &str,
) -> Result<(), String> {
    substring.next().map_or(Ok(()), |extra| Err(format!( "invalid options for {filter} filter\nexpected {format}\n, found extra stuff after {format}: {extra}")))
}
pub fn label<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    label: &str,
    filter: &str,
) -> Result<(), String> {
    substring.next().ok_or_else(||format! ("invalid options for {filter} filter\nexpected {format}\n missing label {label}"))
                .and_then(|l| {
                    if label == l {
                        Ok(())
                    } else {
                        Err(format!("invalid options for {filter} filter\n expected {format}\nexpected {label}, found {l}")) 
                    }
                }
        )
}

pub fn string<'a>(
    substring: &mut impl Iterator<Item = &'a str>,
    format: &str,
    position: &str,
    filter: &str,
) -> Result<String, String> {
    substring.next().map(ToOwned::to_owned).ok_or_else(||format! ("invalid options for function_with_parameter filter\nexpected {format}\n missing {position} [number]"))
}
