//! labwc_ipc — interaction avec labwc
//!
//! labwc n'expose pas de socket IPC JSON. Les deux mécanismes disponibles sont :
//!   - SIGHUP → recharge rc.xml (équivalent de `labwc -r`)
//!   - Keybindings natifs `SnapToEdge` dans rc.xml → gèrent le snapping
//!
//! Ce module :
//!   1. Génère le snippet XML des keybinds à injecter dans rc.xml
//!   2. Patche rc.xml (insère entre les balises <keyboard> existantes, ou
//!      ajoute une section <keyboard> si absente)
//!   3. Envoie SIGHUP à labwc via $LABWC_PID

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Génère le bloc XML des keybindings SnapToEdge pour labwc.
///
/// Zones supportées (même comportement que le snap Ubuntu) :
///   - Moitié gauche/droite/haut/bas : left, right, up, down
///   - Plein écran : Maximize (toggle)
///   - Quarts : up-left, up-right, down-left, down-right
///
/// Le modificateur est typiquement "W" (Super/Logo).
pub fn keybind_snippet(modifier: &str) -> String {
    let m = modifier;
    format!(
        r#"  <!-- waysnap: window snapping keybindings (modifier={m}) -->
  <keyboard>
    <!-- Half-screen snapping -->
    <keybind key="{m}-Left">
      <action name="SnapToEdge" direction="left"/>
    </keybind>
    <keybind key="{m}-Right">
      <action name="SnapToEdge" direction="right"/>
    </keybind>
    <keybind key="{m}-Up">
      <action name="SnapToEdge" direction="up"/>
    </keybind>
    <keybind key="{m}-Down">
      <action name="SnapToEdge" direction="down"/>
    </keybind>

    <!-- Fullscreen toggle -->
    <keybind key="{m}-f">
      <action name="ToggleMaximize"/>
    </keybind>

    <!-- Quarter snapping (press two directional keys in sequence,
         or use combine="yes" with SnapToEdge) -->
    <keybind key="{m}-KP_7">
      <action name="SnapToEdge" direction="up-left"/>
    </keybind>
    <keybind key="{m}-KP_9">
      <action name="SnapToEdge" direction="up-right"/>
    </keybind>
    <keybind key="{m}-KP_1">
      <action name="SnapToEdge" direction="down-left"/>
    </keybind>
    <keybind key="{m}-KP_3">
      <action name="SnapToEdge" direction="down-right"/>
    </keybind>
  </keyboard>
  <!-- end waysnap -->
"#
    )
}

/// Chemin canonique vers ~/.config/labwc/rc.xml
fn rc_xml_path() -> PathBuf {
    let config_home = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").expect("$HOME is not set");
            PathBuf::from(home).join(".config")
        });
    config_home.join("labwc").join("rc.xml")
}

/// Installe le snippet dans rc.xml.
///
/// Stratégie :
///   1. Si rc.xml n'existe pas → crée un rc.xml minimal avec le snippet.
///   2. Si rc.xml existe et contient déjà `<!-- waysnap:` → remplace le bloc.
///   3. Si rc.xml utilise une balise auto-fermante `<openbox_config ... />`
///      → remplace par `<openbox_config ...>\nSNIPPET\n</openbox_config>`.
///   4. Sinon → insère le snippet avant `</labwc_config>` ou `</openbox_config>`.
///
/// Retourne le chemin du fichier modifié.
pub fn install_snippet(snippet: &str) -> io::Result<PathBuf> {
    let path = rc_xml_path();

    // Crée le répertoire si nécessaire
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        let content = minimal_rc_xml(snippet);
        fs::write(&path, content)?;
        return Ok(path);
    }

    let original = fs::read_to_string(&path)?;

    let patched = if original.contains("<!-- waysnap:") {
        replace_waysnap_block(&original, snippet)
    } else if is_self_closing_root(&original) {
        expand_self_closing_root(&original, snippet)
    } else {
        insert_before_closing_tag(&original, snippet)
    };

    // Sauvegarde avant écrasement
    let backup = path.with_extension("xml.waysnap.bak");
    fs::copy(&path, &backup)?;

    fs::write(&path, patched)?;
    Ok(path)
}

