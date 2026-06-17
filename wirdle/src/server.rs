use crate::coach::{CoachIntent, CoachRequest, coach, coach_response_json};
use crate::feedback::LetterStatus;
use crate::filter::GuessInput;
use crate::solver::{PastSolutionPolicy, SolveMode, SolveRequest, SolveResponse, Solver};
use crate::word::Word;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn serve(addr: &str, solver: Solver) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    eprintln!("wordle-server listening on http://{addr}");
    for stream in listener.incoming() {
        let stream = stream?;
        let solver = solver.clone();
        thread::spawn(move || {
            if let Err(err) = handle_connection(stream, &solver) {
                eprintln!("request failed: {err}");
            }
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, solver: &Solver) -> std::io::Result<()> {
    let buffer = read_http_request(&mut stream)?;
    let request = String::from_utf8_lossy(&buffer);
    let (status, content_type, body) = handle_http_request(&request, solver);
    let response = format!(
        "HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())
}

fn read_http_request(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let bytes_read = stream.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
        let Some(header_end) = find_header_end(&buffer) else {
            continue;
        };
        let headers = String::from_utf8_lossy(&buffer[..header_end]);
        let content_length = content_length(&headers);
        let total_length = header_end + 4 + content_length;
        while buffer.len() < total_length {
            let bytes_read = stream.read(&mut chunk)?;
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);
        }
        break;
    }
    Ok(buffer)
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
}

pub fn handle_http_request(raw: &str, solver: &Solver) -> (&'static str, &'static str, String) {
    let mut lines = raw.lines();
    let first = lines.next().unwrap_or_default();
    if first.starts_with("GET / ") || first.starts_with("GET /index.html ") {
        return (
            "200 OK",
            "text/html; charset=utf-8",
            include_str!("../static/index.html").to_string(),
        );
    }
    if first.starts_with("GET /v1/health ") {
        return ("200 OK", "application/json", solver.health_json());
    }
    if first.starts_with("GET /v1/metadata ") {
        return ("200 OK", "application/json", solver.metadata_json());
    }
    if first.starts_with("POST /v1/solve ") {
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default();
        return match parse_solve_request(body) {
            Ok(request) => match solver.solve(&request) {
                Ok(response) => ("200 OK", "application/json", solve_response_json(&response)),
                Err(message) => (
                    "422 Unprocessable Entity",
                    "application/json",
                    format!(
                        "{{\"error\":\"no_candidates_remaining\",\"message\":\"{}\"}}",
                        escape_json(&message)
                    ),
                ),
            },
            Err(message) => (
                "400 Bad Request",
                "application/json",
                format!(
                    "{{\"error\":\"invalid_request\",\"message\":\"{}\"}}",
                    escape_json(&message)
                ),
            ),
        };
    }
    if first.starts_with("POST /v1/coach ") {
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default();
        return match parse_coach_request(body) {
            Ok(request) => match coach(solver, &request) {
                Ok(response) => ("200 OK", "application/json", coach_response_json(&response)),
                Err(err) => (
                    if err.code == "invalid_request" {
                        "400 Bad Request"
                    } else {
                        "422 Unprocessable Entity"
                    },
                    "application/json",
                    format!(
                        "{{\"error\":\"{}\",\"message\":\"{}\"}}",
                        err.code,
                        escape_json(&err.message)
                    ),
                ),
            },
            Err(message) => (
                "400 Bad Request",
                "application/json",
                format!(
                    "{{\"error\":\"invalid_request\",\"message\":\"{}\"}}",
                    escape_json(&message)
                ),
            ),
        };
    }
    (
        "404 Not Found",
        "application/json",
        "{\"error\":\"not_found\",\"message\":\"Unknown route\"}".to_string(),
    )
}

pub fn parse_coach_request(json: &str) -> Result<CoachRequest, String> {
    let intent_value = json_string(json, "intent").ok_or("coach request is missing intent")?;
    let intent = CoachIntent::parse(&intent_value)
        .ok_or_else(|| format!("unsupported coach intent '{intent_value}'"))?;
    let solve_request = parse_solve_request(json)?;

    Ok(CoachRequest {
        intent,
        guesses: solve_request.guesses,
        hard_mode: json_bool(json, "hard_mode").unwrap_or(false),
    })
}

pub fn parse_solve_request(json: &str) -> Result<SolveRequest, String> {
    let mut guesses = Vec::new();
    for object in array_objects(json, "guesses") {
        let word = json_string(object, "word").ok_or("guess is missing word")?;
        let statuses = json_string_array(object, "statuses");
        if statuses.len() != 5 {
            return Err("guess statuses must contain five values".to_string());
        }
        let mut parsed = [LetterStatus::Absent; 5];
        for (idx, status) in statuses.iter().enumerate() {
            parsed[idx] = LetterStatus::parse(status)
                .ok_or_else(|| format!("unknown letter status '{status}'"))?;
        }
        guesses.push(GuessInput::new(
            Word::parse(&word).map_err(|err| format!("invalid guess word: {err:?}"))?,
            parsed,
        ));
    }

    let mode = json_string(json, "mode")
        .map(|value| SolveMode::parse(&value))
        .unwrap_or(SolveMode::Hybrid);
    let hard_mode = json_bool(json, "hard_mode").unwrap_or(false);
    let limit = json_number(json, "limit").unwrap_or(20).clamp(1, 100) as usize;
    let default_policy = PastSolutionPolicy::default();
    let past_solution_policy = PastSolutionPolicy {
        enabled: json_bool(json, "enabled").unwrap_or(default_policy.enabled),
        weight_multiplier: json_float(json, "weight_multiplier")
            .unwrap_or(default_policy.weight_multiplier),
        recent_repeat_multiplier: json_float(json, "recent_repeat_multiplier")
            .unwrap_or(default_policy.recent_repeat_multiplier),
        recent_days: json_number(json, "recent_days").unwrap_or(default_policy.recent_days as u64)
            as usize,
    };

    Ok(SolveRequest {
        guesses,
        mode,
        hard_mode,
        limit,
        past_solution_policy,
    })
}

fn solve_response_json(response: &SolveResponse) -> String {
    let likely = response
        .likely_answers
        .iter()
        .map(|answer| {
            format!(
                "{{\"word\":\"{}\",\"probability\":{:.6},\"used_before\":{},\"score\":{:.6}}}",
                answer.word, answer.probability, answer.used_before, answer.score
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let information = response
        .best_information_guesses
        .iter()
        .map(|guess| {
            format!(
                "{{\"word\":\"{}\",\"entropy_bits\":{:.6},\"expected_remaining\":{:.6},\"worst_case_remaining\":{},\"is_possible_answer\":{},\"used_before\":{},\"score\":{:.6}}}",
                guess.word,
                guess.entropy_bits,
                guess.expected_remaining,
                guess.worst_case_remaining,
                guess.is_possible_answer,
                guess.used_before,
                guess.score
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"remaining_candidates\":{},\"likely_answers\":[{}],\"best_information_guesses\":[{}]}}",
        response.remaining_candidates, likely, information
    )
}

fn array_objects<'a>(json: &'a str, key: &str) -> Vec<&'a str> {
    let Some(key_start) = json.find(&format!("\"{key}\"")) else {
        return Vec::new();
    };
    let Some(open_rel) = json[key_start..].find('[') else {
        return Vec::new();
    };
    let open = key_start + open_rel;
    let Some(close) = matching(json, open, '[', ']') else {
        return Vec::new();
    };
    objects(&json[open + 1..close])
}

fn objects(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (idx, ch) in text.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(start) = start.take() {
                        out.push(&text[start..=idx]);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn json_string(object: &str, key: &str) -> Option<String> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start;
    let first_quote = object[colon..].find('"')? + colon + 1;
    let second_quote = object[first_quote..].find('"')? + first_quote;
    Some(object[first_quote..second_quote].to_string())
}

fn json_string_array(object: &str, key: &str) -> Vec<String> {
    let Some(key_start) = object.find(&format!("\"{key}\"")) else {
        return Vec::new();
    };
    let Some(open_rel) = object[key_start..].find('[') else {
        return Vec::new();
    };
    let open = key_start + open_rel;
    let Some(close) = matching(object, open, '[', ']') else {
        return Vec::new();
    };
    object[open + 1..close]
        .split(',')
        .map(|part| part.trim().trim_matches('"').to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn json_bool(object: &str, key: &str) -> Option<bool> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start + 1;
    let value: String = object[colon..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect();
    match value.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn json_number(object: &str, key: &str) -> Option<u64> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start + 1;
    let digits: String = object[colon..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn json_float(object: &str, key: &str) -> Option<f64> {
    let start = object.find(&format!("\"{key}\""))?;
    let colon = object[start..].find(':')? + start + 1;
    let number: String = object[colon..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    number.parse().ok()
}

fn matching(text: &str, open_idx: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in text[open_idx..].char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(open_idx + idx);
            }
        }
    }
    None
}

fn escape_json(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
