use deunicode::deunicode_char;

pub fn slugify_camel(str: &str) -> String {
    let mut slug = String::with_capacity(str.len());

    let mut is_start_of_word = false;
    let mut add_char = |c: char| {
        match c {
            '0'..='9' | 'A'..='Z' => slug.push(c),
            'a'..='z' if is_start_of_word => slug.push(c.to_ascii_uppercase()),
            'a'..='z' if !is_start_of_word => slug.push(c),

            _ => (),
        }

        is_start_of_word = !c.is_ascii_alphanumeric();
    };

    for char in str.chars() {
        if char.is_ascii() {
            add_char(char);
        } else if let Some(deunicoded) = deunicode_char(char) {
            deunicoded.chars().for_each(&mut add_char);
        }
    }

    slug.shrink_to_fit();
    slug
}

#[cfg(test)]
mod tests {
    use crate::commands::camel_slug::slugify_camel;

    #[test]
    fn simple() {
        assert_eq!(slugify_camel("JEEZ Game Jam 2023"), "JEEZGameJam2023");
    }

    #[test]
    fn simple_lower() {
        assert_eq!(slugify_camel("JEEZ game jam 2023"), "JEEZGameJam2023");
    }

    #[test]
    fn extra_ascii() {
        assert_eq!(
            slugify_camel("1234.foo#&%$*&barJam*&^*(=="),
            "1234FooBarJam"
        );
    }

    #[test]
    fn extra_non_ascii_translit() {
        assert_eq!(slugify_camel("_-_-_-Тест Jam"), "TestJam");
    }

    #[test]
    fn already_camel() {
        assert_eq!(
            slugify_camel("PerfectlyValidCamelCase1337"),
            "PerfectlyValidCamelCase1337"
        );
    }
}
