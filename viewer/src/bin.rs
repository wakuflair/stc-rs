use stc::prelude::*;

use std::fmt::Write;

mod app;
#[cfg(feature = "gui-egui")]
mod egui;

#[cfg(feature = "gui-egui")]
fn main() -> Result<(), eframe::Error> {
    egui::main()
}

pub(crate) trait PrototypeDisplayName {
    fn display_name(&self) -> String;
}

impl PrototypeDisplayName for Prototype {
    fn display_name(&self) -> String {
        let proto = self.read().unwrap();
        let name = proto.name();

        if name.is_empty() {
            format!("{} - {} (No Name)", proto.decl().kind(), proto.id())
        } else {
            name.string().to_owned()
        }
    }
}

pub(crate) trait PrototypeContent {
    fn content(&self) -> String;
}

impl PrototypeContent for Prototype {
    fn content(&self) -> String {
        let proto = self.read().unwrap();
        let mut buf = String::with_capacity(1024 * 4);

        // Prototype properties
        let uuid = proto.object_id();
        if !uuid.is_nil() {
            writeln!(buf, "ObjectId: {}", uuid).unwrap();
        }
        writeln!(buf, "Id: {}", proto.id()).unwrap();
        writeln!(buf).unwrap();

        // name : return_value
        write!(buf, "{}", proto.name()).unwrap();
        if let Some(ty) = proto.return_value().and_then(|x| x.ty()) {
            write!(buf, " : {}", ty).unwrap();
        }
        writeln!(buf).unwrap();

        for v in proto.variables() {
            writeln!(buf, "{}: {}: {}", v.flags(), v.name(), v.ty().unwrap()).unwrap();
        }

        buf
    }
}
