use std::{
    io::Cursor,
    thread::{self, sleep},
    time::Duration,
};

use anyhow::Context;
use image::io::Reader as ImageReader;
// use scut_core::{
//     interface::{LocalStorage, Predict, RemoteStorage, UserInteraction},
//     Config,
// };
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    ClickType, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

/// I'm trust there's a good reason the Menu can't own it's items
///
/// We'll use this outside the event loop to create and own MenuItems, we add
/// references to MenuItems in here to the Menu
#[derive(Default)]
struct DynamicMenuItemStore {
    menu_items: Vec<MenuItem>,
    menu: Option<Menu>,
    tray: Option<TrayIcon>,
}

impl DynamicMenuItemStore {
    fn new() -> Self {
        DynamicMenuItemStore {
            menu_items: Vec::new(),
            menu: None,
            tray: None,
        }
    }

    fn len(&self) -> usize {
        self.menu_items.len()
    }

    /// adds the MenuItem to the end of the store
    ///
    /// panics if no menu is installed!
    fn append(&mut self, item: MenuItem) {
        // TODO: register an event handler somehow that we can hook up dynamically when adding the menu item
        // we have to match the &MenuId from the MenuEvent to the event handler, all dynamically!
        self.menu_items.push(item);
        let item = self.menu_items.last().expect("failed to add menu item");
        let menu = self.menu.as_ref().expect("Menu not installed!");
        menu.append(item).expect("Failed to add menu item");
        self.update_tray(menu);
    }

    /// removes the last MenuItem, and returns whether there was one to remove
    fn pop(&mut self) -> bool {
        if let Some(item) = self.menu_items.pop() {
            let menu = self.menu.as_ref().expect("Menu not installed!");
            menu.remove(&item).expect("Failed to remove menu item");
            self.update_tray(menu);
            true
        } else {
            false
        }
    }

    /// removes the MenuItem at the given position, and returns whether there was one to remove
    fn remove(&mut self, position: usize) -> bool {
        if position >= self.menu_items.len() {
            return false;
        }
        let item = self.menu_items.remove(position);
        let menu = self.menu.as_ref().expect("Menu not installed!");
        menu.remove(&item).expect("Failed to remove menu item");
        self.update_tray(menu);
        true
    }

    /// Installs a tray icon holding the menu. Run this before adding or removing menu items
    fn install_tray(&mut self, tray: TrayIcon) {
        self.tray = Some(tray);
    }

    /// Installs the menu icon holding the menu. Run this before adding or removing menu items
    fn install_menu(&mut self, menu: Menu) {
        self.menu = Some(menu);
    }

    fn update_tray(&self, menu: &Menu) {
        self.tray
            .as_ref()
            .expect("Tray not installed!")
            .set_menu(Some(Box::new(menu.clone())));
    }

    /// Good practive to drop the tray before quitting
    fn drop_tray(&mut self) {
        self.tray.take();
    }
}

pub fn run_tray(// turn_override: Option<u32>,
    // config: &mut Config,
    // mut local: Box<dyn LocalStorage>,
    // mut remote: Box<dyn RemoteStorage>,
    // predictor: Box<dyn Predict>,
    // mut ui: Box<dyn UserInteraction>,
) -> anyhow::Result<()> {
    // let _ = (turn_override, config, local, remote, predictor, ui);
    let icon = load_icon()?;

    let event_loop = EventLoopBuilder::new().build();

    let mut dynamic_menu = DynamicMenuItemStore::new();

    let tray_menu = Menu::new();

    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append_items(&[
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

    dynamic_menu.install_menu(tray_menu.clone());

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("scut - strategic command utility tool")
        .with_icon(icon)
        .build()?;

    dynamic_menu.install_tray(tray_icon);

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    let (tx_scut_side, rx_tray_side) = std::sync::mpsc::channel::<u32>();
    let (tx_tray_side, rx_scut_side) = std::sync::mpsc::channel::<u32>();

    thread::spawn(move || loop {
        if let Ok(event) = rx_scut_side.try_recv() {
            if event > 2 {
                println!("that's enough");
            } else {
                sleep(Duration::from_secs(1));
                println!("tray sent: {event:?}");
                tx_scut_side
                    .send(event + 1)
                    .expect("failed to send to tray");
            }
        }
    });

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Ok(event) = menu_channel.try_recv() {
            println!("menu ui event: {event:?}");
            if event.id == quit_item.id() {
                dynamic_menu.drop_tray();
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Ok(event) = tray_channel.try_recv() {
            println!("tray ui event: {event:?}");
            if event.click_type == ClickType::Left {
                while dynamic_menu.pop() {
                    // we can clear all the dynamic menu items!
                    // The static ones aren't there to be removed :)
                }
                tx_tray_side.send(0).expect("failed to send to scut");
            }
        }

        if let Ok(event) = rx_tray_side.try_recv() {
            println!("scut: {event:?}");

            let item = MenuItem::new(format!("Count: {event}"), false, None);
            dynamic_menu.append(item);

            tx_tray_side.send(event).expect("failed to send to scut");
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