/// Remplace le bloc existant waysnap (entre `<!-- waysnap:` et `<!-- end waysnap -->`).
fn replace_waysnap_block(content: &str, snippet: &str) -> String {
    let start_marker = "<!-- waysnap:";
    let end_marker = "  <!-- end waysnap -->";

    let start = content.find(start_marker);
    let end = content.find(end_marker).map(|i| i + end_marker.len());

    match (start, end) {
        (Some(s), Some(e)) => {
            let mut out = String::with_capacity(content.len());
            out.push_str(&content[..s]);
            out.push_str(snippet);
            // Skip trailing newline after end_marker if present
            let rest = &content[e..];
            out.push_str(rest.trim_start_matches('\n'));
            out
        }
        _ => insert_before_closing_tag(content, snippet),
    }
}

/// Détecte si le rc.xml utilise une balise racine auto-fermante
/// du type `<openbox_config ... />` ou `<labwc_config/>`.
fn is_self_closing_root(content: &str) -> bool {
    // On cherche une balise racine qui se termine par "/>", sans balise fermante séparée
    let has_closing = content.contains("</labwc_config>") || content.contains("</openbox_config>");
    if has_closing {
        return false;
    }
    // Vérifie qu'une balise racine connue existe sous forme auto-fermante
    content.contains("<openbox_config") || content.contains("<labwc_config")
}

/// Transforme `<openbox_config ... />` en `<openbox_config ...>\nSNIPPET\n</openbox_config>`.
/// Fonctionne aussi avec `<labwc_config`.
fn expand_self_closing_root(content: &str, snippet: &str) -> String {
    // Trouve le tag racine (openbox_config ou labwc_config)
    let tag_name = if content.contains("<openbox_config") {
        "openbox_config"
    } else {
        "labwc_config"
    };

    // Trouve la position de "/>" finale de la balise racine
    // On cherche depuis le début du tag pour éviter les faux positifs
    let tag_start = content.find(&format!("<{tag_name}")).unwrap_or(0);
    if let Some(rel_pos) = content[tag_start..].find("/>") {
        let self_close_pos = tag_start + rel_pos;
        let mut out = String::with_capacity(content.len() + snippet.len() + 32);
        // Remplace "/> " par ">\nSNIPPET\n</tag>"
        out.push_str(&content[..self_close_pos]);
        out.push_str(">\n");
        out.push_str(snippet);
        out.push_str(&format!("</{tag_name}>\n"));
        // Ajoute ce qui suit le "/>" s'il y a quelque chose (ex: commentaire)
        let after = content[self_close_pos + 2..].trim();
        if !after.is_empty() {
            out.push_str(after);
            out.push('\n');
        }
        out
    } else {
        // Pas trouvé — on ajoute à la fin
        let mut out = content.to_owned();
        out.push('\n');
        out.push_str(snippet);
        out
    }
}

/// Insère le snippet juste avant la balise fermante racine.
///
/// Labwc accepte deux noms de racine pour la compat openbox :
///   - `</labwc_config>`   (format natif labwc)
///   - `</openbox_config>` (format openbox hérité)
///
/// Si aucune des deux n'est trouvée, on ajoute à la fin du fichier.
fn insert_before_closing_tag(content: &str, snippet: &str) -> String {
    // Priorité à labwc_config, sinon openbox_config
    let closing = ["</labwc_config>", "</openbox_config>"]
        .iter()
        .find_map(|tag| content.rfind(tag).map(|pos| (pos, *tag)));

    if let Some((pos, _tag)) = closing {
        let mut out = String::with_capacity(content.len() + snippet.len());
        out.push_str(&content[..pos]);
        out.push('\n');
        out.push_str(snippet);
        out.push_str(&content[pos..]);
        out
    } else {
        // rc.xml sans balise fermante reconnue — on ajoute à la fin
        let mut out = content.to_owned();
        out.push('\n');
        out.push_str(snippet);
        out
    }
}

/// Crée un rc.xml minimal valide contenant uniquement le snippet waysnap.
fn minimal_rc_xml(snippet: &str) -> String {
    format!(
        r#"<?xml version="1.0"?>
<labwc_config>

{snippet}
</labwc_config>
"#
    )
}

/// Envoie SIGHUP à labwc pour recharger la configuration.
///
/// labwc expose son PID via la variable d'environnement $LABWC_PID.
/// Cela correspond exactement à ce que fait `labwc -r` en interne.
pub fn reload_labwc() -> Result<(), String> {
    let pid_str = env::var("LABWC_PID")
        .map_err(|_| "$LABWC_PID is not set — is labwc running?".to_string())?;

    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|_| format!("$LABWC_PID is not a valid PID: {pid_str:?}"))?;

    if pid <= 0 {
        return Err(format!("invalid PID: {pid}"));
    }

    // SAFETY: kill() est une syscall POSIX standard.
    // pid > 0, signal = SIGHUP (1). Aucune mémoire partagée impliquée.
    let ret = unsafe { libc_kill(pid, SIGHUP) };
    if ret == 0 {
        Ok(())
    } else {
        Err(format!(
            "kill({pid}, SIGHUP) failed — is labwc still running?"
        ))
    }
}

