use std::io::Cursor;

use anyhow::Context;
use image::io::Reader as ImageReader;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

fn build_menu() -> Menu {
    let menu = Menu::new();

    let quit_item = MenuItem::new("Quit", true, None);
    menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("scut".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_item,
    ])?;

    menu
}

pub fn run_tray() -> anyhow::Result<()> {
    let icon = load_icon()?;

    let event_loop = EventLoopBuilder::new().build();

    let tray_menu = build_menu();

    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("scut - strategic command utility tool")
            .with_icon(icon)
            .build()?,
    );

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            }
            println!("{event:?}");
        }

        if let Ok(event) = tray_channel.try_recv() {
            println!("{event:?}");
        }
    })
}

fn load_icon() -> anyhow::Result<tray_icon::Icon> {
    let (rgba, width, height) = {
        let image = ImageReader::with_format(
            Cursor::new(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../media/icon.png"
            ))),
            image::ImageFormat::Png,
        )
        .decode()?
        .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    tray_icon::Icon::from_rgba(rgba, width, height).context("Failed to open icon")
}
