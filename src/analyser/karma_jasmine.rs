use crate::types;

/// Contains the analyser code for the [`crate::config::ParserKind::KarmaJasmine`]
#[must_use]
pub fn analyse(log: &str, project_dir: &str) -> types::AnalyseReport {
    let mut errors: Vec<types::Message> = vec![];
    let lines = log.lines().collect::<Vec<&str>>();
    let lines = lines.as_slice();
    let line_len = &lines.len();

    for i in 0..*line_len {
        if let Some(line) = lines.get(i) {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("Error: ")
                || line_trimmed.starts_with("Usage:")
                || line_trimmed.starts_with("TypeError:")
            {
                let mut exception = vec![line_trimmed];
                'exception: for y in 1.. {
                    let i: usize = i + y;
                    let Some(line) = lines.get(i) else {
                      break 'exception;
                    };

                    let line = line.trim();

                    if !line.starts_with("at ") {
                        break 'exception;
                    }

                    exception.push(line);
                }
                let exception = exception.as_slice();
                if let Some(message) = parse_exception(exception, project_dir) {
                    errors.push(message);
                }
            }
            if line.ends_with(" FAILED[39m") {
                if let Some(error) = lines.get(i + 1) {
                    let error = error.trim();

                    'search_error: for y in 2.. {
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
                                    });
                                    break 'search_error;
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

#[must_use]
fn parse_exception(log: &[&str], project_dir: &str) -> Option<types::Message> {
    let first_line = log.first().expect("A exception should have a fist line");
    let Some((_, error)) = first_line.split_once(": ") else {
        return None;
    };

    let mut locations = vec![];

    'locations: for i in 1.. {
        let Some(line) = log.get(i) else {
                  break 'locations;
                };

        // without closing bracket
        let line: &str = &line[1..line.len() - 1];

        let mut location = String::new();

        if let Some((_, location_w)) = line.split_once("_karma_webpack_/webpack:") {
            location = location_w.to_owned();
        }

        if let Some((_, location_w)) = line.split_once(" (") {
            if location_w.starts_with("src/") {
                location = location_w.to_owned();
            }
        }

        //// without closing bracket
        //let location: &str = &location[1..location.len() - 1];

        if !location.starts_with('/') {
            location = "/".to_owned() + &location;
        }

        if !location.starts_with("/src/") {
            continue;
        }

        let Some((path, row_col)) = location.split_once(':') else {
                continue;
            };
        let path = format!("{project_dir}{path}");

        let Some((row, col)) = row_col.split_once(':') else  {
                    continue;
                };

        let row = row.parse::<usize>().unwrap_or_default();
        let col = col.parse::<usize>().unwrap_or_default();

        locations.push(types::Location { path, row, col });
    }

    Some(types::Message {
        error: error.to_string(),
        locations,
    })
}

fn parse_test_location(location: &str, project_dir: &str) -> Option<types::Location> {
    if let Some((_, location)) = location.split_once('(') {
        if let Some((path, row_col)) = location.split_once(':') {
            let path = format!("{project_dir}/{path}");

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
    use crate::{
        analyser::karma_jasmine::{analyse, parse_exception},
        types,
    };

    #[test]
    fn should_find_syntax_error() {
        static LOG: &str = include_str!("../../tests/karma_jasmine_1.log");
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
        );
    }

    #[test]
    fn should_parse_exception_1() {
        static LOG: &str = include_str!("../../tests/karma_jasmine_exeption_1.log");
        let result = parse_exception(LOG.lines().collect::<Vec<&str>>().as_slice(), "/tmp/project");
        assert_eq!(result, Some(types::Message {
            error: "Cannot read property 'component' of undefined".to_string(), 
            locations: vec![types::Location {
                path: "/tmp/project/src/app/components/layout/main/command-info-dialog-modal/command-info-dialog-modal.component.ts".to_string(),
                row: 83,
                col: 1
            }]})
        );
    }

    #[test]
    fn should_parse_exception_2() {
        static LOG: &str = include_str!("../../tests/karma_jasmine_exeption_2.log");
        let result = parse_exception(LOG.lines().collect::<Vec<&str>>().as_slice(), "/tmp/project");

        assert_eq!(result, Some(types::Message {
            error: "Expected '12.08.2021 08:01:06' to equal '12.08.2021 09:01:06'.".to_string(), 
            locations: vec![types::Location {
                path: "/tmp/project/src/app/components/layout/main/alarm-info-dialog-modal/functions/alarm-info-calculated-fields.functions.spec.ts".to_string(),
                row: 80,
                col: 22
            }]})
        );
    }
}
