use cl_format::*;

#[derive(Debug)]
struct MyStruct {
    a: usize,
    b: String,
}

impl TildeAble for MyStruct {
    fn format_va(&self, _tkind: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push_str(&format!("a: {}, b: {}", self.a, self.b));
        Ok(())
    }

    fn format_digit(&self, _tkind: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push_str(&format!("{}", self.a));
        Ok(())
    }
}

#[test]
fn test_custom_struct() {
    let s = MyStruct {
        a: 1,
        b: "b".to_string(),
    };

    assert_eq!("a: 1, b: b".to_string(), cl_format!("~a", &s).unwrap());
    assert_eq!(
        "a: 1, b: b lalalal a: 1, b: b".to_string(),
        cl_format!("~a lalalal ~a", &s, &s).unwrap()
    );

    assert_eq!("1".to_string(), cl_format!("~d", &s).unwrap());

    assert_eq!(
        "First: a: 1, b: b; Second: 1".to_string(),
        cl_format!("First: ~a; Second: ~d", &s, &s).unwrap()
    );
}
