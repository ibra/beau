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
    #[allow(dead_code)] // shown in the detail panel (milestone 3)
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

    pub fn empty_title(self) -> &'static str {
        match self {
            Self::MyIssues => "Nothing assigned to you",
            Self::Active => "No active tasks",
            Self::Backlog => "Backlog is clear",
            Self::Done => "Nothing finished yet",
        }
    }

    pub fn empty_hint(self) -> &'static str {
        match self {
            Self::MyIssues => "Tasks assigned to you will show up here.",
            Self::Active => "Tasks you start or plan next will show up here.",
            Self::Backlog => "Ideas you park for later will collect here.",
            Self::Done => "Completed tasks will show up here.",
        }
    }

    pub fn matches(self, task: &Task) -> bool {
        match self {
            Self::MyIssues => task.assignee == Some("IH") && task.status != Status::Done,
            Self::Active => matches!(task.status, Status::Todo | Status::InProgress),
            Self::Backlog => task.status == Status::Backlog,
            Self::Done => task.status == Status::Done,
        }
    }
}

pub fn projects() -> Vec<Project> {
    vec![
        Project {
            name: "Beau Core",
            color: Color32::from_rgb(0x3F, 0xA7, 0x96),
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
            47,
            "Focus trap and restore for dialogs",
            S::InProgress,
            P::High,
            0,
            Some("Nov 12"),
            Some("AL"),
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
            46,
            "Publish the web demo",
            S::Todo,
            P::High,
            3,
            Some("Nov 28"),
            Some("IH"),
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
            31,
            "Select with typeahead and keyboard support",
            S::Todo,
            P::Medium,
            0,
            Some("Nov 18"),
            Some("AL"),
        ),
        t(
            34,
            "Bundle size budget for the web build",
            S::Todo,
            P::Medium,
            1,
            None,
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
            42,
            "Contrast report for every theme in CI",
            S::Todo,
            P::Low,
            0,
            Some("Nov 26"),
            None,
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
            "Light and dark palettes",
            S::Done,
            P::High,
            0,
            None,
            Some("IH"),
        ),
    ]
}
