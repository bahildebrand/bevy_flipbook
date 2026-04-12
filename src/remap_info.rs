use std::collections::HashMap;

use serde::Deserialize;

/// Top-level structure of a `*-remap_info.json` file produced by OpenVAT.
#[derive(Debug, Clone, Deserialize)]
pub struct RemapInfo {
    #[serde(rename = "os-remap")]
    pub os_remap: OsRemap,
    /// Animation clips keyed by name (e.g. `"Walk"`, `"Run"`).
    pub animations: HashMap<String, AnimationClip>,
}

impl RemapInfo {
    /// Parse from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Look up a clip by name.
    pub fn clip(&self, name: &str) -> Option<&AnimationClip> {
        self.animations.get(name)
    }

    /// All clips as a sorted `Vec` of `(name, clip)` pairs, ordered by `start_frame`.
    pub fn clips_ordered(&self) -> Vec<(&str, &AnimationClip)> {
        let mut clips: Vec<(&str, &AnimationClip)> = self
            .animations
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        clips.sort_by_key(|(_, c)| c.start_frame);
        clips
    }
}

/// The `os-remap` block describing the overall bounding box and frame count.
#[derive(Debug, Clone, Deserialize)]
pub struct OsRemap {
    #[serde(rename = "Min")]
    pub min: [f32; 3],
    #[serde(rename = "Max")]
    pub max: [f32; 3],
    #[serde(rename = "Frames")]
    pub frames: u32,
}

/// A single named animation clip.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationClip {
    pub start_frame: u32,
    pub end_frame: u32,
    pub framerate: f32,
    pub looping: bool,
}

impl AnimationClip {
    /// Number of frames in the clip (`end_frame - start_frame`).
    pub fn frame_count(&self) -> u32 {
        self.end_frame - self.start_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "os-remap": { "Min": [-8.0, -46.2, -18.5], "Max": [55.1, 52.0, 54.7], "Frames": 128 },
        "animations": {
            "Survey": { "startFrame": 0,  "endFrame": 82,  "framerate": 30, "looping": true },
            "Walk":   { "startFrame": 82, "endFrame": 99,  "framerate": 30, "looping": true },
            "Run":    { "startFrame": 99, "endFrame": 127, "framerate": 30, "looping": true }
        }
    }"#;

    #[test]
    fn parses_os_remap() {
        let info = RemapInfo::from_json(SAMPLE).unwrap();
        assert_eq!(info.os_remap.frames, 128);
        assert_eq!(info.os_remap.min, [-8.0, -46.2, -18.5]);
        assert_eq!(info.os_remap.max, [55.1, 52.0, 54.7]);
    }

    #[test]
    fn parses_animations() {
        let info = RemapInfo::from_json(SAMPLE).unwrap();
        assert_eq!(info.animations.len(), 3);

        let walk = info.clip("Walk").unwrap();
        assert_eq!(walk.start_frame, 82);
        assert_eq!(walk.end_frame, 99);
        assert_eq!(walk.frame_count(), 17);
        assert!(walk.looping);
    }

    #[test]
    fn clips_ordered_by_start_frame() {
        let info = RemapInfo::from_json(SAMPLE).unwrap();
        let ordered = info.clips_ordered();
        assert_eq!(ordered[0].0, "Survey");
        assert_eq!(ordered[1].0, "Walk");
        assert_eq!(ordered[2].0, "Run");
    }
}
