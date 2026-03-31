use crate::types::{Mode, OutputInfo};
use swayipc::Connection;

pub fn get_outputs() -> Result<Vec<OutputInfo>, String> {
    let mut conn = Connection::new().map_err(|e| e.to_string())?;
    let outputs = conn.get_outputs().map_err(|e| e.to_string())?;

    let mut result: Vec<OutputInfo> = outputs
        .into_iter()
        .filter(|o| o.active)
        .map(|o| {
            let current_mode = o
                .current_mode
                .map(|m| Mode {
                    width: m.width,
                    height: m.height,
                    refresh: m.refresh,
                })
                .unwrap_or(Mode {
                    width: 0,
                    height: 0,
                    refresh: 0,
                });

            OutputInfo {
                name: o.name,
                make: o.make,
                model: o.model,
                x: o.rect.x,
                y: o.rect.y,
                width: o.rect.width,
                height: o.rect.height,
                modes: o
                    .modes
                    .into_iter()
                    .map(|m| Mode {
                        width: m.width,
                        height: m.height,
                        refresh: m.refresh,
                    })
                    .collect(),
                current_mode,
                scale: o.scale.unwrap_or(1.0),
                active: o.active,
            }
        })
        .collect();

    result.sort_by_key(|o| o.x);
    Ok(result)
}

pub fn apply_config(outputs: &[OutputInfo], mirror: bool) -> Result<(), String> {
    let mut conn = Connection::new().map_err(|e| e.to_string())?;

    for output in outputs {
        let (x, y) = if mirror { (0, 0) } else { (output.x, output.y) };
        let hz = (output.current_mode.refresh as f64 / 1000.0).round() as i32;
        let cmd = format!(
            "output {} mode {}x{}@{}Hz pos {} {}",
            output.name, output.current_mode.width, output.current_mode.height, hz, x, y
        );
        let results = conn.run_command(&cmd).map_err(|e| e.to_string())?;
        for result in results {
            result.map_err(|e| format!("{}: {}", output.name, e))?;
        }
    }

    // Refocus our window since output changes may move the cursor to another screen
    let _ = conn.run_command(format!("[app_id={}] focus", crate::APP_ID));

    Ok(())
}
