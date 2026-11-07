//! Tab data structures and state for the False browser.

#[derive(Debug, Clone, PartialEq)]
pub struct DomNodePreview {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub text_content: Option<String>,
    pub children: Vec<DomNodePreview>,
}

#[derive(Debug, Clone)]
pub struct NetworkLogEntry {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub status_text: String,
    pub mime_type: String,
    pub size_bytes: usize,
    pub latency_ms: f64,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub page: Option<String>, // "settings", "history", "bookmarks", etc.
    pub is_pinned: bool,
    pub is_sleeping: bool,
    pub is_loading: bool,
    pub history: Vec<String>,
    pub history_idx: usize,
    pub zoom: f32,
    pub dom_tree: Option<DomNodePreview>,
    pub network_logs: Vec<NetworkLogEntry>,
    pub last_render_time_us: u64,
}

impl Tab {
    pub fn new(id: usize, loc: impl Into<String>) -> Self {
        let loc = loc.into();
        let mut tab = Self {
            id,
            title: "New Tab".to_string(),
            url: String::new(),
            page: None,
            is_pinned: false,
            is_sleeping: false,
            is_loading: false,
            history: Vec::new(),
            history_idx: 0,
            zoom: 1.0,
            dom_tree: None,
            network_logs: Vec::new(),
            last_render_time_us: 110,
        };
        tab.navigate(loc);
        tab
    }

    pub fn navigate(&mut self, loc: String) {
        let trimmed = loc.trim();

        // Check if it's an internal page
        if trimmed == "false://welcome" || trimmed == "false://" || trimmed.is_empty() {
            self.page = None;
            self.url = "false://welcome".to_string();
            self.title = "New Tab".to_string();
        } else if let Some(p) = trimmed.strip_prefix("false://") {
            let p_clean = p.split('/').next().unwrap_or(p).to_lowercase();
            match p_clean.as_str() {
                "settings" => {
                    self.page = Some("settings".to_string());
                    self.url = "false://settings".to_string();
                    self.title = "Settings".to_string();
                }
                "history" => {
                    self.page = Some("history".to_string());
                    self.url = "false://history".to_string();
                    self.title = "History".to_string();
                }
                "bookmarks" => {
                    self.page = Some("bookmarks".to_string());
                    self.url = "false://bookmarks".to_string();
                    self.title = "Bookmarks".to_string();
                }
                _ => {
                    self.page = None;
                    self.url = format!("false://{}", p_clean);
                    self.title = p_clean;
                }
            }
        } else {
            self.page = None;
            self.url = trimmed.to_string();
            self.title = if let Some(domain) = self.url.strip_prefix("https://") {
                domain.split('/').next().unwrap_or(domain).to_string()
            } else if let Some(domain) = self.url.strip_prefix("http://") {
                domain.split('/').next().unwrap_or(domain).to_string()
            } else {
                self.url.clone()
            };
        }

        // Manage navigation history stack
        if self.history.is_empty() || self.history[self.history_idx] != self.url {
            self.history.truncate(self.history_idx + 1);
            self.history.push(self.url.clone());
            self.history_idx = self.history.len() - 1;
        }

        self.populate_devtools();
    }

    pub fn can_go_back(&self) -> bool {
        self.history_idx > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_idx + 1 < self.history.len()
    }

    pub fn go_back(&mut self) {
        if self.can_go_back() {
            self.history_idx -= 1;
            let target = self.history[self.history_idx].clone();
            self.navigate(target);
        }
    }

    pub fn go_forward(&mut self) {
        if self.can_go_forward() {
            self.history_idx += 1;
            let target = self.history[self.history_idx].clone();
            self.navigate(target);
        }
    }

    fn populate_devtools(&mut self) {
        if self.page.is_some() || self.url == "false://welcome" {
            self.dom_tree = Some(DomNodePreview {
                tag: "html".into(),
                attributes: vec![("lang".into(), "en".into()), ("class".into(), "false-app".into())],
                text_content: None,
                children: vec![
                    DomNodePreview {
                        tag: "head".into(),
                        attributes: vec![],
                        text_content: None,
                        children: vec![DomNodePreview {
                            tag: "title".into(),
                            attributes: vec![],
                            text_content: Some(format!("False — {}", self.title)),
                            children: vec![],
                        }],
                    },
                    DomNodePreview {
                        tag: "body".into(),
                        attributes: vec![("class".into(), "monochrome".into())],
                        text_content: None,
                        children: vec![
                            DomNodePreview {
                                tag: "div".into(),
                                attributes: vec![("id".into(), "app-root".into())],
                                text_content: Some(format!("Internal Pure Rust View: {}", self.url)),
                                children: vec![],
                            },
                        ],
                    },
                ],
            });

            self.network_logs = vec![
                NetworkLogEntry {
                    method: "GET".into(),
                    url: self.url.clone(),
                    status: 200,
                    status_text: "OK (Internal)".into(),
                    mime_type: "text/html; charset=utf-8".into(),
                    size_bytes: 4096,
                    latency_ms: 0.05,
                },
            ];
        } else {
            self.dom_tree = Some(DomNodePreview {
                tag: "html".into(),
                attributes: vec![],
                text_content: None,
                children: vec![
                    DomNodePreview {
                        tag: "head".into(),
                        attributes: vec![],
                        text_content: None,
                        children: vec![DomNodePreview {
                            tag: "title".into(),
                            attributes: vec![],
                            text_content: Some(self.title.clone()),
                            children: vec![],
                        }],
                    },
                    DomNodePreview {
                        tag: "body".into(),
                        attributes: vec![],
                        text_content: None,
                        children: vec![
                            DomNodePreview {
                                tag: "main".into(),
                                attributes: vec![("class".into(), "web-document".into())],
                                text_content: Some(format!("Loaded: {}", self.url)),
                                children: vec![],
                            },
                        ],
                    },
                ],
            });

            self.network_logs = vec![
                NetworkLogEntry {
                    method: "GET".into(),
                    url: self.url.clone(),
                    status: 200,
                    status_text: "OK".into(),
                    mime_type: "text/html".into(),
                    size_bytes: 18450,
                    latency_ms: 18.2,
                },
            ];
        }
    }
}
