// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The renderer: turns a [`CommandOutput`] (or a self-describing JSON document)
//! into bytes on stdout, honoring the resolved [`OutputMode`], `--fields`
//! projection, and the human/machine split. This is the only module that writes
//! to stdout (CLI Standard §7).

use std::io::{IsTerminal as _, Write as _};

use owo_colors::OwoColorize as _;
use serde_json::Value;

use crate::context::Context;
use crate::output::mode::OutputMode;
use crate::output::{envelope, theme, CommandOutput, HumanRender};

/// Render a data-returning command's output in the active mode.
pub(crate) fn emit(ctx: &Context, output: &CommandOutput) {
    match ctx.mode {
        OutputMode::Json | OutputMode::Explore => emit_json(ctx, &output.data),
        OutputMode::Jsonl => emit_jsonl(ctx, &output.data),
        OutputMode::Yaml => emit_yaml(ctx, &output.data),
        OutputMode::Csv => emit_csv(ctx, &output.data),
        OutputMode::HumanWithColor => emit_human(&output.human, true),
        OutputMode::HumanNoColor => emit_human(&output.human, false),
    }
}

/// Render a self-describing JSON document (`schema` / `describe`) directly,
/// bypassing the data envelope. Honors `--format yaml`; everything else renders
/// as JSON (pretty on a TTY, compact when piped).
pub(crate) fn emit_raw_json(ctx: &Context, value: &Value) {
    if ctx.mode == OutputMode::Yaml {
        match serde_yaml::to_string(value) {
            Ok(text) => print_line(text.trim_end()),
            Err(_) => print_json_doc(value, true),
        }
    } else {
        print_json_doc(value, std::io::stdout().is_terminal());
    }
}

// ── machine modes ───────────────────────────────────────────────────────────

fn emit_json(ctx: &Context, data: &Value) {
    let projected = project(data, ctx.fields.as_deref());
    let document = envelope::wrap(ctx, projected);
    print_json_doc(&document, std::io::stdout().is_terminal());
}

fn emit_jsonl(ctx: &Context, data: &Value) {
    let projected = project(data, ctx.fields.as_deref());
    // First line: metadata with a null data slot.
    let meta = serde_json::json!({ "metadata": envelope::metadata(ctx), "data": Value::Null });
    print_json_doc(&meta, false);
    match projected {
        Value::Array(items) => {
            for item in &items {
                print_json_doc(item, false);
            }
        }
        other => print_json_doc(&other, false),
    }
}

fn emit_yaml(ctx: &Context, data: &Value) {
    let projected = project(data, ctx.fields.as_deref());
    let document = envelope::wrap(ctx, projected);
    match serde_yaml::to_string(&document) {
        Ok(text) => print_line(text.trim_end()),
        Err(_) => print_json_doc(&document, false),
    }
}

fn emit_csv(ctx: &Context, data: &Value) {
    let projected = project(data, ctx.fields.as_deref());
    // CSV is flat and carries no envelope; metadata goes to stderr (§5).
    if let Ok(meta) = serde_json::to_string(&envelope::metadata(ctx)) {
        eprintln!("# {meta}");
    }
    write_csv(&projected);
}

/// Collect the record list for CSV: an array stays as-is, a single object
/// becomes a one-row table, and a scalar becomes a single `value` column.
fn write_csv(data: &Value) {
    let records: Vec<Value> = match data {
        Value::Array(items) => items.clone(),
        Value::Null => Vec::new(),
        other => vec![other.clone()],
    };

    // Column order: keys of the first object record, then any new keys after.
    let mut headers: Vec<String> = Vec::new();
    for record in &records {
        if let Value::Object(map) = record {
            for key in map.keys() {
                if !headers.iter().any(|h| h == key) {
                    headers.push(key.clone());
                }
            }
        }
    }
    let scalar_mode = headers.is_empty();
    if scalar_mode {
        headers.push("value".to_owned());
    }

    let stdout = std::io::stdout();
    let mut writer = csv::Writer::from_writer(stdout.lock());
    if writer.write_record(&headers).is_err() {
        return;
    }
    for record in &records {
        let row: Vec<String> = if scalar_mode {
            vec![scalar_to_string(record)]
        } else {
            headers
                .iter()
                .map(|key| match record {
                    Value::Object(map) => map.get(key).map_or(String::new(), scalar_to_string),
                    _ => String::new(),
                })
                .collect()
        };
        if writer.write_record(&row).is_err() {
            return;
        }
    }
    let _ = writer.flush();
}

/// Stringify a JSON value for a flat cell: scalars verbatim, `null` as empty,
/// nested values as compact JSON.
fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

// ── human mode ──────────────────────────────────────────────────────────────

fn emit_human(human: &HumanRender, color: bool) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match human {
        HumanRender::Message(text) => {
            let _ = writeln!(out, "{text}");
        }
        HumanRender::Summary(pairs) => {
            let width = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            for (key, value) in pairs {
                let label = format!("{key:<width$}");
                if color {
                    let _ = writeln!(
                        out,
                        "{}  {}",
                        label.truecolor(
                            theme::STEEL_BLUE.0,
                            theme::STEEL_BLUE.1,
                            theme::STEEL_BLUE.2
                        ),
                        value.truecolor(
                            theme::LIQUID_COOLANT.0,
                            theme::LIQUID_COOLANT.1,
                            theme::LIQUID_COOLANT.2
                        )
                    );
                } else {
                    let _ = writeln!(out, "{label}  {value}");
                }
            }
        }
        HumanRender::Table { headers, rows } => {
            let widths = column_widths(headers, rows);
            let header_line = join_row(headers, &widths);
            if color {
                let _ = writeln!(
                    out,
                    "{}",
                    header_line.truecolor(
                        theme::MOLTEN_AMBER.0,
                        theme::MOLTEN_AMBER.1,
                        theme::MOLTEN_AMBER.2
                    )
                );
            } else {
                let _ = writeln!(out, "{header_line}");
            }
            let rule: String = widths
                .iter()
                .map(|w| "─".repeat(*w))
                .collect::<Vec<_>>()
                .join("  ");
            let _ = writeln!(out, "{rule}");
            for row in rows {
                let _ = writeln!(out, "{}", join_row(row, &widths));
            }
        }
    }
    let _ = out.flush();
}

fn column_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths: Vec<usize> = headers.iter().map(String::len).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(w) = widths.get_mut(i) {
                *w = (*w).max(cell.len());
            }
        }
    }
    widths
}

fn join_row(cells: &[String], widths: &[usize]) -> String {
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let w = widths.get(i).copied().unwrap_or(cell.len());
            format!("{cell:<w$}")
        })
        .collect::<Vec<_>>()
        .join("  ")
        .trim_end()
        .to_owned()
}

// ── shared helpers ──────────────────────────────────────────────────────────

/// Project a data value onto a `--fields` subset. Objects keep the listed keys
/// (in the requested order); arrays project each element; scalars pass through.
fn project(data: &Value, fields: Option<&[String]>) -> Value {
    let Some(fields) = fields else {
        return data.clone();
    };
    match data {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for field in fields {
                if let Some(v) = map.get(field) {
                    out.insert(field.clone(), v.clone());
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| project(item, Some(fields)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Serialize a JSON document to stdout, pretty or compact, with a trailing
/// newline and an explicit flush.
fn print_json_doc(value: &Value, pretty: bool) {
    let text = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .unwrap_or_else(|_| String::from("{}"));
    print_line(&text);
}

/// Write one line to a locked stdout handle and flush.
fn print_line(text: &str) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "{text}");
    let _ = out.flush();
}
