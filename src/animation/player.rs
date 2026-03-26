use crate::asset::Handle;
use super::AnimationClip;

/// Playback state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayState {
    Playing,
    Paused,
    Stopped,
}

/// Loop mode for animation playback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}

/// Animation playback controller.
#[derive(Clone, Debug)]
pub struct AnimationPlayer {
    pub clip: Handle<AnimationClip>,
    pub time: f32,
    pub speed: f32,
    pub loop_mode: LoopMode,
    pub state: PlayState,
    direction: f32, // 1.0 or -1.0 for ping-pong
}

impl AnimationPlayer {
    /// Create a new animation player.
    pub fn new(clip: Handle<AnimationClip>) -> Self {
        Self {
            clip,
            time: 0.0,
            speed: 1.0,
            loop_mode: LoopMode::Loop,
            state: PlayState::Stopped,
            direction: 1.0,
        }
    }

    /// Start playing.
    pub fn play(&mut self) {
        self.state = PlayState::Playing;
    }

    /// Pause playback.
    pub fn pause(&mut self) {
        self.state = PlayState::Paused;
    }

    /// Stop and reset to beginning.
    pub fn stop(&mut self) {
        self.state = PlayState::Stopped;
        self.time = 0.0;
        self.direction = 1.0;
    }

    /// Set the current time.
    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    /// Update the player, advancing time.
    pub fn update(&mut self, dt: f32, duration: f32) {
        if self.state != PlayState::Playing {
            return;
        }
        
        self.time += dt * self.speed * self.direction;
        
        match self.loop_mode {
            LoopMode::Once => {
                if self.time >= duration {
                    self.time = duration;
                    self.state = PlayState::Stopped;
                }
            }
            LoopMode::Loop => {
                if self.time >= duration {
                    self.time = self.time % duration;
                } else if self.time < 0.0 {
                    self.time = duration + (self.time % duration);
                }
            }
            LoopMode::PingPong => {
                if self.time >= duration {
                    self.time = duration;
                    self.direction = -1.0;
                } else if self.time <= 0.0 {
                    self.time = 0.0;
                    self.direction = 1.0;
                }
            }
        }
    }

    /// Check if playback has finished (for Once mode).
    pub fn is_finished(&self) -> bool {
        self.loop_mode == LoopMode::Once && self.state == PlayState::Stopped && self.time > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_handle() -> Handle<AnimationClip> {
        Handle::new(1)
    }

    #[test]
    fn test_play_pause_stop() {
        let mut player = AnimationPlayer::new(dummy_handle());
        assert_eq!(player.state, PlayState::Stopped);
        
        player.play();
        assert_eq!(player.state, PlayState::Playing);
        
        player.pause();
        assert_eq!(player.state, PlayState::Paused);
        
        player.stop();
        assert_eq!(player.state, PlayState::Stopped);
        assert_eq!(player.time, 0.0);
    }

    #[test]
    fn test_update_advances_time() {
        let mut player = AnimationPlayer::new(dummy_handle());
        player.play();
        player.update(0.5, 1.0);
        assert!((player.time - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_loop_mode() {
        let mut player = AnimationPlayer::new(dummy_handle());
        player.loop_mode = LoopMode::Loop;
        player.play();
        player.time = 0.9;
        player.update(0.2, 1.0);
        assert!((player.time - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_once_mode() {
        let mut player = AnimationPlayer::new(dummy_handle());
        player.loop_mode = LoopMode::Once;
        player.play();
        player.time = 0.9;
        player.update(0.2, 1.0);
        assert_eq!(player.time, 1.0);
        assert_eq!(player.state, PlayState::Stopped);
    }

    #[test]
    fn test_ping_pong_mode() {
        let mut player = AnimationPlayer::new(dummy_handle());
        player.loop_mode = LoopMode::PingPong;
        player.play();
        player.time = 0.9;
        player.update(0.2, 1.0);
        assert_eq!(player.time, 1.0);
        assert_eq!(player.direction, -1.0);
        
        player.update(0.3, 1.0);
        assert!((player.time - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_speed() {
        let mut player = AnimationPlayer::new(dummy_handle());
        player.speed = 2.0;
        player.play();
        player.update(0.5, 2.0);
        assert!((player.time - 1.0).abs() < 0.001);
    }
}
