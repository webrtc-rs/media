use rtp::{header::Header, packet::Packet, packetizer::Depacketizer};

use super::*;

// Turns u8 integers into Bytes Array
macro_rules! bytes {
    ($($item:expr),*) => ({
        static STATIC_SLICE: &'static [u8] = &[$($item), *];
        Bytes::from_static(STATIC_SLICE)
    });
}
#[derive(Default)]
pub struct SampleBuilderTest {
    message: String,
    packets: Vec<Packet>,
    with_head_checker: bool,
    head_bytes: Vec<bytes::Bytes>,
    samples: Vec<Sample>,
    max_late: u16,
    max_late_timestamp: Duration,
}

pub struct FakeDepacketizer {
    head_checker: bool,
    head_bytes: Vec<bytes::Bytes>,
}

impl Depacketizer for FakeDepacketizer {
    fn depacketize(&mut self, b: &Bytes) -> std::result::Result<bytes::Bytes, rtp::Error> {
        Ok(b.clone())
    }

    /// Checks if the packet is at the beginning of a partition.  This
    /// should return false if the result could not be determined, in
    /// which case the caller will detect timestamp discontinuities.
    fn is_partition_head(&self, payload: &Bytes) -> bool {
        if self.head_checker == false {
            // from .go: simulates a bug in 3.0 version, the tests should not assume the bug
            return true;
        }

        for b in self.head_bytes.clone() {
            if *payload == b {
                return true;
            }
        }
        return false;
    }

    /// Checks if the packet is at the end of a partition.  This should
    /// return false if the result could not be determined.
    fn is_partition_tail(&self, marker: bool, _payload: &Bytes) -> bool {
        return marker;
    }
}

