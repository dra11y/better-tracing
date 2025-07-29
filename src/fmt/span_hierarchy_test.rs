#[cfg(test)]
mod tests {
    use crate::fmt::{format::FmtSpan, FormatEvent, FormatFields, Layer};
    use crate::layer::SubscriberExt;
    use crate::registry::Registry;
    use std::fmt;
    use std::sync::{Arc, Mutex};
    use tracing::info_span;

    // Custom writer to capture output
    #[derive(Clone, Default)]
    struct TestWriter {
        output: Arc<Mutex<Vec<String>>>,
    }

    impl TestWriter {
        fn new() -> Self {
            Self {
                output: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_output(&self) -> Vec<String> {
            self.output.lock().unwrap().clone()
        }
    }

    impl std::io::Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let s = String::from_utf8_lossy(buf);
            self.output.lock().unwrap().push(s.to_string());
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl crate::fmt::MakeWriter<'_> for TestWriter {
        type Writer = Self;

        fn make_writer(&self) -> Self::Writer {
            self.clone()
        }
    }

    // External user's custom formatter - like init_tracing.rs
    struct ExternalCustomFormatter;

    impl ExternalCustomFormatter {
        fn build_hierarchy<S, N>(ctx: &crate::fmt::FmtContext<'_, S, N>) -> String
        where
            S: tracing::Subscriber + for<'a> crate::registry::LookupSpan<'a>,
            N: for<'a> FormatFields<'a> + 'static,
        {
            if let Some(current_span) = ctx.lookup_current() {
                let mut hierarchy = Vec::new();
                let mut span_ref = Some(current_span);
                while let Some(span) = span_ref {
                    hierarchy.push(span.name());
                    span_ref = span.parent();
                }
                hierarchy.reverse();
                hierarchy.join(":")
            } else {
                "NO_CURRENT_SPAN".to_string()
            }
        }
    }

    impl<S, N> FormatEvent<S, N> for ExternalCustomFormatter
    where
        S: tracing::Subscriber + for<'a> crate::registry::LookupSpan<'a>,
        N: for<'a> FormatFields<'a> + 'static,
    {
        fn format_event(
            &self,
            ctx: &crate::fmt::FmtContext<'_, S, N>,
            mut writer: crate::fmt::format::Writer<'_>,
            event: &tracing::Event<'_>,
        ) -> fmt::Result {
            let is_exit_event = event.metadata().target().contains("exit");
            let hierarchy = Self::build_hierarchy(ctx);

            if is_exit_event {
                writeln!(writer, "EXIT_EVENT: current_span={} hierarchy={} [BUG: Cannot access exiting span context!]",
                    ctx.lookup_current().map(|s| s.name()).unwrap_or("NONE"),
                    hierarchy)?;
            } else {
                write!(writer, "REGULAR_EVENT: hierarchy={} ", hierarchy)?;
                ctx.format_fields(writer.by_ref(), event)?;
                writeln!(writer)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_custom_formatter_exit_full_hierarchy() {
        let writer = TestWriter::new();

        let subscriber = Registry::default().with(
            Layer::default()
                .with_writer(writer.clone())
                .with_span_events(FmtSpan::EXIT)
                .event_format(ExternalCustomFormatter),
        );

        simulate_hierarchy(subscriber);

        let output_lines = writer.get_output();

        // Regular events should show full hierarchy
        assert!(
            output_lines.iter().any(|line| line.contains(
                "REGULAR_EVENT: hierarchy=parent_function:child_operation Inside child operation"
            )),
            "Regular events should show full hierarchy, got: {:#?}",
            output_lines
        );

        let exit_lines = output_lines
            .iter()
            .filter(|line| line.contains("REGULAR_EVENT") && line.contains("exit"))
            .cloned()
            .collect::<Vec<_>>();

        // Exit events should show full hierarchy
        assert!(
            exit_lines
                .iter()
                .all(|line| line.contains("hierarchy=parent_function:child_operation")),
            "External formatter should show full hierarchy in exit events, got: {:#?}",
            exit_lines
        );
    }

    #[test]
    fn test_builtin_formatter_exit_full_hierarchy() {
        let writer = TestWriter::new();

        let subscriber = Registry::default().with(
            Layer::default()
                .with_writer(writer.clone())
                .with_span_events(FmtSpan::EXIT),
            // Using built-in formatter
        );

        simulate_hierarchy(subscriber);

        let output_lines = writer.get_output();

        let exit_lines = output_lines
            .iter()
            .filter(|line| line.contains("REGULAR_EVENT") && line.contains("exit"))
            .cloned()
            .collect::<Vec<_>>();

        // Exit events should show full hierarchy
        assert!(
            exit_lines
                .iter()
                .all(|line| line.contains("parent_function") && line.contains("child_operation")),
            "External formatter should show full hierarchy in exit events, got: {:#?}",
            exit_lines
        );
    }

    fn simulate_hierarchy<F, N>(
        subscriber: crate::layer::Layered<Layer<Registry, N, F, TestWriter>, Registry>,
    ) where
        F: for<'writer> FormatEvent<Registry, N> + Send + Sync + 'static,
        N: for<'writer> FormatFields<'writer> + Send + Sync + 'static,
    {
        tracing::subscriber::with_default(subscriber, || {
            let parent_span = info_span!("parent_function", user_id = 123);
            let _parent_guard = parent_span.enter();

            tracing::info!("Inside parent function");

            let child_span = info_span!("child_operation", task_id = 456);
            let _child_guard = child_span.enter();

            tracing::info!("Inside child operation");
        });
    }
}
