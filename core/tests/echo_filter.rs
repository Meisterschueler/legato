use legato_core::preprocessing::EchoFilter;

/// FR-030: note-ons of a pitch sent in the last 300 ms are the app's own tones.
#[test]
fn sent_pitches_are_echoes_for_300_ms() {
    let mut filter = EchoFilter::default();
    filter.sent(10.0, 96);
    filter.sent(10.15, 100);

    assert!(filter.is_echo(10.01, 96));
    assert!(filter.is_echo(10.29, 96));
    assert!(!filter.is_echo(10.01, 60), "other pitches are played");
    assert!(filter.is_echo(10.44, 100));
    assert!(!filter.is_echo(10.45, 96), "older than 300 ms");
    assert!(!filter.is_echo(10.5, 100));
}

#[test]
fn nothing_sent_nothing_filtered() {
    let mut filter = EchoFilter::default();
    assert!(!filter.is_echo(0.0, 96));
}
