//! In-memory task data for the tracker.

use eframe::egui::Color32;

/// Workflow status of a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Backlog,
    Todo,
    InProgress,
    Done,
}

impl Status {
    pub const ALL: [Self; 4] = [Self::InProgress, Self::Todo, Self::Backlog, Self::Done];

    pub fn label(self) -> &'static str {
        match self {
            Self::Backlog => "Backlog",
            Self::Todo => "Todo",
            Self::InProgress => "In Progress",
            Self::Done => "Done",
        }
    }
}

/// Task priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    None,
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub number: u32,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub project: usize,
    pub due: Option<&'static str>,
    pub assignee: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub name: &'static str,
    pub color: Color32,
}

/// Sidebar filters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    MyIssues,
    Active,
    Backlog,
    Done,
}

impl Filter {
    pub const ALL: [Self; 4] = [Self::MyIssues, Self::Active, Self::Backlog, Self::Done];

    pub fn label(self) -> &'static str {
        match self {
            Self::MyIssues => "My issues",
            Self::Active => "Active",
            Self::Backlog => "Backlog",
            Self::Done => "Done",
        }
    }
}

pub fn projects() -> Vec<Project> {
    vec![
        Project {
            name: "Beau Core",
            color: Color32::from_rgb(0x9B, 0xC4, 0x1F),
        },
        Project {
            name: "Docs site",
            color: Color32::from_rgb(0x5E, 0xA8, 0xE8),
        },
        Project {
            name: "Tracker",
            color: Color32::from_rgb(0x9B, 0x8A, 0xFB),
        },
        Project {
            name: "Launch",
            color: Color32::from_rgb(0xF0, 0x9A, 0x3E),
        },
    ]
}

pub fn seed_tasks() -> Vec<Task> {
    use Priority as P;
    use Status as S;
    let t = |number, title: &str, status, priority, project, due, assignee| Task {
        number,
        title: title.to_owned(),
        description: String::new(),
        status,
        priority,
        project,
        due,
        assignee,
    };
    vec![
        t(
            41,
            "Presence system: animate out after removal",
            S::InProgress,
            P::Urgent,
            0,
            Some("Nov 8"),
            Some("IH"),
        ),
        t(
            38,
            "Layered shadow tokens for overlay and modal",
            S::InProgress,
            P::High,
            0,
            Some("Nov 9"),
            Some("IH"),
        ),
        t(
            44,
            "Dialog open/close animation with backdrop",
            S::InProgress,
            P::Medium,
            0,
            None,
            Some("MK"),
        ),
        t(
            29,
            "Command palette fuzzy matcher",
            S::Todo,
            P::High,
            2,
            Some("Nov 14"),
            Some("AL"),
        ),
        t(
            33,
            "Detail panel slides in from the right",
            S::Todo,
            P::Medium,
            2,
            Some("Nov 15"),
            None,
        ),
        t(
            36,
            "Live component demos compiled to WASM",
            S::Todo,
            P::Medium,
            1,
            Some("Nov 20"),
            Some("MK"),
        ),
        t(
            39,
            "Keyboard navigation through the task list",
            S::Todo,
            P::Low,
            2,
            None,
            Some("IH"),
        ),
        t(
            45,
            "Hero screenshot and GIF for the README",
            S::Todo,
            P::Low,
            3,
            Some("Dec 1"),
            None,
        ),
        t(
            21,
            "Backdrop blur render pass",
            S::Backlog,
            P::None,
            0,
            None,
            None,
        ),
        t(
            24,
            "Music player showcase app",
            S::Backlog,
            P::Low,
            3,
            None,
            None,
        ),
        t(
            27,
            "Theming API: accent, radii, density",
            S::Backlog,
            P::Medium,
            0,
            None,
            Some("AL"),
        ),
        t(
            12,
            "Bundle Inter variable font",
            S::Done,
            P::High,
            0,
            None,
            Some("IH"),
        ),
        t(
            15,
            "Graphite palette, light and dark",
            S::Done,
            P::High,
            0,
            None,
            Some("IH"),
        ),
    ]
}
