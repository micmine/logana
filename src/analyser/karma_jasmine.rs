use crate::types;

/// Contains the analyser code for the [`crate::config::ParserKind::KarmaJasmine`]
#[must_use]
pub fn analyse(lines: &[&str], project_dir: &str) -> types::AnalyseReport {
    let mut errors: Vec<types::Message> = vec![];
    let line_len = &lines.len();

    for i in 0..lines.len() {
        if let Some(line) = lines.get(i) {
            if line.ends_with(" FAILED[39m") {
                if let Some(error) = lines.get(i + 1) {
                    let error = error.trim();

                    for y in 2.. {
                        let i = i + y;
                        if let Some(line) = lines.get(i) {
                            let line_trimmed = line.trim();
                            if !line_trimmed.starts_with("at ") {
                                break;
                            }

                            if line_trimmed.contains("(src/app") {
                                if let Some(location) =
                                    parse_test_location(line_trimmed, project_dir)
                                {
                                    errors.push(types::Message {
                                        error: error.to_string(),
                                        locations: vec![location],
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    types::AnalyseReport { errors }
}

fn parse_test_location(location: &str, project_dir: &str) -> Option<types::Location> {
    if let Some((_, location)) = location.split_once('(') {
        if let Some((path, row_col)) = location.split_once(':') {
            let path = format!("{}/{}", project_dir, path);

            let row_col = &row_col[..row_col.len() - 1];
            if let Some((row, col)) = row_col.split_once(':') {
                let row = row.parse::<usize>().unwrap_or_default();
                let col = col.parse::<usize>().unwrap_or_default();

                return Some(types::Location { path, row, col });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::{analyser::karma_jasmine::analyse, types};

    #[test]
    fn should_find_syntax_error() {
        static LOG: &'static str = include_str!("../../tests/karma_jasmine_1.log");
        let result = analyse(LOG, "/tmp/project");

        assert_eq!(
            result,
            types::AnalyseReport {
                errors: vec![
                    types::Message {
                        error: "Expected true to be false.".to_string(),
                        locations: vec![types::Location {
                            path: "/tmp/project/src/app/app.component.spec.ts".to_string(),
                            row: 35,
                            col: 18
                        }]
                    },
                    types::Message {
                        error: "Expected OtherServiceService({  }) to be false.".to_string(),
                        locations: vec![types::Location {
                            path: "/tmp/project/src/app/components/other-service.service.spec.ts"
                                .to_string(),
                            row: 14,
                            col: 21
                        }]
                    }
                ],
            }
        )
    }
}