// ---------------------------------------------------------------------------
// Liaison minimale à libc (kill + SIGHUP) sans dépendance à la crate `libc`
// ---------------------------------------------------------------------------

const SIGHUP: i32 = 1;

extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

#[inline(always)]
unsafe fn libc_kill(pid: i32, sig: i32) -> i32 {
    unsafe { kill(pid, sig) }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_contains_snap_to_edge() {
        let s = keybind_snippet("W");
        assert!(s.contains("SnapToEdge"));
        assert!(s.contains("W-Left"));
        assert!(s.contains("W-Right"));
        assert!(s.contains("W-Up"));
        assert!(s.contains("W-Down"));
        assert!(s.contains("direction=\"up-left\""));
        assert!(s.contains("direction=\"down-right\""));
    }

    #[test]
    fn insert_into_minimal_xml() {
        let xml = "<?xml version=\"1.0\"?>\n<labwc_config>\n</labwc_config>\n";
        let snippet = "  <keyboard><keybind key=\"W-Left\"/></keyboard>\n";
        let result = insert_before_closing_tag(xml, snippet);
        assert!(result.contains(snippet));
        assert!(result.contains("</labwc_config>"));
        // Le snippet doit apparaître avant la balise fermante
        let pos_snippet = result.find(snippet).unwrap();
        let pos_close = result.find("</labwc_config>").unwrap();
        assert!(pos_snippet < pos_close);
    }

    #[test]
    fn insert_into_openbox_compat_xml() {
        // labwc accepte aussi <openbox_config> avec balise fermante explicite
        let xml = "<?xml version=\"1.0\"?>\n<openbox_config>\n</openbox_config>\n";
        let snippet = "  <keyboard><keybind key=\"W-Left\"/></keyboard>\n";
        let result = insert_before_closing_tag(xml, snippet);
        assert!(result.contains(snippet));
        assert!(result.contains("</openbox_config>"));
        let pos_snippet = result.find(snippet).unwrap();
        let pos_close = result.find("</openbox_config>").unwrap();
        assert!(pos_snippet < pos_close);
    }

    #[test]
    fn detect_self_closing_root() {
        let self_closing =
            "<?xml version=\"1.0\"?>\n<openbox_config xmlns=\"http://openbox.org/3.4/rc\"/>";
        let normal = "<?xml version=\"1.0\"?>\n<openbox_config>\n</openbox_config>\n";
        assert!(is_self_closing_root(self_closing));
        assert!(!is_self_closing_root(normal));
    }

    #[test]
    fn expand_self_closing_openbox_config() {
        let xml = "<?xml version=\"1.0\"?>\n<openbox_config xmlns=\"http://openbox.org/3.4/rc\"/>";
        let snippet = "  <!-- waysnap: test -->\n  <!-- end waysnap -->\n";
        let result = expand_self_closing_root(xml, snippet);
        assert!(result.contains("</openbox_config>"));
        assert!(result.contains(snippet));
        assert!(!result.contains("/>"));
        let pos_snippet = result.find(snippet).unwrap();
        let pos_close = result.find("</openbox_config>").unwrap();
        assert!(pos_snippet < pos_close);
    }

    #[test]
    fn replace_existing_waysnap_block() {
        let xml = concat!(
            "<labwc_config>\n",
            "  <!-- waysnap: old -->\n",
            "  <keyboard/>\n",
            "  <!-- end waysnap -->\n",
            "</labwc_config>\n",
        );
        let new_snippet = "  <!-- waysnap: new -->\n  <!-- end waysnap -->\n";
        let result = replace_waysnap_block(xml, new_snippet);
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
    }

    #[test]
    fn minimal_xml_is_valid_structure() {
        let s = minimal_rc_xml("  <!-- test -->\n");
        assert!(s.starts_with("<?xml"));
        assert!(s.contains("<labwc_config>"));
        assert!(s.contains("</labwc_config>"));
        assert!(s.contains("<!-- test -->"));
    }
}
