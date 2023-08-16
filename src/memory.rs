use anyhow::Result;
use byte_unit::Byte;
use cnx::text::{Attributes, Text};
use cnx::widgets::{Widget, WidgetStream};
use std::time::Duration;
use sysinfo::{RefreshKind, System, SystemExt};
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

// Abstracted type to represent the render closure
type MemoryRender = Box<dyn Fn(&System) -> String>;

/// cnx widget that shows current system memory usage
pub struct MemoryUsage {
    attrs: Attributes,
    render: Option<MemoryRender>,
    memory_handle: System,
    update_interval: Duration,
}

impl MemoryUsage {
    /// Creates a new  [`MemoryUsage`] widget
    ///
    /// Arguments
    ///
    /// `attrs`: [`Attributes`] - Widget attributes which control font,
    /// foreground and background colour.
    ///
    /// `render`: [`Option<MemoryRender>`] - Optional
    /// parameter to customise the way the widget is rendered. Takes a
    /// closure that returns a String
    #[must_use]
    pub fn new(attrs: Attributes, render: Option<MemoryRender>) -> MemoryUsage {
        let memory_handle = System::new_with_specifics(RefreshKind::new().with_memory());
        MemoryUsage {
            attrs,
            render,
            memory_handle,
            update_interval: Duration::new(10, 0),
        }
    }

    fn tick(&self) -> Vec<Text> {
        let text: String = if self.render.is_none() {
            let used_bytes = Byte::from_bytes(self.memory_handle.free_memory().into());
            let appropriate_used_bytes_unit = used_bytes.get_appropriate_unit(false);
            let total_bytes = Byte::from_bytes(self.memory_handle.total_memory().into());
            let appropriate_total_bytes_unit = total_bytes.get_appropriate_unit(false);

            format!(
                "({used}/{total})",
                used = appropriate_used_bytes_unit.format(1),
                total = appropriate_total_bytes_unit.format(1)
            )
        } else {
            self.render
                .as_ref()
                .map_or(String::new(), |x| (x)(&self.memory_handle))
        };

        vec![Text {
            attr: self.attrs.clone(),
            text,
            stretch: false,
            markup: self.render.is_some(),
        }]
    }
}

impl Widget for MemoryUsage {
    fn into_stream(self: Box<Self>) -> Result<WidgetStream> {
        let interval = time::interval(self.update_interval);
        let stream = IntervalStream::new(interval).map(move |_| Ok(self.tick()));

        Ok(Box::pin(stream))
    }
}
