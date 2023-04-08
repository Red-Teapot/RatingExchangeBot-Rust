use log::error;

pub const DISCORD_BOT_TOKEN: EnvVar = EnvVar {
    name: "DISCORD_BOT_TOKEN",
    validator: non_empty_string,
};
pub const DATABASE_URL: EnvVar = EnvVar {
    name: "DATABASE_URL",
    validator: non_empty_string,
};

pub const REGISTER_COMMANDS_GLOBALLY: EnvVar = EnvVar {
    name: "REGISTER_COMMANDS_GLOBALLY",
    validator: boolean,
};
pub const REGISTER_COMMANDS_IN_GUILDS: EnvVar = EnvVar {
    name: "REGISTER_COMMANDS_IN_GUILDS",
    validator: uint_list,
};

const REQUIRED_VARS: &[EnvVar] = &[DISCORD_BOT_TOKEN, DATABASE_URL];
const OPTIONAL_VARS: &[EnvVar] = &[REGISTER_COMMANDS_GLOBALLY, REGISTER_COMMANDS_IN_GUILDS];

fn non_empty_string(string: &str) -> bool {
    !string.trim().is_empty()
}

fn boolean(string: &str) -> bool {
    let trimmed = string.trim().to_lowercase();

    trimmed == "true" || trimmed == "false"
}

fn uint_list(string: &str) -> bool {
    string
        .split(',')
        .map(|s| s.trim())
        .all(|s| {
            !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
        })
}

pub fn check() -> bool {
    let mut passed = true;

    for var in REQUIRED_VARS {
        if !envmnt::exists(var.name) {
            error!("Environment variable {} must be set!", var.name);
            passed = false
        } else {
            let value = var.required();
            if !(var.validator)(&value) {
                error!("Environment variable {} has an incorrect value!", var.name);
                passed = false;
            }
        }
    }

    for var in OPTIONAL_VARS {
        if let Some(value) = var.option() {
            if !(var.validator)(&value) {
                error!("Environment variable {} has an incorrect value!", var.name);
                passed = false;
            }
        }
    }

    passed
}

pub struct EnvVar {
    name: &'static str,
    validator: fn(&str) -> bool,
}

impl EnvVar {
    pub fn get(&self, default: &str) -> String {
        std::env::var(self.name).unwrap_or(default.to_string())
    }

    pub fn required(&self) -> String {
        std::env::var(self.name).unwrap()
    }

    pub fn option(&self) -> Option<String> {
        std::env::var(self.name).ok()
    }

    // TODO: Make this type-safe.
    pub fn get_bool(&self, default: bool) -> bool {
        self.option().map(|s| s == "true").unwrap_or(default)
    }
}
