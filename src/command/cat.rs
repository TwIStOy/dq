use std::io::Cursor;

use clap::Args;
use html2text::{self, render::text_renderer::RichAnnotation};

use crate::{context::Context, entity::Docset};

use super::Command;

#[derive(Args, Clone, Debug)]
pub struct CatArgs {
    /// The docset to display.
    slug: String,
    /// The doc page to display.
    path: String,
    /// Do not try to update the docset if the expected page is not found.
    #[arg(short, long, default_value = "false")]
    no_update: bool,
    /// Max width of the output.
    #[arg(short, long, default_value = "160")]
    width: usize,
}

fn default_colour_map(annotations: &[RichAnnotation], s: &str) -> String {
    use termion::color::*;
    use RichAnnotation::*;
    let mut have_explicit_colour = true;
    let mut start = Vec::new();
    let mut finish = Vec::new();
    for annotation in annotations.iter() {
        match annotation {
            Default => {}
            Link(_) => {
                start.push(format!("{}", termion::style::Underline));
                finish.push(format!("{}", termion::style::Reset));
            }
            Image(_) => {
                if !have_explicit_colour {
                    start.push(format!("{}", Fg(Blue)));
                    finish.push(format!("{}", Fg(Reset)));
                }
            }
            Emphasis => {
                start.push(format!("{}", termion::style::Bold));
                finish.push(format!("{}", termion::style::Reset));
            }
            Strong => {
                if !have_explicit_colour {
                    start.push(format!("{}", Fg(LightYellow)));
                    finish.push(format!("{}", Fg(Reset)));
                }
            }
            Strikeout => {
                if !have_explicit_colour {
                    start.push(format!("{}", Fg(LightBlack)));
                    finish.push(format!("{}", Fg(Reset)));
                }
            }
            Code => {
                if !have_explicit_colour {
                    start.push(format!("{}", Fg(Blue)));
                    finish.push(format!("{}", Fg(Reset)));
                }
            }
            Preformat(_) => {
                if !have_explicit_colour {
                    start.push(format!("{}", Fg(Blue)));
                    finish.push(format!("{}", Fg(Reset)));
                }
            }
            Colour(c) => {
                start.push(format!("{}", Fg(Rgb(c.r, c.g, c.b))));
                finish.push(format!("{}", Fg(Reset)));
                have_explicit_colour = true;
            }
            BgColour(c) => {
                start.push(format!("{}", Bg(Rgb(c.r, c.g, c.b))));
                finish.push(format!("{}", Bg(Reset)));
            }
            _ => {}
        }
    }
    // Reverse the finish sequences
    finish.reverse();
    let mut result = start.join("");
    result.push_str(s);
    for s in finish {
        result.push_str(&s);
    }
    result
}

#[async_trait::async_trait]
impl Command for CatArgs {
    async fn run(&self, context: &mut Context) -> anyhow::Result<()> {
        let docsets = Docset::try_to_fetch_docsets(context).await?;

        let doc = docsets
            .iter()
            .find(|docset| docset.slug == self.slug)
            .ok_or_else(|| anyhow::anyhow!("docset {} not found", self.slug))?;

        let page_path =
            context.build_cache_path(format!("{}/db/{}/_index", doc.base_directory(), self.path));

        let content = tokio::fs::read_to_string(&page_path).await?;

        let config = html2text::config::rich()
            .use_doc_css()
            .max_wrap_width(self.width);
        let ret = config.coloured(Cursor::new(&content), self.width, move |anns, s| {
            default_colour_map(anns, s)
        })?;

        print!("{}", ret);

        Ok(())
    }
}
