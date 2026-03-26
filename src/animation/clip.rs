/// Which property an animation channel affects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Property {
    Position,
    Rotation,
    Scale,
}

/// Interpolation mode for keyframes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interpolation {
    Step,
    Linear,
    CubicSpline,
}

/// A keyframe with time and value.
#[derive(Clone, Debug)]
pub struct Keyframe {
    pub time: f32,
    pub value: [f32; 4],
}

/// A channel animating one property of one bone.
#[derive(Clone, Debug)]
pub struct Channel {
    pub bone_index: usize,
    pub property: Property,
    pub interpolation: Interpolation,
    pub keyframes: Vec<Keyframe>,
}

impl Channel {
    /// Sample the channel at a given time.
    pub fn sample(&self, time: f32) -> [f32; 4] {
        if self.keyframes.is_empty() {
            return [0.0; 4];
        }
        
        if self.keyframes.len() == 1 || time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        
        let last = &self.keyframes[self.keyframes.len() - 1];
        if time >= last.time {
            return last.value;
        }
        
        // Find surrounding keyframes
        let mut prev_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time > time {
                break;
            }
            prev_idx = i;
        }
        
        let prev = &self.keyframes[prev_idx];
        let next = &self.keyframes[prev_idx + 1];
        let t = (time - prev.time) / (next.time - prev.time);
        
        match self.interpolation {
            Interpolation::Step => prev.value,
            Interpolation::Linear => interpolate_linear(&prev.value, &next.value, t),
            Interpolation::CubicSpline => interpolate_linear(&prev.value, &next.value, t), // TODO: proper cubic
        }
    }
}

fn interpolate_linear(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// An animation clip containing multiple channels.
#[derive(Clone, Debug)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<Channel>,
}

impl AnimationClip {
    /// Sample all channels at a given time.
    pub fn sample(&self, time: f32) -> Vec<(usize, Property, [f32; 4])> {
        self.channels
            .iter()
            .map(|c| (c.bone_index, c.property, c.sample(time)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let channel = Channel {
            bone_index: 0,
            property: Property::Position,
            interpolation: Interpolation::Linear,
            keyframes: vec![
                Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
                Keyframe { time: 1.0, value: [1.0, 1.0, 1.0, 1.0] },
            ],
        };
        
        let val = channel.sample(0.5);
        assert!((val[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_step_interpolation() {
        let channel = Channel {
            bone_index: 0,
            property: Property::Position,
            interpolation: Interpolation::Step,
            keyframes: vec![
                Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
                Keyframe { time: 1.0, value: [1.0, 1.0, 1.0, 1.0] },
            ],
        };
        
        let val = channel.sample(0.5);
        assert!((val[0] - 0.0).abs() < 0.001); // Should be first value
    }

    #[test]
    fn test_before_first_keyframe() {
        let channel = Channel {
            bone_index: 0,
            property: Property::Position,
            interpolation: Interpolation::Linear,
            keyframes: vec![
                Keyframe { time: 1.0, value: [1.0, 1.0, 1.0, 1.0] },
            ],
        };
        
        let val = channel.sample(0.0);
        assert!((val[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_clip_sample() {
        let clip = AnimationClip {
            name: "test".into(),
            duration: 1.0,
            channels: vec![
                Channel {
                    bone_index: 0,
                    property: Property::Position,
                    interpolation: Interpolation::Linear,
                    keyframes: vec![
                        Keyframe { time: 0.0, value: [0.0, 0.0, 0.0, 0.0] },
                        Keyframe { time: 1.0, value: [1.0, 0.0, 0.0, 0.0] },
                    ],
                },
            ],
        };
        
        let samples = clip.sample(0.5);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].0, 0);
        assert_eq!(samples[0].1, Property::Position);
        assert!((samples[0].2[0] - 0.5).abs() < 0.001);
    }
}
