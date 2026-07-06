use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TimeOfDay {
    #[default]
    Dawn,
    Day,
    Dusk,
    Night,
}

#[derive(Resource)]
pub struct DayNightCycle {
    pub current_phase: TimeOfDay,
    pub time_elapsed: f32,
    pub phase_duration: f32, // How long each phase lasts in seconds
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            current_phase: TimeOfDay::Day,
            time_elapsed: 0.0,
            phase_duration: 30.0, // 30 seconds per phase for demo purposes
        }
    }
}

pub struct TimeCyclePlugin;

impl Plugin for TimeCyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayNightCycle>()
           .add_systems(Update, update_time_cycle);
    }
}

fn update_time_cycle(time: Res<Time>, mut cycle: ResMut<DayNightCycle>) {
    cycle.time_elapsed += time.delta_secs();

    if cycle.time_elapsed >= cycle.phase_duration {
        cycle.time_elapsed = 0.0;
        cycle.current_phase = match cycle.current_phase {
            TimeOfDay::Dawn => TimeOfDay::Day,
            TimeOfDay::Day => TimeOfDay::Dusk,
            TimeOfDay::Dusk => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::Dawn,
        };
        info!("Time of day shifted to: {:?}", cycle.current_phase);
    }
}
