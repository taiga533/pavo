/// シェル統合スクリプトを生成する
///
/// # Arguments
/// * `shell` - シェルの種類 ("bash", "zsh", "fish")
///
/// # Returns
/// * `Result<String, anyhow::Error>` - 生成されたスクリプト文字列、または無効なシェルの場合はエラー
pub fn generate_init_script(shell: &str) -> anyhow::Result<String> {
    match shell {
        "bash" | "zsh" => Ok(generate_bash_zsh_script()),
        "fish" => Ok(generate_fish_script()),
        _ => Err(anyhow::anyhow!(
            "Unsupported shell: {}. Supported shells are: bash, zsh, fish",
            shell
        )),
    }
}

fn generate_bash_zsh_script() -> String {
    r#"# Pavo shell integration
# Add this line to your ~/.bashrc or ~/.zshrc:
# eval "$(pavo init bash)" or eval "$(pavo init zsh)"

p() {
    local result
    local args=()

    # Parse options and collect all arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -t|--tag)
                args+=("$1" "$2")
                shift 2
                ;;
            *)
                args+=("$1")
                shift
                ;;
        esac
    done

    result=$(pavo "${args[@]}" </dev/tty)
    if [ $? -eq 0 ] && [ -n "$result" ]; then
        if [ -d "$result" ]; then
            cd "$result" || return
        else
            echo "$result"
        fi
    fi
}

# Auto-record git repositories on directory change
_pavo_record_hook() {
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        local git_root
        git_root=$(git rev-parse --show-toplevel 2>/dev/null)
        if [ -n "$git_root" ]; then
            pavo add "$git_root" 2>/dev/null
        fi
    fi
}

# Hook for bash
if [ -n "$BASH_VERSION" ]; then
    if [[ "$PROMPT_COMMAND" != *"_pavo_record_hook"* ]]; then
        PROMPT_COMMAND="_pavo_record_hook${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
    fi
fi

# Hook for zsh
if [ -n "$ZSH_VERSION" ]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd _pavo_record_hook
fi
"#
    .to_string()
}

fn generate_fish_script() -> String {
    r#"# Pavo shell integration
# Add this line to your ~/.config/fish/config.fish:
# pavo init fish | source

function p
    set -l args

    # Parse options
    argparse 't/tag=' -- $argv
    if set -q _flag_tag
        set args --tag $_flag_tag
    end

    set -l result (pavo $args </dev/tty)
    if test $status -eq 0 -a -n "$result"
        if test -d "$result"
            cd $result
        else
            echo $result
        end
    end
end

# Auto-record git repositories on directory change
function _pavo_record_hook --on-variable PWD
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1
        set -l git_root (git rev-parse --show-toplevel 2>/dev/null)
        if test -n "$git_root"
            pavo add $git_root 2>/dev/null
        end
    end
end
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bashスクリプトが生成できること() {
        let script = generate_init_script("bash").unwrap();
        assert!(script.contains("p() {"));
        assert!(script.contains("cd \"$result\""));
        assert!(script.contains("</dev/tty"));
        assert!(script.contains("[ $? -eq 0 ]"));
    }

    #[test]
    fn test_zshスクリプトが生成できること() {
        let script = generate_init_script("zsh").unwrap();
        assert!(script.contains("p() {"));
        assert!(script.contains("cd \"$result\""));
        assert!(script.contains("</dev/tty"));
        assert!(script.contains("[ $? -eq 0 ]"));
    }

    #[test]
    fn test_fishスクリプトが生成できること() {
        let script = generate_init_script("fish").unwrap();
        assert!(script.contains("function p"));
        assert!(script.contains("cd $result"));
        assert!(script.contains("</dev/tty"));
        assert!(script.contains("test $status -eq 0"));
    }

    #[test]
    fn test_無効なシェルでエラーが返ること() {
        let result = generate_init_script("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported shell"));
    }

    #[test]
    fn test_bashスクリプトにインストール手順が含まれること() {
        let script = generate_init_script("bash").unwrap();
        assert!(script.contains("eval \"$(pavo init bash)\""));
        assert!(script.contains("~/.bashrc"));
    }

    #[test]
    fn test_fishスクリプトにインストール手順が含まれること() {
        let script = generate_init_script("fish").unwrap();
        assert!(script.contains("pavo init fish | source"));
        assert!(script.contains("~/.config/fish/config.fish"));
    }

    #[test]
    fn test_bashスクリプトに自動記録フックが含まれること() {
        let script = generate_init_script("bash").unwrap();
        assert!(script.contains("_pavo_record_hook"));
        assert!(script.contains("pavo add"));
        assert!(script.contains("git rev-parse --is-inside-work-tree"));
        assert!(script.contains("PROMPT_COMMAND"));
    }

    #[test]
    fn test_zshスクリプトに自動記録フックが含まれること() {
        let script = generate_init_script("zsh").unwrap();
        assert!(script.contains("_pavo_record_hook"));
        assert!(script.contains("pavo add"));
        assert!(script.contains("git rev-parse --is-inside-work-tree"));
        assert!(script.contains("add-zsh-hook precmd"));
    }

    #[test]
    fn test_fishスクリプトに自動記録フックが含まれること() {
        let script = generate_init_script("fish").unwrap();
        assert!(script.contains("_pavo_record_hook"));
        assert!(script.contains("pavo add"));
        assert!(script.contains("git rev-parse --is-inside-work-tree"));
        assert!(script.contains("--on-variable PWD"));
    }
}
