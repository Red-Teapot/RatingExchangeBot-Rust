use log::error;

pub const DISCORD_BOT_TOKEN: EnvVar = EnvVar {
    name: "DISCORD_BOT_TOKEN",
    validator: non_empty_string,
};
pub const DATABASE_URL: EnvVar = EnvVar {
    name: "DATABASE_URL",
    validator: non_empty_string,
};

const REQUIRED_VARS: &[EnvVar] = &[DISCORD_BOT_TOKEN, DATABASE_URL];

fn non_empty_string(string: &str) -> bool {
    !string.trim().is_empty()
}

pub fn check_required_vars() -> bool {
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
}
