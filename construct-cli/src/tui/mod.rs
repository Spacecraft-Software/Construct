// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `--format explore` TUI: an interactive, Steelbore-themed browser of the
//! skill catalogue and the agent registry, built on ratatui + crossterm. It
//! never runs for agents/CI/non-TTY (the [`crate::output::mode`] cascade guards
//! that). On exit it returns a [`TuiAction`] the caller executes after the
//! terminal is restored, so installs/syncs print their result normally.

use std::collections::BTreeSet;
use std::path::Path;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::{DefaultTerminal, Frame};

use crate::context::Context;
use crate::install::plan::DEFAULT_SOURCE;
use crate::output::error::{AppError, ErrorCode};
use crate::output::theme;
use crate::registry::{self, detect};
use crate::sources::{self, DiscoveredSkill};

/// What the user asked the TUI to do on exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiAction {
    /// Leave without doing anything.
    Quit,
    /// Run `skill sync`.
    Sync,
    /// Install the given skills (project-local, into detected agents).
    Install(Vec<String>),
}

/// Which list the user is browsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Skills,
    Agents,
}

/// A precomputed agent row for display.
struct AgentRow {
    id: String,
    display_name: String,
    project_path: String,
    global_path: String,
    installed: bool,
    hm_managed: bool,
}

/// TUI state.
struct App {
    tab: Tab,
    skills: Vec<DiscoveredSkill>,
    agents: Vec<AgentRow>,
    filter: String,
    searching: bool,
    skill_state: ListState,
    agent_state: ListState,
    marks: BTreeSet<String>,
}

impl App {
    fn new() -> Self {
        let skills = sources::discover(Path::new(DEFAULT_SOURCE));
        let agents = registry::all()
            .into_iter()
            .map(|a| AgentRow {
                installed: detect::detect_installed(&a),
                hm_managed: detect::global_is_hm_managed(&a),
                global_path: a
                    .global_path
                    .clone()
                    .unwrap_or_else(|| "(project-only)".to_owned()),
                id: a.id,
                display_name: a.display_name,
                project_path: a.project_path,
            })
            .collect();
        let mut skill_state = ListState::default();
        skill_state.select(Some(0));
        let mut agent_state = ListState::default();
        agent_state.select(Some(0));
        Self {
            tab: Tab::Skills,
            skills,
            agents,
            filter: String::new(),
            searching: false,
            skill_state,
            agent_state,
            marks: BTreeSet::new(),
        }
    }

    /// Indices of skills matching the current filter.
    fn filtered_skills(&self) -> Vec<usize> {
        let f = self.filter.to_lowercase();
        self.skills
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                f.is_empty()
                    || format!("{} {}", s.name, s.description.clone().unwrap_or_default())
                        .to_lowercase()
                        .contains(&f)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Indices of agents matching the current filter.
    fn filtered_agents(&self) -> Vec<usize> {
        let f = self.filter.to_lowercase();
        self.agents
            .iter()
            .enumerate()
            .filter(|(_, a)| f.is_empty() || a.id.to_lowercase().contains(&f))
            .map(|(i, _)| i)
            .collect()
    }

    /// Move the active list selection one row up or down, clamped to the range.
    fn move_sel(&mut self, down: bool) {
        let len = match self.tab {
            Tab::Skills => self.filtered_skills().len(),
            Tab::Agents => self.filtered_agents().len(),
        };
        if len == 0 {
            return;
        }
        let state = match self.tab {
            Tab::Skills => &mut self.skill_state,
            Tab::Agents => &mut self.agent_state,
        };
        let cur = state.selected().unwrap_or(0);
        let next = if down {
            (cur + 1).min(len - 1)
        } else {
            cur.saturating_sub(1)
        };
        state.select(Some(next));
    }

    /// The skill currently highlighted, if any.
    fn current_skill(&self) -> Option<&DiscoveredSkill> {
        let visible = self.filtered_skills();
        let pos = self.skill_state.selected()?;
        visible.get(pos).map(|&i| &self.skills[i])
    }

    /// The agent currently highlighted, if any.
    fn current_agent(&self) -> Option<&AgentRow> {
        let visible = self.filtered_agents();
        let pos = self.agent_state.selected()?;
        visible.get(pos).map(|&i| &self.agents[i])
    }

    /// Toggle the mark on the highlighted skill.
    fn toggle_mark(&mut self) {
        if self.tab != Tab::Skills {
            return;
        }
        if let Some(name) = self.current_skill().map(|s| s.name.clone()) {
            if !self.marks.remove(&name) {
                self.marks.insert(name);
            }
        }
    }

    /// The skills to act on: the marked set, or the highlighted one.
    fn action_skills(&self) -> Vec<String> {
        if self.marks.is_empty() {
            self.current_skill()
                .map(|s| vec![s.name.clone()])
                .unwrap_or_default()
        } else {
            self.marks.iter().cloned().collect()
        }
    }
}

/// Run the explore TUI, returning the action chosen on exit.
pub(crate) fn run(ctx: &Context) -> Result<TuiAction, AppError> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut App::new());
    ratatui::restore();
    result.map_err(|e| {
        AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("explore TUI error: {e}"),
            "construct skill find   # use the non-interactive browser instead",
        )
    })
}

