use crate::errors::*;
use bloodhound::ExclusionPattern;
use crate::yaml::Yaml;

pub fn parse(exclusion_data: &Vec<Yaml>) -> Result<Vec<ExclusionPattern>> {
    let mut mapped_exclusions = Vec::new();

    for exclusion in exclusion_data.iter() {
        if let &Yaml::String(ref pattern) = exclusion {
            mapped_exclusions.push(
                ExclusionPattern::new(&pattern).chain_err(|| {
                    format!("Failed to parse exclusion pattern: {}", pattern)
                })?
            );
        } else {
            bail!("Found a non-string exclusion that can't be parsed.");
        }
    }

    Ok(mapped_exclusions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_converts_yaml_strings_into_glob_patterns() {
        let exclusion_data = vec![Yaml::String(String::from("pattern"))];
        assert_eq!(
            parse(&exclusion_data).unwrap(),
            vec![ExclusionPattern::new("pattern").unwrap()]
        );
    }

    #[test]
    fn parse_returns_an_error_when_parsing_fails() {
        let exclusion_data = vec![Yaml::String(String::from("["))];

        assert!(parse(&exclusion_data).is_err());
    }

    #[test]
    fn parse_returns_an_error_when_a_non_string_is_yaml_value_is_encountered() {
        let exclusion_data = vec![Yaml::Boolean(true)];

        assert!(parse(&exclusion_data).is_err());
    }
}
