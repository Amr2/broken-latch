#[cfg(test)]
mod overlay_test {
    use broken_latch::overlay::Rect;

    #[test]
    fn test_rect_bounds() {
        let rect = Rect {
            x: 100,
            y: 200,
            width: 300,
            height: 400,
        };

        // Test that rect properties are correct
        assert_eq!(rect.x, 100);
        assert_eq!(rect.y, 200);
        assert_eq!(rect.width, 300);
        assert_eq!(rect.height, 400);

        // Test right and bottom bounds
        assert_eq!(rect.x + rect.width, 400);
        assert_eq!(rect.y + rect.height, 600);
    }

    #[test]
    fn test_rect_json_roundtrip() {
        let rect = Rect {
            x: 50,
            y: 75,
            width: 150,
            height: 225,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&rect).expect("Failed to serialize");

        // Deserialize back
        let deserialized: Rect = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.x, rect.x);
        assert_eq!(deserialized.y, rect.y);
        assert_eq!(deserialized.width, rect.width);
        assert_eq!(deserialized.height, rect.height);
    }

    #[test]
    fn test_multiple_rects() {
        let rects = vec![
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            Rect {
                x: 150,
                y: 150,
                width: 200,
                height: 200,
            },
            Rect {
                x: 400,
                y: 400,
                width: 300,
                height: 300,
            },
        ];

        assert_eq!(rects.len(), 3);
        assert_eq!(rects[1].x, 150);
        assert_eq!(rects[2].width, 300);
    }
}
