use gpui::{
    App, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point, prelude::*, px, size,
};
use local::{DEFAULT_DATABASE_BUSY_TIMEOUT, ZygoLocalConfig, ZygoLocalService};
use zygo_core::ZygoConfig;

mod dependencies;
mod features;
mod navigation;
mod root;
mod stores;
mod theme;
mod ui;

pub use navigation::Routes;

use crate::root::ZygoDesktop;

fn open_window(cx: &mut App) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: Some(point(px(14.0), px(10.0))),
            }),
            ..Default::default()
        },
        |_, cx| cx.new(ZygoDesktop::new),
    )?;

    Ok(())
}

fn main() {
    gpui_platform::application()
        .with_assets(gpuikit::assets())
        .run(|cx: &mut App| {
            gpuikit::init(cx);
            cx.set_global(theme::Theme::dark());
            cx.set_global(dependencies::AppStartup::default());
            open_window(cx).expect("failed to open Zygo desktop window");
            cx.activate(true);

            let task = cx.spawn(async move |cx| {
                let service = cx
                    .background_spawn(async {
                        ZygoLocalService::new(ZygoLocalConfig {
                            base: ZygoConfig::new(1),
                            database_busy_timeout: DEFAULT_DATABASE_BUSY_TIMEOUT,
                        })
                        .await
                    })
                    .await;

                cx.update(|cx| {
                    match service {
                        Ok(service) => {
                            let dependencies = dependencies::AppDeps::new(service, cx);
                            cx.set_global(dependencies);
                        }
                        Err(error) => {
                            cx.global_mut::<dependencies::AppStartup>()
                                .set_error(format!("Unable to start the local service: {error}"));
                        }
                    }
                    cx.refresh_windows();
                });

                Ok::<(), anyhow::Error>(())
            });
            task.detach_and_log_err(cx);
        });
}
