mod big_number;

fn main() {
    // A signed n-bit type can represent -2ⁿ⁻¹, but not 2ⁿ⁻¹.
    assert_eq!((-128_i8).checked_div(-1), None);

    // so we get 250000 modulo 2¹⁶
    assert_eq!(500_u16.wrapping_mul(500), 53392);

    // Operations on signed types may wrap to negative values.
    assert_eq!(500_i16.wrapping_mul(500), -12144);
    assert_eq!((-128_i8).checked_neg(), None);

    // So a shift of 17 bits in a 16-bit type is a shift of 1
    assert_eq!(5_i16.wrapping_shl(17), 10);

    // Overflowing operations return a tuple (result, overflowed)
    assert_eq!(255_u8.overflowing_sub(2), (253, false));
    assert_eq!(255_u8.overflowing_add(2), (1, true));

    assert!((-1. / f32::INFINITY).is_sign_negative());
    assert_eq!(-f32::MIN, f32::MAX);
    assert_eq!(std::char::from_digit(2, 10), Some('2'));

    println!("\u{CA0}_\u{CA0}");

    /*
    let mut i: u64 = 1;
    loop {
        i = i.checked_mul(10).expect("overflow!");
    }
     */
    print_padovan();

    let mut composers = Vec::new();
    composers.push(Composer {
        name: "Palestrina".to_string(),
        birth_year: 1525,
    });
    composers.push(Composer {
        name: "Lully".to_string(),
        birth_year: 1632,
    });
    composers.push(Composer {
        name: "Bach".to_string(),
        birth_year: 1685,
    });
    composers.push(Composer {
        name: "Handel".to_string(),
        birth_year: 1685,
    });
    composers.push(Composer {
        name: "Haydn".to_string(),
        birth_year: 1732,
    });
    print_persons(&composers);

    println!("{}", &composers[0].name);
}

fn print_padovan() {
    let mut padovan = vec![1, 1, 1];
    for i in 3..10 {
        println!("iteration: {}", i);

        let next = padovan[i - 2] + padovan[i - 3];
        padovan.push(next);
    }
    println!("P(1..10) = {:?}", padovan);
}

struct Composer {
    name: String,
    birth_year: i32,
}

fn print_persons(composers: &Vec<Composer>) {
    for composer in composers {
        println!("{} was born in {}", composer.name, composer.birth_year);
    }
}
