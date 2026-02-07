use anyhow::Result;

use crate::config::schema::{
    DotfileMapping, DotfilesConfig, HookCommand, Profile, ProfileHooks, ProfileSource,
    ProfileTemplateConfig,
};

/// Built-in profile templates
pub struct ProfileTemplates;

impl ProfileTemplates {
    /// Get all available template names
    pub fn list() -> Vec<&'static str> {
        vec![
            "minimal",
            "developer",
            "devops",
            "macos-desktop",
            "linux-server",
            "workstation",
        ]
    }

    /// Get a template by name
    pub fn get(name: &str) -> Option<Profile> {
        match name {
            "minimal" => Some(Self::minimal()),
            "developer" => Some(Self::developer()),
            "devops" => Some(Self::devops()),
            "macos-desktop" => Some(Self::macos_desktop()),
            "linux-server" => Some(Self::linux_server()),
            "workstation" => Some(Self::workstation()),
            _ => None,
        }
    }

    /// Get template description
    pub fn description(name: &str) -> Option<&'static str> {
        match name {
            "minimal" => Some("Minimal profile with basic shell config"),
            "developer" => Some("Developer profile with common dev tools"),
            "devops" => Some("DevOps profile with infrastructure tools"),
            "macos-desktop" => Some("macOS desktop profile with GUI apps"),
            "linux-server" => Some("Linux server profile with system tools"),
            "workstation" => Some("Full workstation setup with all features"),
            _ => None,
        }
    }

    /// Minimal profile template
    fn minimal() -> Profile {
        Profile {
            extends: None,
            sources: vec![ProfileSource::Name("packages".to_string())],
            dotfiles: DotfilesConfig {
                use_stowrc: false,
                symlink_all: false,
                ignore: vec![],
                files: vec![
                    DotfileMapping {
                        source: "shell/.bashrc".to_string(),
                        target: "~/.bashrc".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "shell/.zshrc".to_string(),
                        target: "~/.zshrc".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "git/.gitconfig".to_string(),
                        target: "~/.gitconfig".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                ],
            },
            hooks: ProfileHooks::default(),
            templates: ProfileTemplateConfig::default(),
        }
    }

    /// Developer profile template
    fn developer() -> Profile {
        Profile {
            extends: Some("minimal".to_string()),
            sources: vec![
                ProfileSource::Name("homebrew".to_string()),
                ProfileSource::Name("github".to_string()),
            ],
            dotfiles: DotfilesConfig {
                use_stowrc: false,
                symlink_all: false,
                ignore: vec![],
                files: vec![
                    DotfileMapping {
                        source: "vim/.vimrc".to_string(),
                        target: "~/.vimrc".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "tmux/.tmux.conf".to_string(),
                        target: "~/.tmux.conf".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "vscode/settings.json".to_string(),
                        target: "~/Library/Application Support/Code/User/settings.json".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                ],
            },
            hooks: ProfileHooks::default(),
            templates: ProfileTemplateConfig::default(),
        }
    }

    /// DevOps profile template
    fn devops() -> Profile {
        Profile {
            extends: Some("developer".to_string()),
            sources: vec![
                ProfileSource::Name("homebrew".to_string()),
                ProfileSource::Name("apt".to_string()),
            ],
            dotfiles: DotfilesConfig {
                use_stowrc: false,
                symlink_all: false,
                ignore: vec![],
                files: vec![
                    DotfileMapping {
                        source: "aws/.aws/config".to_string(),
                        target: "~/.aws/config".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "kubectl/.kube/config".to_string(),
                        target: "~/.kube/config".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "docker/.docker/config.json".to_string(),
                        target: "~/.docker/config.json".to_string(),
                        post_link: vec![],
                        pre_unlink: vec![],
                        when: None,
                    },
                ],
            },
            hooks: ProfileHooks {
                post_apply: vec![HookCommand::Simple(
                    "echo 'DevOps tools configured!'".to_string(),
                )],
                ..Default::default()
            },
            templates: ProfileTemplateConfig::default(),
        }
    }

    /// macOS desktop profile template
    fn macos_desktop() -> Profile {
        Profile {
            extends: Some("developer".to_string()),
            sources: vec![
                ProfileSource::Name("homebrew".to_string()),
                ProfileSource::Name("mas".to_string()),
            ],
            dotfiles: DotfilesConfig {
                use_stowrc: false,
                symlink_all: false,
                ignore: vec![],
                files: vec![
                    DotfileMapping {
                        source: "macos/.yabairc".to_string(),
                        target: "~/.yabairc".to_string(),
                        post_link: vec![HookCommand::Simple(
                            "brew services restart yabai".to_string(),
                        )],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "macos/.skhdrc".to_string(),
                        target: "~/.skhdrc".to_string(),
                        post_link: vec![HookCommand::Simple(
                            "brew services restart skhd".to_string(),
                        )],
                        pre_unlink: vec![],
                        when: None,
                    },
                ],
            },
            hooks: ProfileHooks {
                post_apply: vec![HookCommand::Simple(
                    "echo 'macOS desktop configured!'".to_string(),
                )],
                ..Default::default()
            },
            templates: ProfileTemplateConfig::default(),
        }
    }

    /// Linux server profile template
    fn linux_server() -> Profile {
        Profile {
            extends: Some("minimal".to_string()),
            sources: vec![
                ProfileSource::Name("apt".to_string()),
                ProfileSource::Name("dnf".to_string()),
            ],
            dotfiles: DotfilesConfig {
                use_stowrc: false,
                symlink_all: false,
                ignore: vec![],
                files: vec![
                    DotfileMapping {
                        source: "nginx/nginx.conf".to_string(),
                        target: "/etc/nginx/nginx.conf".to_string(),
                        post_link: vec![HookCommand::Simple(
                            "sudo systemctl reload nginx".to_string(),
                        )],
                        pre_unlink: vec![],
                        when: None,
                    },
                    DotfileMapping {
                        source: "systemd/services/".to_string(),
                        target: "/etc/systemd/system/".to_string(),
                        post_link: vec![HookCommand::Simple(
                            "sudo systemctl daemon-reload".to_string(),
                        )],
                        pre_unlink: vec![],
                        when: None,
                    },
                ],
            },
            hooks: ProfileHooks {
                post_apply: vec![HookCommand::Simple(
                    "echo 'Linux server configured!'".to_string(),
                )],
                ..Default::default()
            },
            templates: ProfileTemplateConfig::default(),
        }
    }

    /// Workstation profile template (comprehensive)
    fn workstation() -> Profile {
        Profile {
            extends: Some("developer".to_string()),
            sources: vec![
                ProfileSource::Name("homebrew".to_string()),
                ProfileSource::Name("github".to_string()),
                ProfileSource::Name("custom".to_string()),
            ],
            dotfiles: DotfilesConfig {
                use_stowrc: true,
                symlink_all: true,
                ignore: vec![
                    "README*".to_string(),
                    "LICENSE*".to_string(),
                    ".git".to_string(),
                ],
                files: vec![],
            },
            hooks: ProfileHooks {
                pre_apply: vec![HookCommand::Simple(
                    "echo 'Starting workstation setup...'".to_string(),
                )],
                post_apply: vec![HookCommand::Simple(
                    "echo 'Workstation fully configured!'".to_string(),
                )],
                ..Default::default()
            },
            templates: ProfileTemplateConfig::default(),
        }
    }
}

/// Create a new profile from a template
pub fn create_from_template(_name: &str, template_name: &str) -> Result<Profile> {
    let template = ProfileTemplates::get(template_name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

    Ok(template)
}

/// Clone an existing profile with a new name
pub fn clone_profile(source_profile: &Profile) -> Profile {
    source_profile.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates() {
        let templates = ProfileTemplates::list();
        assert!(!templates.is_empty());
        assert!(templates.contains(&"minimal"));
        assert!(templates.contains(&"developer"));
    }

    #[test]
    fn test_get_template() {
        let minimal = ProfileTemplates::get("minimal");
        assert!(minimal.is_some());
        let profile = minimal.unwrap();
        assert!(profile.extends.is_none());
    }

    #[test]
    fn test_get_nonexistent_template() {
        let result = ProfileTemplates::get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_template_description() {
        let desc = ProfileTemplates::description("minimal");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("Minimal"));
    }

    #[test]
    fn test_create_from_template() {
        let result = create_from_template("my-profile", "minimal");
        assert!(result.is_ok());
    }

    #[test]
    fn test_developer_extends_minimal() {
        let dev = ProfileTemplates::get("developer").unwrap();
        assert_eq!(dev.extends, Some("minimal".to_string()));
    }

    #[test]
    fn test_clone_profile() {
        let original = ProfileTemplates::get("minimal").unwrap();
        let cloned = clone_profile(&original);
        assert_eq!(original.sources.len(), cloned.sources.len());
    }
}
