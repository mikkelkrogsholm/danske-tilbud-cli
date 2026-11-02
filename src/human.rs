//! Static presentation of the exact values returned by the command engine.
use chrono::{DateTime, Utc};
use iocraft::prelude::*;
use serde_json::Value;
use std::io::{self, IsTerminal, Write};

fn string(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}
fn age(v: &Value) -> String {
    let s = string(v);
    match DateTime::parse_from_rfc3339(s) {
        Ok(t) => {
            let minutes = (Utc::now() - t.with_timezone(&Utc)).num_minutes().max(0);
            let elapsed = if minutes < 60 {
                format!("{minutes} min")
            } else if minutes < 1440 {
                format!("{} timer", minutes / 60)
            } else {
                format!("{} dage", minutes / 1440)
            };
            format!("{s} ({elapsed} siden)")
        }
        Err(_) => "ukendt".into(),
    }
}
fn money(ore: &Value) -> String {
    match ore.as_u64() {
        Some(n) => format!("{},{:02} kr.", n / 100, n % 100),
        None => "ukendt pris".into(),
    }
}
fn boundary(v: &Value) -> &str {
    if string(&v["precision"]) == "unknown" {
        "ukendt"
    } else {
        v["value"].as_str().unwrap_or("ukendt")
    }
}
fn description(data: &Value, lines: &mut Vec<String>) {
    lines.push(format!(
        "{} ({})",
        string(&data["name"]),
        string(&data["id"])
    ));
    let cap = &data["capabilities"];
    lines.push(format!(
        "Kildefiltre: butik {}, dato {}",
        if cap["store_filter"] == true {
            "ja"
        } else {
            "nej"
        },
        if cap["date_filter"] == true {
            "ja"
        } else {
            "nej"
        }
    ));
    let chains = data["chains"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    lines.push(format!("Kæder: {chains}"));
    lines.push(format!("Dækning: {}", string(&data["coverage_limitation"])));
    let snap = &data["snapshot"];
    if snap.is_null() {
        lines.push("Snapshot: endnu ikke hentet".into());
    } else if snap["error"].is_string() {
        lines.push("Snapshot: kan ikke læses".into());
    } else {
        lines.push(format!(
            "Snapshot: {} tilbud, hentet {}",
            snap["offer_count"],
            age(&snap["fetched_at"])
        ));
    }
}
fn document(value: &Value) -> (String, Vec<String>) {
    let cmd = string(&value["command"]);
    let data = &value["data"];
    let mut lines = Vec::new();
    if value["error"].is_string() {
        if string(&value["error"]) == "incomplete_run" {
            lines.push(format!(
                "Hentning af {} er ufuldstændig. Intet nyt snapshot er gemt.",
                string(&value["source"])
            ));
            lines.push(format!(
                "{} tilbud modtaget; {} kildefejl og {} ugyldige poster.",
                value["offer_count"],
                value["source_errors"].as_array().map_or(0, Vec::len),
                value["invalid_records"].as_array().map_or(0, Vec::len)
            ));
            if let Some(errors) = value["source_errors"].as_array() {
                for e in errors {
                    let prefix = string(e).split(':').next().unwrap_or("");
                    lines.push(format!("Kunne ikke hente eller læse: {prefix}"));
                }
            }
            limitation(&value["coverage_limitation"], &mut lines);
            coverage(&value["coverage"], &mut lines);
        } else {
            lines.push(
                value["message"]
                    .as_str()
                    .unwrap_or("Forespørgslen mislykkedes.")
                    .into(),
            );
        }
        return ("Fejl".into(), lines);
    }
    match cmd {
        "kilder" => {
            if let Some(sources) = data["sources"].as_array() {
                for s in sources {
                    description(s, &mut lines);
                }
            }
        }
        "beskriv" => description(data, &mut lines),
        "hent" => {
            lines.push(format!("{}: hentning fuldført", string(&data["source"])));
            lines.push(format!(
                "{} tilbud gemt; hentet {}",
                data["offer_count"],
                age(&data["fetched_at"])
            ));
            limitation(&data["coverage_limitation"], &mut lines);
            coverage(&data["coverage"], &mut lines);
        }
        "find" => {
            let results = data["results"].as_array();
            lines.push(format!(
                "Søgning: {} · {} match, {} vist",
                string(&data["query"]),
                data["total"],
                results.map_or(0, Vec::len)
            ));
            lines.push(format!("Kildedata hentet: {}", age(&data["fetched_at"])));
            if let Some(results) = results {
                if results.is_empty() {
                    lines.push(
                        "Ingen tilbud fundet. Prøv en anden søgning eller ændr filtrene.".into(),
                    );
                }
                for result in results {
                    let o = &result["offer"];
                    let heading = o["heading"]
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or("Navn ikke oplyst");
                    lines.push(format!("\n{} · {heading}", string(&o["chain_id"])));
                    let mut price = format!("Pris: {}", money(&o["price"]["amount_ore"]));
                    if let (Some(ore), Some(unit)) = (
                        result["unit_price"]["ore"].as_str(),
                        result["unit_price"]["unit"].as_str(),
                    ) {
                        if let Ok(n) = ore.parse::<rust_decimal::Decimal>() {
                            price.push_str(&format!(
                                " · Enhedspris: {} kr./{unit}",
                                (n / rust_decimal::Decimal::from(100))
                                    .round_dp(2)
                                    .to_string()
                                    .replace('.', ",")
                            ));
                        }
                    }
                    lines.push(price);
                    if let Some(description) =
                        o["description"].as_str().filter(|s| !s.trim().is_empty())
                    {
                        lines.push(format!("Beskrivelse: {description}"));
                    }
                    lines.push(format!(
                        "Gyldig: {} til {}",
                        boundary(&o["validity"]["starts"]),
                        boundary(&o["validity"]["ends"])
                    ));
                    lines.push(format!(
                        "Medlemskab: {}",
                        match string(&o["membership"]) {
                            "required" => "kræves",
                            "not_required" => "kræves ikke",
                            _ => "ukendt",
                        }
                    ));
                    let store = &o["store"];
                    let details = [
                        store["name"].as_str(),
                        store["location"].as_str(),
                        store["store_id"].as_str(),
                    ]
                    .into_iter()
                    .flatten()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                    lines.push(format!(
                        "Butik: {}",
                        if details.is_empty() {
                            "ikke oplyst"
                        } else {
                            &details
                        }
                    ));
                    lines.push(format!(
                        "Kilde: {} · {}",
                        string(&o["source"]),
                        string(&o["source_reference"])
                    ));
                }
            }
        }
        _ => lines.push("Resultatet kunne ikke vises.".into()),
    }
    (cmd.to_uppercase(), lines)
}
fn limitation(v: &Value, lines: &mut Vec<String>) {
    if let Some(text) = v.as_str().filter(|s| !s.is_empty()) {
        lines.push(format!("Dækning: {text}"));
    }
}
fn coverage(v: &Value, lines: &mut Vec<String>) {
    if let Some(s) = v.as_str() {
        for item in s.split("; ") {
            if let Some((chain, status)) = item.split_once(": ") {
                if status.contains("partial") {
                    lines.push(format!("{chain}: delvis dækning"));
                } else if status.contains("complete for scope") {
                    lines.push(format!("{chain}: fuldført inden for kildens dækning"));
                }
            }
        }
    }
}
pub fn write(value: &Value, error: bool) -> io::Result<()> {
    let (title, lines) = document(value);
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|w| *w > 0)
        .or_else(|| terminal_size::terminal_size().map(|(terminal_size::Width(w), _)| w))
        .unwrap_or(80)
        .clamp(1, 100);
    let mut view = element! {
        View(flex_direction: FlexDirection::Column, width: width as u32, flex_shrink: 0.0) {
            Text(content: format!("TILBUD / {title}"), color: Color::Cyan, weight: Weight::Bold)
            #(lines.iter().map(|line| element! { Text(content: line.clone()) }))
        }
    };
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    if !error
        && io::stdout().is_terminal()
        && !no_color
        && std::env::var("TERM").as_deref() != Ok("dumb")
    {
        view.write_to_is_terminal(io::stdout())
    } else {
        let rendered = view.render(Some(width.into())).to_string();
        if error {
            let mut out = io::stderr().lock();
            out.write_all(rendered.as_bytes())?;
            out.flush()
        } else {
            let mut out = io::stdout().lock();
            out.write_all(rendered.as_bytes())?;
            out.flush()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fetch_uses_the_supplied_source_limitation() {
        let value = json!({"command":"hent", "data":{"source":"anden", "complete":true, "offer_count":2, "fetched_at":null, "coverage_limitation":"Kun udvalgte lokale butikker", "coverage":null}});
        let (_, lines) = document(&value);
        let output = lines.join("\n");
        assert!(output.contains("Kun udvalgte lokale butikker"));
        assert!(!output.contains("Tjek"));
        assert!(!output.contains("Bilka"));
    }

    #[test]
    fn unknown_values_and_partial_errors_are_explained() {
        let value = json!({"command":"find", "data":{"query":"mælk", "total":1, "fetched_at":null, "results":[{"offer":{"chain_id":"netto", "heading":null, "price":{"amount_ore":null,"before_ore":900}, "validity":{"starts":{"precision":"unknown"},"ends":{"precision":"unknown"}}, "membership":"unknown", "store":null, "source":"tjek", "source_reference":"x"}}]}});
        let (_, lines) = document(&value);
        let output = lines.join("\n");
        for term in [
            "Navn ikke oplyst",
            "ukendt pris",
            "ukendt til ukendt",
            "Medlemskab: ukendt",
            "Butik: ikke oplyst",
        ] {
            assert!(output.contains(term), "{output}");
        }
        assert!(!output.contains("9,00"));
        let error = json!({"command":"hent", "error":"incomplete_run", "source":"tjek", "offer_count":4, "source_errors":["netto offers: technical payload"], "invalid_records":["bad"], "coverage":"netto: partial (4 records; direct feed)"});
        let (_, lines) = document(&error);
        let output = lines.join("\n");
        for term in [
            "ufuldstændig",
            "Intet nyt snapshot",
            "1 kildefejl",
            "1 ugyldige",
            "netto: delvis dækning",
        ] {
            assert!(output.contains(term), "{output}");
        }
        assert!(!output.contains("technical payload"));
    }
}
