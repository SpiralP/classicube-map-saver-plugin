use std::cell::{Cell, RefCell};

use classicube_helpers::{
    events::chat::{ChatReceivedEvent, ChatReceivedEventHandler},
    tab_list::remove_color,
    tick::TickEventHandler,
};
use classicube_sys::*;

thread_local!(
    static TICK_HANDLER: RefCell<Option<TickEventHandler>> = Default::default();
);

thread_local!(
    static CHAT_HANDLER: RefCell<Option<ChatReceivedEventHandler>> = Default::default();
);

thread_local!(
    static ENABLED: Cell<bool> = const { Cell::new(false) };
);

thread_local!(
    static MAP_NAME_TRIGGER: RefCell<Option<String>> = const { RefCell::new(None) };
);

thread_local!(
    static MAP_LOADED: Cell<bool> = const { Cell::new(false) };
);

thread_local!(
    static TEXTURE_PACK_DOWNLOADED: Cell<bool> = const { Cell::new(false) };
);

pub fn init() {
    if unsafe { Server.IsSinglePlayer } != 0 {
        return;
    }

    CHAT_HANDLER.with(|cell| {
        let mut chat_received_event_handler = ChatReceivedEventHandler::new();

        chat_received_event_handler.on(
            move |ChatReceivedEvent {
                      message,
                      message_type,
                  }| {
                let message = remove_color(message);

                if *message_type == MsgType_MSG_TYPE_NORMAL {
                    if let Some(map_name) = message.strip_prefix("SpiralP went to ") {
                        MAP_NAME_TRIGGER.set(Some(map_name.to_string()));
                    }
                } else if *message_type == MsgType_MSG_TYPE_EXTRASTATUS_1 {
                    if message.starts_with("Retrieving texture pack")
                        || message.starts_with("Downloading texture pack")
                    {
                        TEXTURE_PACK_DOWNLOADED.set(false);
                    } else if message.is_empty() {
                        TEXTURE_PACK_DOWNLOADED.set(true);
                    }
                }

                println!("!!! {:?} {:?}", message_type, message);
            },
        );

        *cell.borrow_mut() = Some(chat_received_event_handler);
    });

    TICK_HANDLER.with(|cell| {
        let mut tick_event_handler = TickEventHandler::new();

        tick_event_handler.on(move |_event| {
            MAP_NAME_TRIGGER.with_borrow_mut(|map_name_trigger| {
                if let Some(map_name) = map_name_trigger {
                    if MAP_LOADED.get() && TEXTURE_PACK_DOWNLOADED.get() {
                        let text = OwnedString::new(format!("saving {:?}", map_name));
                        unsafe {
                            Chat_Add(text.as_cc_string());
                        }
                        *map_name_trigger = None;
                    }
                }
            });
        });

        *cell.borrow_mut() = Some(tick_event_handler);
    });
}

pub fn free() {
    TICK_HANDLER.with(|cell| drop(cell.borrow_mut().take()));
    CHAT_HANDLER.with(|cell| drop(cell.borrow_mut().take()));
}

pub fn reset() {
    MAP_NAME_TRIGGER.take();
    MAP_LOADED.set(false);
    TEXTURE_PACK_DOWNLOADED.set(false);
}

pub fn on_new_map() {
    MAP_NAME_TRIGGER.take();
    MAP_LOADED.set(false);
    TEXTURE_PACK_DOWNLOADED.set(false);
}

pub fn on_new_map_loaded() {
    MAP_NAME_TRIGGER.take();
    MAP_LOADED.set(true);
    TEXTURE_PACK_DOWNLOADED.set(false);
}
