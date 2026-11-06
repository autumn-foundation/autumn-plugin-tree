//! The widget pack and the plugin.

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::plugin::Plugin;
use autumn_web::widget_pack::WidgetPack;

/// The pack name. Classes start with `autumn-tree`.
pub const PACK_NAME: &str = "tree";

/// The pack CSS and JS.
pub static ASSETS: PluginAssets = PluginAssets::from_files(PACK_NAME, &[]);

/// The pack: its stories and its assets.
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new(PACK_NAME)
}

/// The plugin. It installs [`pack`].
#[derive(Debug, Default, Clone, Copy)]
pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(self, app: AppBuilder) -> AppBuilder {
        app
    }
}
