use std::{
    borrow::Cow,
    sync::{Arc, Weak},
};

use crate::config::Config;
use parking_lot::Mutex;

const LEVEL_INDENT: u32 = 2;

const KNOWN_TOTAL_TEMPLATE: &str =
    "{prefix}{spinner:.green} [{bar:40.cyan/blue}] {binary_bytes}/{binary_total_bytes} {binary_bytes_per_sec} ({eta}) {wide_msg}";
const UNKNOWN_TOTAL_TEMPLATE: &str =
    "{prefix}{spinner:.green} {binary_bytes} {binary_bytes_per_sec} {wide_msg}";

struct BarState {
    level: u32,
    parent: Weak<ProgressBar>,
    children: Vec<Arc<ProgressBar>>,
}

pub struct ProgressBar {
    inner: indicatif::ProgressBar,
    state: Mutex<BarState>,
}

#[derive(Debug)]
pub struct ProgressBarManager {
    root: indicatif::MultiProgress,
}

impl ProgressBar {
    fn new_root() -> Self {
        let style = indicatif::ProgressStyle::default_bar()
            .template(UNKNOWN_TOTAL_TEMPLATE)
            .unwrap();
        let inner = indicatif::ProgressBar::new_spinner().with_style(style);
        Self {
            inner,
            state: Mutex::new(BarState::new_root()),
        }
    }

    fn new_child(total: Option<u64>, parent: &Arc<ProgressBar>) -> Self {
        let style = if total.is_some() {
            indicatif::ProgressStyle::default_bar()
                .template(KNOWN_TOTAL_TEMPLATE)
                .unwrap()
                .progress_chars("##-")
        } else {
            indicatif::ProgressStyle::default_bar()
                .template(UNKNOWN_TOTAL_TEMPLATE)
                .unwrap()
        };
        let inner = indicatif::ProgressBar::new(total.unwrap_or(0)).with_style(style);
        Self {
            inner,
            state: Mutex::new(BarState::new_child(parent)),
        }
    }

    fn bar(&self) -> &indicatif::ProgressBar {
        &self.inner
    }

    fn switch_to_unknown_template(&self) {
        self.inner.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(UNKNOWN_TOTAL_TEMPLATE)
                .unwrap(),
        );
    }

    fn switch_to_known_template(&self, total: u64) {
        self.inner.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(KNOWN_TOTAL_TEMPLATE)
                .unwrap(),
        );
        self.inner.set_length(total);
    }

    pub fn set_position(&self, pos: u64) {
        self.inner.set_position(pos);
    }

    pub fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.inner.set_message(msg);
    }

    pub fn finish(&self, msg: impl Into<Cow<'static, str>>) {
        self.inner.finish_with_message(msg);
    }

    pub fn update_template(&self, total: Option<u64>) {
        if let Some(total) = total {
            self.switch_to_known_template(total);
        } else {
            self.switch_to_unknown_template();
        }
    }

    pub fn inc(&self, n: u64) {
        self.inner.inc(n);
    }
}

impl BarState {
    fn new_root() -> Self {
        Self {
            level: 0,
            parent: Weak::new(),
            children: Vec::new(),
        }
    }

    fn new_child(parent: &Arc<ProgressBar>) -> Self {
        let level = parent.state.lock().level + 1;
        let parent = Arc::downgrade(parent);
        Self {
            level,
            parent,
            children: Vec::new(),
        }
    }

    fn get_last_child(&self) -> Option<&Arc<ProgressBar>> {
        self.children.last()
    }

    fn on_last_child(&self, last: bool) {}
}

impl ProgressBarManager {
    pub fn new(config: &Config) -> Self {
        let root = if config.progress() {
            indicatif::MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::stderr())
        } else {
            indicatif::MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::hidden())
        };
        Self { root }
    }

    pub fn add_root(&self) -> Arc<ProgressBar> {
        let bar = Arc::new(ProgressBar::new_root());
        self.root.add(bar.inner.clone());
        bar
    }

    pub fn add_child_with_total(&self, parent: &Arc<ProgressBar>, total: u64) -> Arc<ProgressBar> {
        let bar = Arc::new(ProgressBar::new_child(Some(total), parent));
        {
            bar.state.lock().on_last_child(true);
        }

        let previous_bar = {
            let mut parent_state = parent.state.lock();
            parent_state.children.push(bar.clone());
            if let Some(last) = parent_state.get_last_child() {
                last.state.lock().on_last_child(false);
                Some(last.bar().clone())
            } else {
                None
            }
        };

        let previous_bar = previous_bar.unwrap_or_else(|| parent.bar().clone());
        self.root.insert_after(&previous_bar, bar.bar().clone());
        bar
    }
}
