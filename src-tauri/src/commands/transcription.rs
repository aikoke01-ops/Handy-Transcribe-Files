use crate::managers::transcription::TranscriptionManager;
use crate::settings::{get_settings, write_settings, ModelUnloadTimeout};
use serde::Serialize;
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Serialize, Type)]
pub struct ModelLoadStatus {
    is_loaded: bool,
    current_model: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn set_model_unload_timeout(app: AppHandle, timeout: ModelUnloadTimeout) {
    let mut settings = get_settings(&app);
    settings.model_unload_timeout = timeout;
    write_settings(&app, settings);
}

#[tauri::command]
#[specta::specta]
pub fn get_model_load_status(
    transcription_manager: State<TranscriptionManager>,
) -> Result<ModelLoadStatus, String> {
    Ok(ModelLoadStatus {
        is_loaded: transcription_manager.is_model_loaded(),
        current_model: transcription_manager.get_current_model(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn unload_model_manually(
    transcription_manager: State<TranscriptionManager>,
) -> Result<(), String> {
    transcription_manager
        .unload_model()
        .map_err(|e| format!("Failed to unload model: {}", e))
}

/// Transcribe an arbitrary audio or video file the user picked from disk
/// (as opposed to a live microphone recording). Decodes/resamples the file
/// to 16 kHz mono via `audio_toolkit::decode_media_file_to_16k_mono` (which
/// falls back to a system `ffmpeg` for containers our built-in decoder can't
/// handle), then runs it through the same transcription pipeline used for
/// recordings. If no model is currently loaded, the user's selected model is
/// loaded first.
#[tauri::command]
#[specta::specta]
pub async fn transcribe_media_file(
    app: AppHandle,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    path: String,
) -> Result<String, String> {
    let tm = transcription_manager.inner().clone();
    let path_clone = path.clone();

    // Decoding + transcription can both take a while (and the decoder shells
    // out to ffmpeg in the fallback case), so run this off the async runtime
    // thread that's servicing the IPC call.
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        if !tm.is_model_loaded() {
            let model_id = get_settings(&app).selected_model;
            if model_id.is_empty() {
                return Err(
                    "No transcription model is selected. Pick one in Settings first.".into(),
                );
            }
            tm.load_model(&model_id)
                .map_err(|e| format!("Failed to load model '{}': {}", model_id, e))?;
        }

        let samples = crate::audio_toolkit::decode_media_file_to_16k_mono(&path_clone)
            .map_err(|e| format!("Failed to decode '{}': {}", path_clone, e))?;

        if samples.is_empty() {
            return Err("No audio could be extracted from that file.".into());
        }

        tm.transcribe(samples)
            .map_err(|e| format!("Transcription failed: {}", e))
    })
    .await
    .map_err(|e| format!("Transcription task panicked: {}", e))?
}
