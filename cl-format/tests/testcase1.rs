use cl_format::*;

#[derive(Debug)]
struct MyStruct {
    a: usize,
    b: String,
}

impl TildeAble for MyStruct {
    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        Some(self)
    }

    // fn into_tildekind_char(&self) -> Option<&dyn TildeKindChar> {
    //     None
    // }

    // fn into_tildekind_float(&self) -> Option<&dyn TildeKindFloat> {
    //     None
    // }

    fn into_tildekind_digit(&self) -> Option<&dyn TildeKindDigit> {
        Some(self)
    }

    // fn into_tildekind_star(&self) -> Option<&dyn TildeKindStar> {
    //     None
    // }

    // fn into_tildekind_standard(&self) -> Option<&dyn TildeKindStandard> {
    //     None
    // }

    // fn into_tildekind_loop(&self) -> Option<&dyn TildeKindLoop> {
    //     None
    // }

    // fn into_tildekind_loopend(&self) -> Option<&dyn TildeKindLoopEnd> {
    //     None
    // }

    // fn into_tildekind_tildes(&self) -> Option<&dyn TildeKindTildes> {
    //     None
    // }

    // fn into_tildekind_cond(&self) -> Option<&dyn TildeKindCond> {
    //     None
    // }

    // fn into_tildekind_text(&self) -> Option<&dyn TildeKindText> {
    //     None
    // }

    // fn into_tildekind_vectilde(&self) -> Option<&dyn TildeKindVecTilde> {
    //     None
    // }

    fn len(&self) -> usize {
        1
    }
}

impl TildeKindVa for MyStruct {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("a: {}, b: {}", self.a, self.b)))
    }
}

impl TildeKindDigit for MyStruct {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("{}", self.a)))
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