/// The draw/event loop.
fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<TuiAction> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.searching {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => app.searching = false,
                KeyCode::Backspace => {
                    app.filter.pop();
                }
                KeyCode::Char(c) => app.filter.push(c),
                _ => {}
            }
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(TuiAction::Quit),
            KeyCode::Char('s') => return Ok(TuiAction::Sync),
            KeyCode::Char('i') => {
                let skills = app.action_skills();
                if !skills.is_empty() {
                    return Ok(TuiAction::Install(skills));
                }
            }
            KeyCode::Tab => {
                app.tab = if app.tab == Tab::Skills {
                    Tab::Agents
                } else {
                    Tab::Skills
                }
            }
            KeyCode::Char('1') => app.tab = Tab::Skills,
            KeyCode::Char('2') => app.tab = Tab::Agents,
            KeyCode::Down | KeyCode::Char('j') => app.move_sel(true),
            KeyCode::Up | KeyCode::Char('k') => app.move_sel(false),
            KeyCode::Char(' ') => app.toggle_mark(),
            KeyCode::Char('/') => {
                app.searching = true;
                app.filter.clear();
            }
            _ => {}
        }
    }
}

// ── rendering ───────────────────────────────────────────────────────────────

fn rgb((r, g, b): (u8, u8, u8)) -> Color {
    Color::Rgb(r, g, b)
}

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_tabs(frame, app, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);
    render_list(frame, app, body[0]);
    render_detail(frame, app, body[1]);

    render_help(frame, app, chunks[2]);
}

fn render_tabs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let titles = vec![
        Line::from(format!(" Skills ({}) ", app.skills.len())),
        Line::from(format!(" Agents ({}) ", app.agents.len())),
    ];
    let selected = match app.tab {
        Tab::Skills => 0,
        Tab::Agents => 1,
    };
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" construct explore ")
                .border_style(Style::default().fg(rgb(theme::STEEL_BLUE))),
        )
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(rgb(theme::MOLTEN_AMBER))
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn render_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let highlight = Style::default()
        .bg(rgb(theme::STEEL_BLUE))
        .fg(rgb(theme::VOID_NAVY))
        .add_modifier(Modifier::BOLD);
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(rgb(theme::STEEL_BLUE)));

    match app.tab {
        Tab::Skills => {
            let items: Vec<ListItem> = app
                .filtered_skills()
                .into_iter()
                .map(|i| {
                    let s = &app.skills[i];
                    let mark = if app.marks.contains(&s.name) {
                        "[x] "
                    } else {
                        "[ ] "
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(mark, Style::default().fg(rgb(theme::RADIUM_GREEN))),
                        Span::raw(s.name.clone()),
                    ]))
                })
                .collect();
            let list = List::new(items)
                .block(border.title(" Skills "))
                .highlight_style(highlight);
            let mut state = app.skill_state.clone();
            frame.render_stateful_widget(list, area, &mut state);
        }
        Tab::Agents => {
            let items: Vec<ListItem> = app
                .filtered_agents()
                .into_iter()
                .map(|i| {
                    let a = &app.agents[i];
                    let flag = if a.hm_managed {
                        Span::styled(" hm", Style::default().fg(rgb(theme::MOLTEN_AMBER)))
                    } else if a.installed {
                        Span::styled(" ✓", Style::default().fg(rgb(theme::RADIUM_GREEN)))
                    } else {
                        Span::raw("")
                    };
                    ListItem::new(Line::from(vec![Span::raw(a.id.clone()), flag]))
                })
                .collect();
            let list = List::new(items)
                .block(border.title(" Agents "))
                .highlight_style(highlight);
            let mut state = app.agent_state.clone();
            frame.render_stateful_widget(list, area, &mut state);
        }
    }
}

fn render_detail(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let border = Block::default()
        .borders(Borders::ALL)
        .title(" Detail ")
        .border_style(Style::default().fg(rgb(theme::STEEL_BLUE)));
    let value = Style::default().fg(rgb(theme::LIQUID_COOLANT));
    let label = Style::default().fg(rgb(theme::MOLTEN_AMBER));

    let lines: Vec<Line> = match app.tab {
        Tab::Skills => match app.current_skill() {
            Some(s) => vec![
                Line::from(vec![
                    Span::styled("skill: ", label),
                    Span::styled(s.name.clone(), value),
                ]),
                Line::from(""),
                Line::from(
                    s.description
                        .clone()
                        .unwrap_or_else(|| "(no description)".to_owned()),
                ),
            ],
            None => vec![Line::from("(no skills — is the Construct clone present?)")],
        },
        Tab::Agents => match app.current_agent() {
            Some(a) => vec![
                Line::from(vec![
                    Span::styled("agent: ", label),
                    Span::styled(a.display_name.clone(), value),
                ]),
                Line::from(vec![
                    Span::styled("id:      ", label),
                    Span::raw(a.id.clone()),
                ]),
                Line::from(vec![
                    Span::styled("project: ", label),
                    Span::raw(a.project_path.clone()),
                ]),
                Line::from(vec![
                    Span::styled("global:  ", label),
                    Span::raw(a.global_path.clone()),
                ]),
                Line::from(vec![
                    Span::styled("status:  ", label),
                    Span::raw(if a.hm_managed {
                        "Home-Manager-managed (global install refused)"
                    } else if a.installed {
                        "detected"
                    } else {
                        "not detected"
                    }),
                ]),
            ],
            None => vec![Line::from("(no agents)")],
        },
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(border)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_help(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let help = if app.searching {
        format!("/{}_   (Enter/Esc to apply)", app.filter)
    } else {
        let marks = if app.marks.is_empty() {
            String::new()
        } else {
            format!("  [{} marked]", app.marks.len())
        };
        format!(
            "Tab/1/2 switch · j/k move · / search · Space mark · i install · s sync · q quit{marks}"
        )
    };
    frame.render_widget(
        Paragraph::new(help).style(Style::default().fg(rgb(theme::STEEL_BLUE))),
        area,
    );
}
