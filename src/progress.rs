use std::{
    borrow::Cow,
    sync::{Arc, Weak},
    time::Duration,
};

use crate::config::Config;
use parking_lot::Mutex;

const KNOWN_TOTAL_TEMPLATE: &str =
    "{prefix}{spinner:.green} [{bar:40.cyan/blue}] {binary_bytes}/{binary_total_bytes} {binary_bytes_per_sec} ({eta}) {wide_msg}";
const UNKNOWN_TOTAL_TEMPLATE: &str =
    "{prefix}{spinner:.green} {binary_bytes} {binary_bytes_per_sec} {wide_msg}";
const ONLY_MESSAGE_TEMPLATE: &str = "{prefix}{spinner:.green} {wide_msg}";

const PREFIX_EMPTY: &str = "    ";
const PREFIX_NORMAL: &str = "│   ";
const PREFIX_MIDDLE: &str = "├── ";
const PREFIX_LAST: &str = "└── ";

struct BarState {
    level: u32,
    parent: Weak<ProgressBar>,
    children: Vec<Arc<ProgressBar>>,
    is_last_child: bool,
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
    fn new(parent: Option<&Arc<ProgressBar>>) -> Self {
        let inner = indicatif::ProgressBar::new_spinner();
        Self {
            inner,
            state: Mutex::new(BarState::new(parent)),
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

    fn refresh_prefix(self: &Arc<Self>) {
        let mut prefix = vec![];

        let mut now = self.clone();
        while let Some(parent) = {
            let parent = now.state.lock().parent.upgrade().clone();
            parent
        } {
            let is_last = now.state.lock().is_last_child;
            if is_last {
                if prefix.is_empty() {
                    prefix.push(PREFIX_LAST);
                } else {
                    prefix.push(PREFIX_EMPTY);
                }
            } else if prefix.is_empty() {
                prefix.push(PREFIX_MIDDLE);
            } else {
                prefix.push(PREFIX_NORMAL);
            }
            now = parent;
        }

        prefix.reverse();
        self.inner.set_prefix(prefix.concat());
    }

    fn on_last_child(self: &Arc<Self>, last: bool) {
        self.state.lock().is_last_child = last;
        self.refresh_prefix();
    }
}

impl BarState {
    fn new(parent: Option<&Arc<ProgressBar>>) -> Self {
        let level = parent.map_or(0, |p| p.state.lock().level + 1);
        let parent = parent.map_or_else(Weak::new, Arc::downgrade);
        Self {
            level,
            parent,
            children: Vec::new(),
            is_last_child: true,
        }
    }

    fn get_last_child(&self) -> Option<&Arc<ProgressBar>> {
        self.children.last()
    }
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
        let bar = Arc::new(ProgressBar::new(None));
        let style = indicatif::ProgressStyle::default_bar()
            .template(UNKNOWN_TOTAL_TEMPLATE)
            .unwrap();
        self.root.add(bar.inner.clone());
        bar.inner.set_style(style);
        bar.inner.enable_steady_tick(Duration::from_millis(50));
        bar
    }

    pub fn add_msg(&self, parent: Option<&Arc<ProgressBar>>) -> Arc<ProgressBar> {
        let bar = Arc::new(ProgressBar::new(parent));
        if let Some(parent) = parent {
            self.insert_after_last_child(parent, &bar);
        } else {
            self.root.add(bar.inner.clone());
        }
        bar.inner.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(ONLY_MESSAGE_TEMPLATE)
                .unwrap(),
        );
        bar.inner.enable_steady_tick(Duration::from_millis(50));
        bar
    }

    fn insert_after_last_child(&self, parent: &Arc<ProgressBar>, bar: &Arc<ProgressBar>) {
        let previous_bar = {
            let mut parent_state = parent.state.lock();
            let previous = if let Some(last) = parent_state.get_last_child() {
                last.on_last_child(false);
                Some(last.bar().clone())
            } else {
                None
            };
            parent_state.children.push(bar.clone());
            previous
        };
        let previous_bar = previous_bar.unwrap_or_else(|| parent.bar().clone());
        self.root.insert_after(&previous_bar, bar.bar().clone());
        bar.on_last_child(true);
    }

    pub fn add_child_with_total(
        &self,
        parent: &Arc<ProgressBar>,
        total: Option<u64>,
    ) -> Arc<ProgressBar> {
        let style = if total.is_some() {
            indicatif::ProgressStyle::default_bar()
                .template(KNOWN_TOTAL_TEMPLATE)
                .unwrap()
        } else {
            indicatif::ProgressStyle::default_bar()
                .template(UNKNOWN_TOTAL_TEMPLATE)
                .unwrap()
        };
        let bar = Arc::new(ProgressBar::new(Some(parent)));
        self.insert_after_last_child(parent, &bar);

        bar.inner.set_style(style);
        bar.inner.enable_steady_tick(Duration::from_millis(50));

        bar
    }

    pub fn remove_bar(&self, bar: &Arc<ProgressBar>) {
        let state = bar.state.lock();
        let parent = state.parent.upgrade();
        if let Some(parent) = parent {
            let mut parent_state = parent.state.lock();
            let index = parent_state
                .children
                .iter()
                .position(|child| Arc::ptr_eq(child, bar))
                .expect("child not found");
            parent_state.children.remove(index);
        }
        for child in state.children.iter() {
            self.remove_bar(child);
        }
        self.root.remove(&bar.inner);
    }
}
