#![allow(non_snake_case)]
//! Configeration for how to serve a Dioxus application

#[cfg(feature = "router")]
use crate::router::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use dioxus::prelude::*;

/// A ServeConfig is used to configure how to serve a Dioxus application. It contains information about how to serve static assets, and what content to render with [`dioxus-ssr`].
#[derive(Clone)]
pub struct ServeConfigBuilder<P: Clone> {
    pub(crate) app: Component<P>,
    pub(crate) props: P,
    pub(crate) root_id: Option<&'static str>,
    pub(crate) index_path: Option<&'static str>,
    pub(crate) assets_path: Option<&'static str>,
    pub(crate) incremental:
        Option<std::sync::Arc<dioxus_ssr::incremental::IncrementalRendererConfig>>,
}

/// A template for incremental rendering that does nothing.
#[derive(Default, Clone)]
pub struct EmptyIncrementalRenderTemplate;

impl dioxus_ssr::incremental::WrapBody for EmptyIncrementalRenderTemplate {
    fn render_after_body<R: std::io::Write>(
        &self,
        _: &mut R,
    ) -> Result<(), dioxus_ssr::incremental::IncrementalRendererError> {
        Ok(())
    }

    fn render_before_body<R: std::io::Write>(
        &self,
        _: &mut R,
    ) -> Result<(), dioxus_ssr::incremental::IncrementalRendererError> {
        Ok(())
    }
}

#[cfg(feature = "router")]
impl<R> ServeConfigBuilder<FullstackRouterConfig<R>>
where
    R: dioxus_router::prelude::Routable,
    <R as std::str::FromStr>::Err: std::fmt::Display,
{
    /// Create a new ServeConfigBuilder to serve a router on the server.
    pub fn new_with_router(cfg: FullstackRouterConfig<R>) -> Self {
        Self::new(RouteWithCfg::<R>, cfg)
    }
}

impl<P: Clone> ServeConfigBuilder<P> {
    /// Create a new ServeConfigBuilder with the root component and props to render on the server.
    pub fn new(app: Component<P>, props: P) -> Self {
        Self {
            app,
            props,
            root_id: None,
            index_path: None,
            assets_path: None,
            incremental: None,
        }
    }

    /// Enable incremental static generation
    pub fn incremental(mut self, cfg: dioxus_ssr::incremental::IncrementalRendererConfig) -> Self {
        self.incremental = Some(std::sync::Arc::new(cfg));
        self
    }

    /// Set the path of the index.html file to be served. (defaults to {assets_path}/index.html)
    pub fn index_path(mut self, index_path: &'static str) -> Self {
        self.index_path = Some(index_path);
        self
    }

    /// Set the id of the root element in the index.html file to place the prerendered content into. (defaults to main)
    pub fn root_id(mut self, root_id: &'static str) -> Self {
        self.root_id = Some(root_id);
        self
    }

    /// Set the path of the assets folder generated by the Dioxus CLI. (defaults to dist)
    pub fn assets_path(mut self, assets_path: &'static str) -> Self {
        self.assets_path = Some(assets_path);
        self
    }

    /// Build the ServeConfig
    pub fn build(self) -> ServeConfig<P> {
        let assets_path = self.assets_path.unwrap_or("dist");

        let index_path = self
            .index_path
            .map(PathBuf::from)
            .unwrap_or_else(|| format!("{assets_path}/index.html").into());

        let root_id = self.root_id.unwrap_or("main");

        let index = load_index_html(index_path, root_id);

        ServeConfig {
            app: self.app,
            props: self.props,
            index,
            assets_path,
            incremental: self.incremental,
        }
    }
}

fn load_index_html(path: PathBuf, root_id: &'static str) -> IndexHtml {
    let mut file = File::open(path).expect("Failed to find index.html. Make sure the index_path is set correctly and the WASM application has been built.");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read index.html");

    let (pre_main, post_main) = contents.split_once(&format!("id=\"{root_id}\"")).unwrap_or_else(|| panic!("Failed to find id=\"{root_id}\" in index.html. The id is used to inject the application into the page."));

    let post_main = post_main.split_once('>').unwrap_or_else(|| {
        panic!("Failed to find closing > after id=\"{root_id}\" in index.html.")
    });

    let (pre_main, post_main) = (
        pre_main.to_string() + &format!("id=\"{root_id}\"") + post_main.0 + ">",
        post_main.1.to_string(),
    );

    IndexHtml {
        pre_main,
        post_main,
    }
}

#[derive(Clone)]
pub(crate) struct IndexHtml {
    pub(crate) pre_main: String,
    pub(crate) post_main: String,
}

/// Used to configure how to serve a Dioxus application. It contains information about how to serve static assets, and what content to render with [`dioxus-ssr`].
/// See [`ServeConfigBuilder`] to create a ServeConfig
#[derive(Clone)]
pub struct ServeConfig<P: Clone> {
    pub(crate) app: Component<P>,
    pub(crate) props: P,
    pub(crate) index: IndexHtml,
    /// The assets path.
    pub assets_path: &'static str,
    pub(crate) incremental:
        Option<std::sync::Arc<dioxus_ssr::incremental::IncrementalRendererConfig>>,
}

impl<P: Clone> From<ServeConfigBuilder<P>> for ServeConfig<P> {
    fn from(builder: ServeConfigBuilder<P>) -> Self {
        builder.build()
    }
}