// .go uses testing.T as parameter, have to look into that
#[test]
pub fn test_sample_builder() {
    let test_data: Vec<SampleBuilderTest> = vec![
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder shouldn't emit anything if only one RTP packet has been pushed".into(),
            packets: vec![Packet {
                header: Header {
                    sequence_number: 5000,
                    timestamp: 5,
                    ..Default::default()
                },
                payload: bytes!(1u8),
                ..Default::default()
            }],
            samples: vec![],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder shouldn't emit anything if only one RTP packet has been pushed even if the marker bit is set".into(),
            packets: vec![Packet {
                header: Header {
                    sequence_number: 5000,
                    timestamp: 5,
                    marker: true,
                    ..Default::default()
                },
                payload: bytes!(1u8),
                ..Default::default()
            }],
            samples: vec![],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder should emit two packets, we had three packets with unique timestamps".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5001,
                        timestamp: 6,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
            ],
            samples: vec![
                Sample {
                    // First sample
                    data: bytes!(1u8),
                    duration: Duration::from_secs(1), // technically this is the default value, but since it was in .go source....
                    packet_timestamp: 5,
                    ..Default::default()
                },
                Sample {
                    // Second sample
                    data: bytes!(2u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 6,
                    ..Default::default()
                },
            ],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder should emit one packet, we had a packet end of sequence marker and run out of space".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5004,
                        timestamp: 9,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
                Packet {
                    // Fourth packet
                    header: Header {
                        sequence_number: 5006,
                        timestamp: 11,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
                Packet {
                    // Fifth packet
                    header: Header {
                        sequence_number: 5008,
                        timestamp: 13,
                        ..Default::default()
                    },
                    payload: bytes!(5u8),
                    ..Default::default()
                },
                Packet {
                    // Sixth packet
                    header: Header {
                        sequence_number: 5010,
                        timestamp: 15,
                        ..Default::default()
                    },
                    payload: bytes!(6u8),
                    ..Default::default()
                },
                Packet {
                    // Seventh packet
                    header: Header {
                        sequence_number: 5012,
                        timestamp: 17,
                        ..Default::default()
                    },
                    payload: bytes!(7u8),
                    ..Default::default()
                },
            ],
            samples: vec![Sample {
                // First sample
                data: bytes!(1u8),
                duration: Duration::from_secs(2),
                packet_timestamp: 5,
                ..Default::default()
            }],
            max_late: 5,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder shouldn't emit any packet, we do not have a valid end of sequence and run out of space".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5004,
                        timestamp: 9,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
                Packet {
                    // Fourth packet
                    header: Header {
                        sequence_number: 5006,
                        timestamp: 11,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
                Packet {
                    // Fifth packet
                    header: Header {
                        sequence_number: 5008,
                        timestamp: 13,
                        ..Default::default()
                    },
                    payload: bytes!(5u8),
                    ..Default::default()
                },
                Packet {
                    // Sixth packet
                    header: Header {
                        sequence_number: 5010,
                        timestamp: 15,
                        ..Default::default()
                    },
                    payload: bytes!(6u8),
                    ..Default::default()
                },
                Packet {
                    // Seventh packet
                    header: Header {
                        sequence_number: 5012,
                        timestamp: 17,
                        ..Default::default()
                    },
                    payload: bytes!(7u8),
                    ..Default::default()
                },
            ],
            samples: vec![],
            max_late: 5,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder should emit one packet, we had a packet end of sequence marker and run out of space".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 7,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5004,
                        timestamp: 9,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
                Packet {
                    // Fourth packet
                    header: Header {
                        sequence_number: 5006,
                        timestamp: 11,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
                Packet {
                    // Fifth packet
                    header: Header {
                        sequence_number: 5008,
                        timestamp: 13,
                        ..Default::default()
                    },
                    payload: bytes!(5u8),
                    ..Default::default()
                },
                Packet {
                    // Sixth packet
                    header: Header {
                        sequence_number: 5010,
                        timestamp: 15,
                        ..Default::default()
                    },
                    payload: bytes!(6u8),
                    ..Default::default()
                },
                Packet {
                    // Seventh packet
                    header: Header {
                        sequence_number: 5012,
                        timestamp: 17,
                        ..Default::default()
                    },
                    payload: bytes!(7u8),
                    ..Default::default()
                },
            ],
            samples: vec![
                Sample {
                    // First (dropped) sample
                    data: bytes!(1u8),
                    duration: Duration::from_secs(2),
                    packet_timestamp: 5,
                    ..Default::default()
                },
                Sample {
                    // First correct sample
                    data: bytes!(2u8),
                    duration: Duration::from_secs(2),
                    packet_timestamp: 7,
                    prev_dropped_packets: 1,
                    ..Default::default()
                },
            ],
            max_late: 5,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder should emit one packet, we had two packets but with duplicate timestamps".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5001,
                        timestamp: 6,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 6,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
                Packet {
                    // Fourth packet
                    header: Header {
                        sequence_number: 5003,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
            ],
            samples: vec![
                Sample {
                    // First sample
                    data: bytes!(1u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 5,
                    ..Default::default()
                },
                Sample {
                    // Second (duplicate) correct sample
                    data: bytes!(2u8, 2u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 6,
                    ..Default::default()
                },
            ],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder shouldn't emit a packet because we have a gap before a valid one".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5007,
                        timestamp: 6,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5008,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
            ],
            samples: vec![],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder shouldn't emit a packet after a gap as there are gaps and have not reached maxLate yet".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5007,
                        timestamp: 6,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5008,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
            ],
            with_head_checker: true,
            head_bytes: vec![bytes!(2u8)],
            samples: vec![],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder shouldn't emit a packet after a gap if PartitionHeadChecker doesn't assume it head".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 5,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5007,
                        timestamp: 6,
                        marker: true,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5008,
                        timestamp: 7,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
            ],
            with_head_checker: true,
            head_bytes: vec![],
            samples: vec![],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder should emit multiple valid packets".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 1,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5001,
                        timestamp: 2,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 3,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
                Packet {
                    // Fourth packet
                    header: Header {
                        sequence_number: 5003,
                        timestamp: 4,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
                Packet {
                    // Fifth packet
                    header: Header {
                        sequence_number: 5004,
                        timestamp: 5,
                        ..Default::default()
                    },
                    payload: bytes!(5u8),
                    ..Default::default()
                },
                Packet {
                    // Sixth packet
                    header: Header {
                        sequence_number: 5005,
                        timestamp: 6,
                        ..Default::default()
                    },
                    payload: bytes!(6u8),
                    ..Default::default()
                },
            ],
            samples: vec![
                Sample {
                    // First sample
                    data: bytes!(1u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 1,
                    ..Default::default()
                },
                Sample {
                    // Second sample
                    data: bytes!(2u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 2,
                    ..Default::default()
                },
                Sample {
                    // Third sample
                    data: bytes!(3u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 3,
                    ..Default::default()
                },
                Sample {
                    // Fourth sample
                    data: bytes!(4u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 4,
                    ..Default::default()
                },
                Sample {
                    // Fifth sample
                    data: bytes!(5u8),
                    duration: Duration::from_secs(1),
                    packet_timestamp: 5,
                    ..Default::default()
                },
            ],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(0),
            ..Default::default()
        },
        SampleBuilderTest {
            #[rustfmt::skip]
            message: "Sample builder should skipt time stamps too old".into(),
            packets: vec![
                Packet {
                    // First packet
                    header: Header {
                        sequence_number: 5000,
                        timestamp: 1,
                        ..Default::default()
                    },
                    payload: bytes!(1u8),
                    ..Default::default()
                },
                Packet {
                    // Second packet
                    header: Header {
                        sequence_number: 5001,
                        timestamp: 2,
                        ..Default::default()
                    },
                    payload: bytes!(2u8),
                    ..Default::default()
                },
                Packet {
                    // Third packet
                    header: Header {
                        sequence_number: 5002,
                        timestamp: 3,
                        ..Default::default()
                    },
                    payload: bytes!(3u8),
                    ..Default::default()
                },
                Packet {
                    // Fourth packet
                    header: Header {
                        sequence_number: 5013,
                        timestamp: 4000,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
                Packet {
                    // Fifth packet
                    header: Header {
                        sequence_number: 5014,
                        timestamp: 4000,
                        ..Default::default()
                    },
                    payload: bytes!(5u8),
                    ..Default::default()
                },
                Packet {
                    // Sixth packet
                    header: Header {
                        sequence_number: 5015,
                        timestamp: 4002,
                        ..Default::default()
                    },
                    payload: bytes!(6u8),
                    ..Default::default()
                },
                Packet {
                    // Seventh packet
                    header: Header {
                        sequence_number: 5016,
                        timestamp: 7000,
                        ..Default::default()
                    },
                    payload: bytes!(4u8),
                    ..Default::default()
                },
                Packet {
                    // Eigth packet
                    header: Header {
                        sequence_number: 5017,
                        timestamp: 7001,
                        ..Default::default()
                    },
                    payload: bytes!(5u8),
                    ..Default::default()
                },
            ],
            samples: vec![Sample {
                // First sample
                data: bytes!(4u8, 5u8),
                duration: Duration::from_secs(2),
                packet_timestamp: 4000,
                prev_dropped_packets: 13,
                ..Default::default()
            }],
            with_head_checker: true,
            head_bytes: vec![bytes!(4u8)],
            max_late: 50,
            max_late_timestamp: Duration::from_secs(2000),
            ..Default::default()
        },
    ];

    for t in test_data {
        let d = FakeDepacketizer {
            head_checker: t.with_head_checker,
            head_bytes: t.head_bytes,
        };

        let mut s = {
            let sample_builder = SampleBuilder::new(t.max_late, d, 1);
            if t.max_late_timestamp != Duration::from_secs(0) {
                sample_builder.with_max_time_delay(t.max_late_timestamp)
            } else {
                sample_builder
            }
        };

        let mut samples = Vec::<Sample>::new();
        for p in t.packets {
            s.push(p)
        }

        // Here we need some fancy loop that pops from s until empty. This propbably exists somewhere already.
        // HAH, found it.
        while let Some(sample) = s.pop() {
            samples.push(sample)
        }

        // Current problem: Sample does not implement Eq. Either implement myself or find another way of comparison. (Derive does not work)
        assert_eq!(t.samples, samples, "{}", t.message);
    }
}

// SampleBuilder should respect maxLate if we popped successfully but then have a gap larger then maxLate
#[test]
fn test_sample_builder_max_late() {
    let mut s = SampleBuilder::new(
        50,
        FakeDepacketizer {
            head_bytes: vec![],
            head_checker: false,
        },
        1,
    );

    s.push(Packet {
        header: Header {
            sequence_number: 0,
            timestamp: 1,
            ..Default::default()
        },
        payload: bytes!(0x01),
    });
    s.push(Packet {
        header: Header {
            sequence_number: 1,
            timestamp: 2,
            ..Default::default()
        },
        payload: bytes!(0x01),
    });
    s.push(Packet {
        header: Header {
            sequence_number: 2,
            timestamp: 3,
            ..Default::default()
        },
        payload: bytes!(0x01),
    });
    assert_eq!(
        Some(Sample {
            data: bytes!(0x01),
            duration: Duration::from_secs(1),
            packet_timestamp: 1,
            ..Default::default()
        }),
        s.pop(),
        "Failed to build samples before gap"
    );

    s.push(Packet {
        header: Header {
            sequence_number: 5000,
            timestamp: 500,
            ..Default::default()
        },
        payload: bytes!(0x02),
    });
    s.push(Packet {
        header: Header {
            sequence_number: 5001,
            timestamp: 501,
            ..Default::default()
        },
        payload: bytes!(0x02),
    });
    s.push(Packet {
        header: Header {
            sequence_number: 5002,
            timestamp: 502,
            ..Default::default()
        },
        payload: bytes!(0x02),
    });

    assert_eq!(
        Some(Sample {
            data: bytes!(0x01),
            duration: Duration::from_secs(1),
            packet_timestamp: 2,
            ..Default::default()
        }),
        s.pop(),
        "Failed to build samples after large gap"
    );
    assert_eq!(None, s.pop(), "Failed to build samples after large gap");

    s.push(Packet {
        header: Header {
            sequence_number: 6000,
            timestamp: 600,
            ..Default::default()
        },
        payload: bytes!(0x03),
    });
    assert_eq!(
        Some(Sample {
            data: bytes!(0x02),
            duration: Duration::from_secs(1),
            packet_timestamp: 500,
            prev_dropped_packets: 4998,
            ..Default::default()
        }),
        s.pop(),
        "Failed to build samples after large gap"
    );
    assert_eq!(
        Some(Sample {
            data: bytes!(0x02),
            duration: Duration::from_secs(1),
            packet_timestamp: 501,
            ..Default::default()
        }),
        s.pop(),
        "Failed to build samples after large gap"
    );
}
