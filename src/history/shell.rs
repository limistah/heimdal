pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

impl std::str::FromStr for Shell {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "zsh" => Ok(Shell::Zsh),
            "bash" => Ok(Shell::Bash),
            "fish" => Ok(Shell::Fish),
            other => anyhow::bail!("unsupported shell: {other}. Supported: zsh, bash, fish"),
        }
    }
}

pub fn init_script(shell: Shell) -> String {
    match shell {
        Shell::Zsh => ZSH_INIT.to_string(),
        Shell::Bash => BASH_INIT.to_string(),
        Shell::Fish => FISH_INIT.to_string(),
    }
}

const ZSH_INIT: &str = r#"
# heimdal history — added by `heimdal history shell-init --shell zsh`
_heimdal_record() {
  local exit_code=$?
  local raw_cmd
  local cmd
  raw_cmd=$(fc -ln -1 2>/dev/null)
  # Respect leading-space convention: " command" is not recorded (check BEFORE stripping)
  case "$raw_cmd" in
    " "*)  return $exit_code ;;
  esac
  cmd=$(printf '%s' "$raw_cmd" | sed 's/^[[:space:]]*//')
  [ -z "$cmd" ] && return $exit_code
  heimdal history record --cmd "$cmd" --exit $exit_code --dir "$PWD" --session "${HEIMDAL_SESSION:-$$}" &>/dev/null &
  return $exit_code
}
precmd_functions+=(_heimdal_record)

# Assign a stable session ID for this shell instance
: "${HEIMDAL_SESSION:=$(heimdal history session-id 2>/dev/null || echo $$)}"
export HEIMDAL_SESSION

# Cross-machine Ctrl+R
_heimdal_search_widget() {
  local selected
  selected=$(heimdal history search --interactive 2>/dev/null)
  if [ -n "$selected" ]; then
    LBUFFER="$selected"
  fi
  zle redisplay
}
zle -N _heimdal_search_widget
bindkey '^R' _heimdal_search_widget
"#;

const BASH_INIT: &str = r#"
# heimdal history — added by `heimdal history shell-init --shell bash`
_heimdal_record() {
  local exit_code=$?
  local cmd
  # Strip history line number using the fixed two-space separator bash uses,
  # so a leading space in the command is preserved for the HIST_IGNORE_SPACE check.
  cmd=$(history 1 | sed 's/^[[:space:]]*[0-9][0-9]*  //')
  case "$cmd" in
    " "*) return $exit_code ;;
  esac
  [ -z "$cmd" ] && return $exit_code
  heimdal history record --cmd "$cmd" --exit $exit_code --dir "$PWD" --session "${HEIMDAL_SESSION:-$$}" &>/dev/null &
  return $exit_code
}
if [[ "$PROMPT_COMMAND" != *"_heimdal_record"* ]]; then
  PROMPT_COMMAND="_heimdal_record${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
: "${HEIMDAL_SESSION:=$(heimdal history session-id 2>/dev/null || echo $$)}"
export HEIMDAL_SESSION
"#;

const FISH_INIT: &str = r#"
# heimdal history — added by `heimdal history shell-init --shell fish`
function _heimdal_record --on-event fish_postexec
  set -l exit_code $status
  set -l cmd $argv[1]
  string match -rq '^ ' -- $cmd; and return $exit_code
  test -z "$cmd"; and return $exit_code
  heimdal history record --cmd "$cmd" --exit $exit_code --dir "$PWD" --session "$HEIMDAL_SESSION" &>/dev/null &
  return $exit_code
end
if not set -q HEIMDAL_SESSION
  set -gx HEIMDAL_SESSION (heimdal history session-id 2>/dev/null; or echo $fish_pid)
end
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_script_contains_precmd_hook() {
        let script = init_script(Shell::Zsh);
        assert!(script.contains("precmd_functions"));
        assert!(script.contains("heimdal history record"));
    }

    #[test]
    fn bash_script_contains_prompt_command() {
        let script = init_script(Shell::Bash);
        assert!(script.contains("PROMPT_COMMAND"));
        assert!(script.contains("heimdal history record"));
    }

    #[test]
    fn script_respects_leading_space_convention() {
        // The hook must check for a leading space and skip recording
        let script = init_script(Shell::Zsh);
        assert!(script.contains("\" \"*"));
    }
}
