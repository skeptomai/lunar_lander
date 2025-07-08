use rusty_audio::Audio;

pub fn load_audio() -> Audio {
    let mut audio = Audio::new();
    audio.add("ambient", "assets/sounds/218883-jet_whine_v2_mid_loop.wav");
    audio.add(
        "acceleration",
        "assets/sounds/218837-jet_turbine_main_blast.wav",
    );
    audio
}

pub fn update_audio(audio: &mut Audio) {
    if !audio.is_playing() {
        audio.play("ambient"); // Execution continues while playback occurs in another thread.
    }
}

pub fn shutdown_audio(audio: &mut Audio) {
    audio.stop();
}