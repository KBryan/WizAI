use leptos::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Clone)]
pub struct AuthProvider {
    pub agent: Signal<Option<Agent>>,
    pub is_authenticated: ReadSignal<bool>,
    pub login: Callback<(String, String)>,
    pub logout: Callback<()>,
}

impl AuthProvider {
    pub fn new() -> Self {
        let (agent, set_agent) = create_signal(None);
        let is_authenticated = Signal::derive(move || agent().is_some());

        Self {
            agent,
            is_authenticated,
            login: Callback::new(move |(email, _): (String, String)| {
                let agent = Agent {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: email.split('@').next().unwrap_or("Agent").to_string(),
                    email,
                    role: "Agent".to_string(),
                };
                set_agent.set(Some(agent));
            }),
            logout: Callback::new(move |_| {
                set_agent.set(None);
            }),
        }
    }
}

impl Default for AuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_auth() -> AuthProvider {
    use_context().unwrap_or_else(AuthProvider::new)
}
