//! Deliberately tiny valid Java 8 classfiles, independent of a compiler installation.
fn u2(out: &mut Vec<u8>, n: u16) {
    out.extend(n.to_be_bytes());
}
fn u4(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_be_bytes());
}
fn utf(out: &mut Vec<u8>, s: &str) {
    out.push(1);
    u2(out, s.len().try_into().unwrap());
    out.extend(s.as_bytes());
}
fn class(out: &mut Vec<u8>, utf: u16) {
    out.push(7);
    u2(out, utf);
}
/// public static Target run(Source): aload_0; checkcast Target; areturn.
pub fn cast_bci(wide: bool) -> u32 {
    if wide { 4 } else { 1 }
}
pub fn subject(name: &str, source: &str, target: &str, wide: bool) -> Vec<u8> {
    let mut b = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
    u2(&mut b, 10);
    utf(&mut b, name);
    class(&mut b, 1);
    utf(&mut b, "java/lang/Object");
    class(&mut b, 3);
    utf(&mut b, "run");
    utf(&mut b, &format!("(L{source};)L{target};"));
    utf(&mut b, "Code");
    utf(&mut b, target);
    class(&mut b, 8);
    for n in [0x0021, 2, 4, 0, 0, 1, 0x0009, 5, 6, 1, 7] {
        u2(&mut b, n);
    }
    let mut code = if wide {
        vec![0xc4, 0x19, 0, 0]
    } else {
        vec![0x2a]
    };
    assert_eq!(code.len() as u32, cast_bci(wide));
    code.extend([0xc0, 0, 9, 0xb0]);
    u4(&mut b, 12 + code.len() as u32);
    u2(&mut b, 1);
    u2(&mut b, 1);
    u4(&mut b, code.len() as u32);
    b.extend(code);
    u2(&mut b, 0);
    u2(&mut b, 0);
    u2(&mut b, 0);
    b
}
pub fn interface(name: &str, parent: Option<&str>, annotation: bool) -> Vec<u8> {
    let mut b = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
    u2(&mut b, if parent.is_some() { 7 } else { 5 });
    utf(&mut b, name);
    class(&mut b, 1);
    utf(&mut b, "java/lang/Object");
    class(&mut b, 3);
    if let Some(p) = parent {
        utf(&mut b, p);
        class(&mut b, 5);
    }
    u2(&mut b, if annotation { 0x2601 } else { 0x0601 });
    u2(&mut b, 2);
    u2(&mut b, 4);
    u2(&mut b, if parent.is_some() { 1 } else { 0 });
    if parent.is_some() {
        u2(&mut b, 6);
    }
    for _ in 0..3 {
        u2(&mut b, 0);
    }
    b
}
