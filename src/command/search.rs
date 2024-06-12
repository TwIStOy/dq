use clap::{Args, ValueEnum};
use fuzzy_matcher::FuzzyMatcher;
use stanza::{
    renderer::{console::Console, Renderer as _},
    style::{Header, Styles},
    table::{Row, Table},
};

use crate::{
    context::Context,
    entity::{Docset, Index, IndexEntry},
};

use super::Command;

#[derive(ValueEnum, Clone, Debug, Copy)]
enum Matcher {
    SkimMatcherV1,
    SkimMatcherV2,
    Clangd,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
enum OutputFormat {
    Text,
    Json,
    Table,
}

#[derive(Args, Clone, Debug)]
pub struct SearchArgs {
    /// The matcher to use.
    #[arg(long, default_value = "skim-matcher-v2", value_enum)]
    matcher: Matcher,
    /// The docset to search.
    slug: String,
    /// The query to search.
    keyword: String,
    /// The output format.
    #[arg(long, default_value = "text", value_enum)]
    format: OutputFormat,
}

impl Matcher {
    fn to_matcher(self) -> Box<dyn FuzzyMatcher> {
        match self {
            #[allow(deprecated)]
            Matcher::SkimMatcherV1 => Box::new(fuzzy_matcher::skim::SkimMatcher::default()),
            Matcher::SkimMatcherV2 => Box::new(fuzzy_matcher::skim::SkimMatcherV2::default()),
            Matcher::Clangd => Box::new(fuzzy_matcher::clangd::ClangdMatcher::default()),
        }
    }
}

trait Outputs {
    fn output(&self, entries: &[(&IndexEntry, i64)]);
}

struct TextOutput;

impl Outputs for TextOutput {
    fn output(&self, entries: &[(&IndexEntry, i64)]) {
        for (entry, _) in entries {
            println!("{}", entry.path);
        }
    }
}

struct JsonOutput;

impl Outputs for JsonOutput {
    fn output(&self, entries: &[(&IndexEntry, i64)]) {
        let entries = entries
            .iter()
            .map(|(entry, score)| {
                serde_json::json!({
                    "entry": entry,
                    "score": score,
                })
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string(&entries).unwrap());
    }
}

struct TableOutput;

impl Outputs for TableOutput {
    fn output(&self, entries: &[(&IndexEntry, i64)]) {
        // build a table model
        let mut table = Table::default().with_row(Row::new(
            Styles::default().with(Header(true)),
            vec!["Name".into(), "Path".into(), "Score".into()],
        ));
        for (entry, score) in entries {
            table.push_row(vec![
                entry.name.clone(),
                entry.path.clone(),
                format!("{:.2}", score),
            ]);
        }
        let renderer = Console::default();
        println!("{}", renderer.render(&table));
    }
}

impl OutputFormat {
    fn to_output(self) -> Box<dyn Outputs> {
        match self {
            OutputFormat::Text => Box::new(TextOutput),
            OutputFormat::Json => Box::new(JsonOutput),
            OutputFormat::Table => Box::new(TableOutput),
        }
    }
}

#[async_trait::async_trait]
impl Command for SearchArgs {
    async fn run(&self, context: &mut Context) -> anyhow::Result<()> {
        let matcher = self.matcher.to_matcher();
        let docsets = Docset::try_to_fetch_docsets(context).await?;
        let doc = docsets
            .iter()
            .find(|docset| docset.slug == self.slug)
            .ok_or_else(|| anyhow::anyhow!("docset {} not found", self.slug))?;
        let index_file: Index = context
            .read_from_cache(format!("{}/index.json", doc.base_directory()))
            .await?;
        let mut entries = index_file
            .entries
            .iter()
            .filter_map(|entry| {
                matcher
                    .fuzzy_match(&entry.name, &self.keyword)
                    .map(|score| (entry, score))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let outputs = self.format.to_output();
        outputs.output(&entries);

        Ok(())
    }
}
