use std::cell::{Cell, RefCell};

use anyhow::{Result, bail};
use classicube_helpers::{
    events::chat::{ChatReceivedEvent, ChatReceivedEventHandler},
    tab_list::remove_color,
};
use classicube_sys::{MsgType_MSG_TYPE_NORMAL, OwnedString, Server, cc_result, cc_string};

thread_local!(
    static CHAT_HANDLER: RefCell<Option<ChatReceivedEventHandler>> = Default::default();
);

thread_local!(
    static ENABLED: Cell<bool> = const { Cell::new(false) };
);

thread_local!(
    static MAP_NAME_TRIGGER: RefCell<Option<String>> = const { RefCell::new(None) };
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
                        println!("saving map: {:?}", map_name);
                        save_map(map_name)
                            .unwrap_or_else(|err| eprintln!("Failed to save map: {}", err));
                    }
                }
            },
        );

        *cell.borrow_mut() = Some(chat_received_event_handler);
    });
}

pub fn free() {
    CHAT_HANDLER.with(|cell| drop(cell.borrow_mut().take()));
}

unsafe extern "C" {
    fn SaveLevelScreen_SaveMap(path: *const cc_string) -> cc_result;
}

fn save_map(name: &str) -> Result<()> {
    if !name.is_ascii()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '_')
    {
        bail!("Invalid map name: {}", name);
    }

    let path = OwnedString::new(format!("maps/{}.cw", name));
    if unsafe { SaveLevelScreen_SaveMap(path.as_cc_string()) } != 0 {
        bail!("SaveLevelScreen_SaveMap failed");
    }

    Ok(())
}
