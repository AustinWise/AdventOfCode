#[derive(Debug, PartialEq)]
enum PasswordValidity {
    Valid,
    WrongLength,
    NotMonotonicallyIncrasing,
    NoRepeatedDigits,
}

fn is_valid_password(password: u32) -> PasswordValidity {
    let mut pass = password;
    let mut num_digits = 0;
    let mut repeated_digits = false;
    let mut prev: Option<u32> = None;
    while pass != 0 {
        num_digits += 1;
        let digit = pass % 10;

        //We want to make sure when the number is writing in
        //decimal the numbers are increasing from let to right.
        //Since we are iterating from right to left,
        //we check that the numbers are decreasing.
        if let Some(p) = prev {
            if p < digit {
                return PasswordValidity::NotMonotonicallyIncrasing;
            } else if p == digit {
                repeated_digits = true;
            }
        }

        pass = pass / 10;
        prev = Some(digit);
    }

    if num_digits != 6 {
        PasswordValidity::WrongLength
    } else if !repeated_digits {
        PasswordValidity::NoRepeatedDigits
    } else {
        PasswordValidity::Valid
    }
}

fn main() {
    let mut valid_passwords = 0;
    for i in 171309..643603 {
        if is_valid_password(i) == PasswordValidity::Valid {
            valid_passwords += 1;
        }
    }
    println!("valid passwords: {}", valid_passwords);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_password_valitity() {
        assert_eq!(PasswordValidity::Valid, is_valid_password(111111));
        assert_eq!(
            PasswordValidity::NotMonotonicallyIncrasing,
            is_valid_password(223450)
        );
        assert_eq!(
            PasswordValidity::NoRepeatedDigits,
            is_valid_password(123789)
        );
    }
}
