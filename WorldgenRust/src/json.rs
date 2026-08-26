// json.rs — 手写 JSON parser（沙箱无网络无 serde）。解析 worldgen JSON 到 JsonValue 树。
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(o) = self { o.iter().find(|(k, _)| k == key).map(|(_, v)| v) } else { None }
    }
    pub fn as_str(&self) -> Option<&str> { if let JsonValue::String(s) = self { Some(s.as_str()) } else { None } }
    pub fn as_f64(&self) -> Option<f64> { if let JsonValue::Number(n) = self { Some(*n) } else { None } }
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> { if let JsonValue::Array(a) = self { Some(a) } else { None } }
    pub fn as_object(&self) -> Option<&Vec<(String, JsonValue)>> { if let JsonValue::Object(o) = self { Some(o) } else { None } }
}

struct P<'a> { s: &'a [u8], i: usize }
fn skip_ws(p: &mut P) { while p.i < p.s.len() && (p.s[p.i] == b' ' || p.s[p.i] == b'\t' || p.s[p.i] == b'\n' || p.s[p.i] == b'\r') { p.i += 1; } }
fn parse_value(p: &mut P) -> Result<JsonValue, String> {
    skip_ws(p);
    if p.i >= p.s.len() { return Err("eof".into()); }
    match p.s[p.i] {
        b'{' => parse_object(p),
        b'[' => parse_array(p),
        b'"' => Ok(JsonValue::String(parse_string(p)?)),
        b't' => { if p.s[p.i..].starts_with(b"true") { p.i += 4; Ok(JsonValue::Bool(true)) } else { Err("bool".into()) } }
        b'f' => { if p.s[p.i..].starts_with(b"false") { p.i += 5; Ok(JsonValue::Bool(false)) } else { Err("bool".into()) } }
        b'n' => { if p.s[p.i..].starts_with(b"null") { p.i += 4; Ok(JsonValue::Null) } else { Err("null".into()) } }
        c if c == b'-' || c.is_ascii_digit() => parse_number(p),
        _ => Err(format!("unexpected {}", p.s[p.i] as char)),
    }
}
fn parse_object(p: &mut P) -> Result<JsonValue, String> {
    p.i += 1; // {
    let mut out = Vec::new();
    loop {
        skip_ws(p);
        if p.i < p.s.len() && p.s[p.i] == b'}' { p.i += 1; break; }
        let key = parse_string(p)?;
        skip_ws(p); if p.s[p.i] != b':' { return Err(":".into()); } p.i += 1;
        let val = parse_value(p)?;
        out.push((key, val));
        skip_ws(p);
        if p.s[p.i] == b',' { p.i += 1; } else if p.s[p.i] == b'}' { p.i += 1; break; } else { return Err(",".into()); }
    }
    Ok(JsonValue::Object(out))
}
fn parse_array(p: &mut P) -> Result<JsonValue, String> {
    p.i += 1; // [
    let mut out = Vec::new();
    loop {
        skip_ws(p);
        if p.i < p.s.len() && p.s[p.i] == b']' { p.i += 1; break; }
        if p.s[p.i] == b']' { p.i += 1; break; }
        out.push(parse_value(p)?);
        skip_ws(p);
        if p.s[p.i] == b',' { p.i += 1; } else if p.s[p.i] == b']' { p.i += 1; break; } else { return Err(",".into()); }
    }
    Ok(JsonValue::Array(out))
}
fn parse_string(p: &mut P) -> Result<String, String> {
    if p.s[p.i] != b'"' { return Err("quote".into()); }
    p.i += 1;
    let mut out = String::new();
    while p.i < p.s.len() && p.s[p.i] != b'"' {
        let c = p.s[p.i];
        if c == b'\\' {
            p.i += 1;
            let e = p.s[p.i];
            out.push(match e { b'n' => '\n', b't' => '\t', b'r' => '\r', b'\\' => '\\', b'"' => '"', b'/' => '/', _ => e as char });
            p.i += 1;
        } else {
            out.push(c as char);
            p.i += 1;
        }
    }
    p.i += 1; // "
    Ok(out)
}
fn parse_number(p: &mut P) -> Result<JsonValue, String> {
    let start = p.i;
    let start_neg = if p.s[p.i] == b'-' { p.i += 1; true } else { false };
    let _ = start_neg;
    while p.i < p.s.len() && (p.s[p.i].is_ascii_digit() || p.s[p.i] == b'.' || p.s[p.i] == b'e' || p.s[p.i] == b'E' || p.s[p.i] == b'+' || p.s[p.i] == b'-') { p.i += 1; }
    let txt = std::str::from_utf8(&p.s[start..p.i]).map_err(|e| e.to_string())?;
    let n: f64 = txt.parse::<f64>().map_err(|e| e.to_string())?;
    Ok(JsonValue::Number(n))
}

pub fn parse(text: &str) -> Result<JsonValue, String> {
    let mut p = P { s: text.as_bytes(), i: 0 };
    let v = parse_value(&mut p)?;
    Ok(v)
}
