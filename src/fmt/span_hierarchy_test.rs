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
            let hierarchy = ctx
                .scope()
                .map(|scope| {
                    // Collect spans and reverse for root-to-leaf order
                    let spans: Vec<_> = scope.collect();
                    spans.iter().rev().map(|span| span.name()).collect::<Vec<_>>().join(":")
                })
                .unwrap_or_else(|| "NO_CURRENT_SPAN".to_string());

            if is_exit_event {
                writeln!(
                    writer,
                    "EXIT_EVENT: current_span={} hierarchy={}",
                    ctx.lookup_current().map(|s| s.name()).unwrap_or("NONE"),
                    hierarchy
                )?;
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

        // Should have exactly 2 exit events
        assert_eq!(
            exit_lines.len(),
            2,
            "Expected 2 exit events, got: {:#?}",
            exit_lines
        );

        // First exit event: child_operation should show full hierarchy
        assert!(
            exit_lines[0].contains("hierarchy=parent_function:child_operation"),
            "Child exit should show full hierarchy, got: {}",
            exit_lines[0]
        );

        // Second exit event: parent_function should show only itself
        assert!(
            exit_lines[1].contains("hierarchy=parent_function")
                && !exit_lines[1].contains("child_operation"),
            "Parent exit should show only parent span, got: {}",
            exit_lines[1]
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

    #[test]
    fn test_custom_formatter_close_full_hierarchy() {
        let writer = TestWriter::new();

        let subscriber = Registry::default().with(
            Layer::default()
                .with_writer(writer.clone())
                .with_span_events(FmtSpan::CLOSE)
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

        let close_lines = output_lines
            .iter()
            .filter(|line| line.contains("REGULAR_EVENT") && line.contains("close"))
            .cloned()
            .collect::<Vec<_>>();

        // Should have exactly 2 close events
        assert_eq!(
            close_lines.len(),
            2,
            "Expected 2 close events, got: {:#?}",
            close_lines
        );

        // First close event: child_operation should show full hierarchy
        assert!(
            close_lines[0].contains("hierarchy=parent_function:child_operation"),
            "Child close should show full hierarchy, got: {}",
            close_lines[0]
        );

        // Second close event: parent_function should show only itself
        assert!(
            close_lines[1].contains("hierarchy=parent_function")
                && !close_lines[1].contains("child_operation"),
            "Parent close should show only parent span, got: {}",
            close_lines[1]
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
