use std::process::Command;

#[derive(Clone, Debug)]
pub struct SignatureInfo {
    pub team_id: Option<String>,
    pub authority: Option<String>,
}

impl SignatureInfo {
    pub fn display_name(&self) -> String {
        if let Some(auth) = &self.authority {
            // Strip "Developer ID Application: " if present for cleaner display
            let clean = auth.replace("Developer ID Application: ", "");
            if let Some(team) = &self.team_id {
                format!("{} (Team: {})", clean, team)
            } else {
                clean
            }
        } else if let Some(team) = &self.team_id {
            format!("Team ID: {}", team)
        } else {
            "Firma Ad-Hoc / Sin Autoridad".to_string()
        }
    }
}

pub fn get_signature_info(path: &str) -> Option<SignatureInfo> {
    let output = Command::new("codesign")
        .arg("-dv")
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None; // No firmado o error
    }

    // codesign -dv escribe en stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let mut team_id = None;
    let mut authority = None;

    for line in stderr.lines() {
        if line.starts_with("TeamIdentifier=") {
            team_id = Some(line["TeamIdentifier=".len()..].to_string());
        } else if authority.is_none() && line.starts_with("Authority=") {
            // Guardamos la primera autoridad, que suele ser el Developer ID
            authority = Some(line["Authority=".len()..].to_string());
        }
    }

    if team_id.is_none() && authority.is_none() {
        return None;
    }

    Some(SignatureInfo { team_id, authority })
}
