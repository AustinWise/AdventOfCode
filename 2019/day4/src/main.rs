#[derive(Debug, PartialEq)]
enum PasswordValidity {
    Valid,
    WrongLength,
    NotMonotonicallyIncreasing,
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
                return PasswordValidity::NotMonotonicallyIncreasing;
            } else if p == digit {
                repeated_digits = true;
            }
        }

        pass /= 10;
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

enum DigitState {
    None,
    One(u32),
    Two(u32),
    More(u32),
}

fn is_valid_password2(password: u32) -> PasswordValidity {
    let mut pass = password;
    let mut num_digits = 0;
    let mut repeated_digits = false;
    let mut prev: DigitState = DigitState::None;
    while pass != 0 {
        num_digits += 1;
        let digit = pass % 10;

        {
            let p = match prev {
                DigitState::One(p) => p,
                DigitState::Two(p) => p,
                DigitState::More(p) => p,
                DigitState::None => 99, // larger than any possible digit
            };
            if p < digit {
                return PasswordValidity::NotMonotonicallyIncreasing;
            }
        }

        pass /= 10;
        prev = match prev {
            DigitState::None => DigitState::One(digit),
            DigitState::One(p) => {
                if p == digit {
                    DigitState::Two(digit)
                } else {
                    DigitState::One(digit)
                }
            }
            DigitState::Two(p) => {
                if p == digit {
                    DigitState::More(digit)
                } else {
                    repeated_digits = true;
                    DigitState::One(digit)
                }
            }
            DigitState::More(p) => {
                if p == digit {
                    DigitState::More(digit)
                } else {
                    DigitState::One(digit)
                }
            }
        }
    }

    if let DigitState::Two(_) = prev {
        repeated_digits = true;
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
    println!("valid passwords part1: {}", valid_passwords);

    let mut valid_passwords = 0;
    for i in 171309..643603 {
        if is_valid_password2(i) == PasswordValidity::Valid {
            valid_passwords += 1;
        }
    }
    println!("valid passwords part2: {}", valid_passwords);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_password_valitity() {
        assert_eq!(PasswordValidity::Valid, is_valid_password(111111));
        assert_eq!(
            PasswordValidity::NotMonotonicallyIncreasing,
            is_valid_password(223450)
        );
        assert_eq!(
            PasswordValidity::NoRepeatedDigits,
            is_valid_password(123789)
        );
    }

    #[test]
    fn test_password_valitity2() {
        assert_eq!(PasswordValidity::Valid, is_valid_password2(112233));
        assert_eq!(
            PasswordValidity::NoRepeatedDigits,
            is_valid_password2(123444)
        );
        assert_eq!(PasswordValidity::Valid, is_valid_password2(111122));
    }
}
